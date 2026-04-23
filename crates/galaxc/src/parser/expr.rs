// Expression parsing using Pratt parsing (operator precedence climbing).
// Handles all GalaxC expression forms including binary/unary operators,
// function calls, method calls, field access, closures, struct literals,
// error propagation (?), error conversion (!!), and pipelines (>>).

use super::core::Parser;
use crate::lexer::TokenKind;
use crate::ast::*;
use crate::diagnostics::Span;

/// Operator precedence levels, lowest to highest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    None,
    Pipeline,     // >>
    Or,           // or
    And,          // and
    Equality,     // == !=
    Comparison,   // < > <= >=
    Concat,       // ++
    BitOr,        // |
    BitXor,       // ^
    BitAnd,       // &
    Shift,        // << >>
    Range,        // ..
    Addition,     // + -
    Multiply,     // * / %
    ErrorConvert, // !!
    Unary,        // - not ~
    Postfix,      // ? . () []
}

impl Parser {
    /// Parse an expression at the default (lowest) precedence.
    pub(super) fn parse_expr(&mut self) -> Result<Expr, crate::diagnostics::Diagnostic> {
        self.parse_precedence(Precedence::None)
    }

    /// Core Pratt parser: parse an expression at the given minimum precedence.
    fn parse_precedence(
        &mut self,
        min_prec: Precedence,
    ) -> Result<Expr, crate::diagnostics::Diagnostic> {
        // Parse the prefix (left-hand side)
        let mut left = self.parse_prefix()?;

        // Parse infix and postfix operators at or above the minimum precedence
        loop {
            let prec = self.current_infix_precedence();
            if prec <= min_prec {
                break;
            }

            left = self.parse_infix(left, prec)?;
        }

        Ok(left)
    }

    /// Parse a prefix expression (literal, identifier, unary, grouping, etc.)
    fn parse_prefix(&mut self) -> Result<Expr, crate::diagnostics::Diagnostic> {
        match self.current_kind() {
            // Literals
            TokenKind::IntLiteral => self.parse_int_literal(),
            TokenKind::FloatLiteral => self.parse_float_literal(),
            TokenKind::StringLiteral => self.parse_string_literal(),
            TokenKind::CharLiteral => self.parse_char_literal(),
            TokenKind::True => self.parse_bool_literal(true),
            TokenKind::False => self.parse_bool_literal(false),
            TokenKind::None_ => self.parse_none_literal(),

            // Wrapping constructors
            TokenKind::Ok_ => self.parse_ok_expr(),
            TokenKind::Err_ => self.parse_err_expr(),
            TokenKind::Some_ => self.parse_some_expr(),

            // Identifiers and paths
            TokenKind::Identifier => self.parse_ident_or_path(),

            // Self
            TokenKind::SelfLower => {
                let span = self.current_span();
                self.advance();
                Ok(Expr::SelfExpr(SelfExprNode { span }))
            }

            // Unary operators
            TokenKind::Minus => {
                let start = self.current_span();
                self.advance();
                let operand = self.parse_precedence(Precedence::Unary)?;
                let end = operand.span();
                Ok(Expr::Unary(UnaryExpr {
                    op: UnaryOp::Neg,
                    operand: Box::new(operand),
                    span: start.merge(end),
                }))
            }
            TokenKind::Not => {
                let start = self.current_span();
                self.advance();
                let operand = self.parse_precedence(Precedence::Unary)?;
                let end = operand.span();
                Ok(Expr::Unary(UnaryExpr {
                    op: UnaryOp::Not,
                    operand: Box::new(operand),
                    span: start.merge(end),
                }))
            }
            TokenKind::Tilde => {
                let start = self.current_span();
                self.advance();
                let operand = self.parse_precedence(Precedence::Unary)?;
                let end = operand.span();
                Ok(Expr::Unary(UnaryExpr {
                    op: UnaryOp::BitNot,
                    operand: Box::new(operand),
                    span: start.merge(end),
                }))
            }

            // Grouping or tuple
            TokenKind::LParen => self.parse_paren_or_tuple(),

            // Array literal
            TokenKind::LBracket => self.parse_array_literal(),

            // Unsafe block
            TokenKind::Unsafe => self.parse_unsafe_block_expr(),

            // Closure
            TokenKind::Pipe => self.parse_closure(),

            _ => Err(self.error(format!(
                "expected expression, found {}",
                self.current_kind()
            ))),
        }
    }

    /// Determine the precedence of the current token when used as an infix operator.
    fn current_infix_precedence(&self) -> Precedence {
        match self.current_kind() {
            TokenKind::ShiftRight => Precedence::Pipeline, // >> is pipeline in expr context
            TokenKind::Or => Precedence::Or,
            TokenKind::And => Precedence::And,
            TokenKind::Eq | TokenKind::NotEq => Precedence::Equality,
            TokenKind::Lt | TokenKind::Gt | TokenKind::LtEq | TokenKind::GtEq => {
                Precedence::Comparison
            }
            TokenKind::PlusPlus => Precedence::Concat,
            TokenKind::Pipe => Precedence::BitOr,
            TokenKind::Caret => Precedence::BitXor,
            TokenKind::Ampersand => Precedence::BitAnd,
            TokenKind::ShiftLeft => Precedence::Shift,
            TokenKind::DotDot => Precedence::Range,
            TokenKind::Plus | TokenKind::Minus => Precedence::Addition,
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Precedence::Multiply,
            TokenKind::BangBang => Precedence::ErrorConvert,
            // Postfix: ?, ., (), []
            TokenKind::Question => Precedence::Postfix,
            TokenKind::Dot => Precedence::Postfix,
            TokenKind::LParen => Precedence::Postfix,
            TokenKind::LBracket => Precedence::Postfix,
            _ => Precedence::None,
        }
    }

    /// Parse an infix or postfix expression.
    fn parse_infix(
        &mut self,
        left: Expr,
        prec: Precedence,
    ) -> Result<Expr, crate::diagnostics::Diagnostic> {
        match self.current_kind() {
            // Postfix: error propagation
            TokenKind::Question => {
                let start = left.span();
                self.advance();
                Ok(Expr::Propagate(PropagateExpr {
                    inner: Box::new(left),
                    span: start.merge(self.previous_span()),
                }))
            }

            // Postfix: field access and method call
            TokenKind::Dot => {
                self.advance();
                let field_name = self.expect_identifier("field or method name");
                let start = left.span();

                // Check for method call
                if self.check(TokenKind::LParen) {
                    self.advance();
                    let args = self.parse_call_args()?;
                    self.expect(TokenKind::RParen);
                    let end = self.previous_span();
                    Ok(Expr::MethodCall(MethodCallExpr {
                        receiver: Box::new(left),
                        method: field_name,
                        args,
                        span: start.merge(end),
                    }))
                } else {
                    Ok(Expr::FieldAccess(FieldAccessExpr {
                        object: Box::new(left),
                        field: field_name,
                        span: start.merge(self.previous_span()),
                    }))
                }
            }

            // Postfix: function call
            TokenKind::LParen => {
                let start = left.span();
                self.advance();
                let args = self.parse_call_args()?;
                self.expect(TokenKind::RParen);
                let end = self.previous_span();
                Ok(Expr::Call(CallExpr {
                    callee: Box::new(left),
                    args,
                    span: start.merge(end),
                }))
            }

            // Postfix: index
            TokenKind::LBracket => {
                let start = left.span();
                self.advance();
                let index = self.parse_expr()?;
                self.expect(TokenKind::RBracket);
                let end = self.previous_span();
                Ok(Expr::Index(IndexExpr {
                    object: Box::new(left),
                    index: Box::new(index),
                    span: start.merge(end),
                }))
            }

            // Error conversion: !!
            TokenKind::BangBang => {
                let start = left.span();
                self.advance();
                let fallback = self.parse_precedence(prec)?;
                let end = fallback.span();
                Ok(Expr::ErrorConvert(ErrorConvertExpr {
                    inner: Box::new(left),
                    fallback: Box::new(fallback),
                    span: start.merge(end),
                }))
            }

            // String concatenation: ++
            TokenKind::PlusPlus => {
                let start = left.span();
                self.advance();
                let right = self.parse_precedence(prec)?;
                let end = right.span();
                Ok(Expr::Concat(ConcatExpr {
                    left: Box::new(left),
                    right: Box::new(right),
                    span: start.merge(end),
                }))
            }

            // Pipeline: >>
            TokenKind::ShiftRight if prec == Precedence::Pipeline => {
                let start = left.span();
                self.advance();
                let right = self.parse_precedence(prec)?;
                let end = right.span();
                Ok(Expr::Pipeline(PipelineExpr {
                    left: Box::new(left),
                    right: Box::new(right),
                    span: start.merge(end),
                }))
            }

            // Range: ..
            TokenKind::DotDot => {
                let start = left.span();
                self.advance();
                let right = self.parse_precedence(prec)?;
                let end = right.span();
                Ok(Expr::Range(RangeExpr {
                    start: Box::new(left),
                    end: Box::new(right),
                    inclusive: false,
                    span: start.merge(end),
                }))
            }

            // Binary operators
            _ => {
                let op = self.parse_binary_op()?;
                let right = self.parse_precedence(prec)?;
                let span = left.span().merge(right.span());
                Ok(Expr::Binary(BinaryExpr {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                    span,
                }))
            }
        }
    }

    fn parse_binary_op(&mut self) -> Result<BinaryOp, crate::diagnostics::Diagnostic> {
        let op = match self.current_kind() {
            TokenKind::Plus => BinaryOp::Add,
            TokenKind::Minus => BinaryOp::Sub,
            TokenKind::Star => BinaryOp::Mul,
            TokenKind::Slash => BinaryOp::Div,
            TokenKind::Percent => BinaryOp::Mod,
            TokenKind::Eq => BinaryOp::Eq,
            TokenKind::NotEq => BinaryOp::NotEq,
            TokenKind::Lt => BinaryOp::Lt,
            TokenKind::Gt => BinaryOp::Gt,
            TokenKind::LtEq => BinaryOp::LtEq,
            TokenKind::GtEq => BinaryOp::GtEq,
            TokenKind::And => BinaryOp::And,
            TokenKind::Or => BinaryOp::Or,
            TokenKind::Ampersand => BinaryOp::BitAnd,
            TokenKind::Pipe => BinaryOp::BitOr,
            TokenKind::Caret => BinaryOp::BitXor,
            TokenKind::ShiftLeft => BinaryOp::ShiftLeft,
            TokenKind::ShiftRight => BinaryOp::ShiftRight,
            _ => return Err(self.error(format!("expected operator, found {}", self.current_kind()))),
        };
        self.advance();
        Ok(op)
    }

    // -- Specific expression forms --

    fn parse_int_literal(&mut self) -> Result<Expr, crate::diagnostics::Diagnostic> {
        let span = self.current_span();
        let text = self.current().lexeme.clone();
        self.advance();
        let cleaned = text.replace('_', "");
        let value = if cleaned.starts_with("0x") || cleaned.starts_with("0X") {
            i64::from_str_radix(&cleaned[2..], 16)
        } else if cleaned.starts_with("0b") || cleaned.starts_with("0B") {
            i64::from_str_radix(&cleaned[2..], 2)
        } else if cleaned.starts_with("0o") || cleaned.starts_with("0O") {
            i64::from_str_radix(&cleaned[2..], 8)
        } else {
            cleaned.parse::<i64>()
        };

        match value {
            Ok(v) => Ok(Expr::Literal(LiteralExpr {
                value: LiteralValue::Int(v),
                span,
            })),
            Err(_) => Err(crate::diagnostics::Diagnostic::error(format!(
                "invalid integer literal: {text}"
            ))
            .with_span(span)),
        }
    }

    fn parse_float_literal(&mut self) -> Result<Expr, crate::diagnostics::Diagnostic> {
        let span = self.current_span();
        let text = self.current().lexeme.clone();
        self.advance();
        let cleaned = text.replace('_', "");
        match cleaned.parse::<f64>() {
            Ok(v) => Ok(Expr::Literal(LiteralExpr {
                value: LiteralValue::Float(v),
                span,
            })),
            Err(_) => Err(crate::diagnostics::Diagnostic::error(format!(
                "invalid float literal: {text}"
            ))
            .with_span(span)),
        }
    }

    fn parse_string_literal(&mut self) -> Result<Expr, crate::diagnostics::Diagnostic> {
        let span = self.current_span();
        let raw = self.current().lexeme.clone();
        self.advance();
        // Strip surrounding quotes and process escapes
        let inner = &raw[1..raw.len() - 1];
        let value = unescape_string(inner);
        Ok(Expr::Literal(LiteralExpr {
            value: LiteralValue::String(value),
            span,
        }))
    }

    fn parse_char_literal(&mut self) -> Result<Expr, crate::diagnostics::Diagnostic> {
        let span = self.current_span();
        let raw = self.current().lexeme.clone();
        self.advance();
        let inner = &raw[1..raw.len() - 1];
        let ch = if inner.starts_with('\\') {
            match inner.as_bytes().get(1) {
                Some(b'n') => '\n',
                Some(b't') => '\t',
                Some(b'r') => '\r',
                Some(b'\\') => '\\',
                Some(b'\'') => '\'',
                Some(b'0') => '\0',
                _ => inner.chars().next().unwrap_or('\0'),
            }
        } else {
            inner.chars().next().unwrap_or('\0')
        };
        Ok(Expr::Literal(LiteralExpr {
            value: LiteralValue::Char(ch),
            span,
        }))
    }

    fn parse_bool_literal(&mut self, val: bool) -> Result<Expr, crate::diagnostics::Diagnostic> {
        let span = self.current_span();
        self.advance();
        Ok(Expr::Literal(LiteralExpr {
            value: LiteralValue::Bool(val),
            span,
        }))
    }

    fn parse_none_literal(&mut self) -> Result<Expr, crate::diagnostics::Diagnostic> {
        let span = self.current_span();
        self.advance();
        Ok(Expr::Literal(LiteralExpr {
            value: LiteralValue::None,
            span,
        }))
    }

    fn parse_ok_expr(&mut self) -> Result<Expr, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        self.advance();
        self.expect(TokenKind::LParen);
        let inner = self.parse_expr()?;
        self.expect(TokenKind::RParen);
        let end = self.previous_span();
        Ok(Expr::Call(CallExpr {
            callee: Box::new(Expr::Identifier(IdentExpr {
                name: "ok".to_string(),
                span: start,
            })),
            args: vec![CallArg {
                name: None,
                value: inner,
                span: start.merge(end),
            }],
            span: start.merge(end),
        }))
    }

    fn parse_err_expr(&mut self) -> Result<Expr, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        self.advance();
        self.expect(TokenKind::LParen);
        let inner = self.parse_expr()?;
        self.expect(TokenKind::RParen);
        let end = self.previous_span();
        Ok(Expr::Call(CallExpr {
            callee: Box::new(Expr::Identifier(IdentExpr {
                name: "err".to_string(),
                span: start,
            })),
            args: vec![CallArg {
                name: None,
                value: inner,
                span: start.merge(end),
            }],
            span: start.merge(end),
        }))
    }

    fn parse_some_expr(&mut self) -> Result<Expr, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        self.advance();
        self.expect(TokenKind::LParen);
        let inner = self.parse_expr()?;
        self.expect(TokenKind::RParen);
        let end = self.previous_span();
        Ok(Expr::Call(CallExpr {
            callee: Box::new(Expr::Identifier(IdentExpr {
                name: "some".to_string(),
                span: start,
            })),
            args: vec![CallArg {
                name: None,
                value: inner,
                span: start.merge(end),
            }],
            span: start.merge(end),
        }))
    }

    fn parse_ident_or_path(&mut self) -> Result<Expr, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        let name = self.expect_identifier("expression");

        // Check for path: Foo.Bar or Foo::bar
        if self.check(TokenKind::Dot) && self.peek_ahead(1) == TokenKind::Identifier {
            // Could be field access on a value or an enum variant path.
            // We treat it as an identifier and let the postfix parsing handle the dot.
            // But for Type.Variant patterns we need qualified names.
            // Heuristic: if the name starts uppercase and next segment starts uppercase,
            // treat as path.
            let next_lexeme = self.tokens.get(self.pos + 1).map(|t| t.lexeme.as_str()).unwrap_or("");
            if name.starts_with(|c: char| c.is_uppercase())
                && next_lexeme.starts_with(|c: char| c.is_uppercase())
            {
                let mut segments = vec![name];
                while self.match_token(TokenKind::Dot) {
                    if self.check(TokenKind::Identifier) {
                        segments.push(self.expect_identifier("path segment"));
                    } else {
                        self.pos -= 1; // put dot back
                        break;
                    }
                }
                let end = self.previous_span();
                return Ok(Expr::Path(PathExpr {
                    segments,
                    span: start.merge(end),
                }));
            }
        }

        // Check for struct literal: Name { field: value, ... }
        if self.check(TokenKind::LBrace)
            && name.starts_with(|c: char| c.is_uppercase())
        {
            return self.parse_struct_literal(name, start);
        }

        Ok(Expr::Identifier(IdentExpr { name, span: start }))
    }

    fn parse_struct_literal(
        &mut self,
        name: String,
        start: Span,
    ) -> Result<Expr, crate::diagnostics::Diagnostic> {
        self.expect(TokenKind::LBrace);
        self.skip_newlines();
        let mut fields = Vec::new();

        while !self.check(TokenKind::RBrace) && !self.at_end() {
            self.skip_newlines();
            if self.check(TokenKind::RBrace) {
                break;
            }
            let f_start = self.current_span();
            let f_name = self.expect_identifier("field name");
            self.expect(TokenKind::Colon);
            let f_value = self.parse_expr()?;
            let f_end = self.previous_span();

            fields.push(StructFieldInit {
                name: f_name,
                value: f_value,
                span: f_start.merge(f_end),
            });

            if !self.match_token(TokenKind::Comma) {
                self.skip_newlines();
            }
            self.skip_newlines();
        }

        self.expect(TokenKind::RBrace);
        let end = self.previous_span();

        Ok(Expr::StructLiteral(StructLiteralExpr {
            name,
            fields,
            span: start.merge(end),
        }))
    }

    fn parse_paren_or_tuple(&mut self) -> Result<Expr, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        self.advance(); // consume (

        // Empty tuple
        if self.check(TokenKind::RParen) {
            self.advance();
            return Ok(Expr::TupleLiteral(TupleLiteralExpr {
                elements: Vec::new(),
                span: start.merge(self.previous_span()),
            }));
        }

        let first = self.parse_expr()?;

        // Single element with comma = tuple, without comma = grouping
        if self.match_token(TokenKind::Comma) {
            let mut elements = vec![first];
            while !self.check(TokenKind::RParen) && !self.at_end() {
                elements.push(self.parse_expr()?);
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
            self.expect(TokenKind::RParen);
            Ok(Expr::TupleLiteral(TupleLiteralExpr {
                elements,
                span: start.merge(self.previous_span()),
            }))
        } else {
            self.expect(TokenKind::RParen);
            Ok(first) // just grouping
        }
    }

    fn parse_array_literal(&mut self) -> Result<Expr, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        self.advance(); // [
        let mut elements = Vec::new();

        while !self.check(TokenKind::RBracket) && !self.at_end() {
            elements.push(self.parse_expr()?);
            if !self.match_token(TokenKind::Comma) && !self.match_token(TokenKind::Semicolon) {
                break;
            }
        }

        self.expect(TokenKind::RBracket);
        Ok(Expr::ArrayLiteral(ArrayLiteralExpr {
            elements,
            span: start.merge(self.previous_span()),
        }))
    }

    fn parse_unsafe_block_expr(&mut self) -> Result<Expr, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        self.expect(TokenKind::Unsafe);
        self.expect(TokenKind::FatArrow);
        let body = self.parse_block_body(start)?;
        Ok(Expr::UnsafeBlock(UnsafeBlockExpr {
            body,
            span: start.merge(self.previous_span()),
        }))
    }

    fn parse_closure(&mut self) -> Result<Expr, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        self.expect(TokenKind::Pipe);

        let mut params = Vec::new();
        while !self.check(TokenKind::Pipe) && !self.at_end() {
            let p_start = self.current_span();
            let name = self.expect_identifier("closure parameter");
            let type_expr = if self.match_token(TokenKind::Colon) {
                Some(self.parse_type()?)
            } else {
                None
            };
            params.push(ClosureParam {
                name,
                type_expr,
                span: p_start.merge(self.previous_span()),
            });
            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }

        self.expect(TokenKind::Pipe);
        let body = self.parse_expr()?;
        let end = body.span();

        Ok(Expr::Closure(ClosureExpr {
            params,
            body: Box::new(body),
            span: start.merge(end),
        }))
    }

    fn parse_call_args(&mut self) -> Result<Vec<CallArg>, crate::diagnostics::Diagnostic> {
        let mut args = Vec::new();
        if self.check(TokenKind::RParen) {
            return Ok(args);
        }

        loop {
            let arg_start = self.current_span();

            // Check for named argument: name: value
            let name = if self.check(TokenKind::Identifier)
                && self.peek_ahead(1) == TokenKind::Colon
            {
                let n = self.expect_identifier("argument name");
                self.expect(TokenKind::Colon);
                Some(n)
            } else {
                None
            };

            let value = self.parse_expr()?;
            let arg_end = self.previous_span();

            args.push(CallArg {
                name,
                value,
                span: arg_start.merge(arg_end),
            });

            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }

        Ok(args)
    }
}

/// Process escape sequences in a string literal body.
fn unescape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('r') => result.push('\r'),
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some('0') => result.push('\0'),
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }
    result
}
