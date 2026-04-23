// Type expression parsing. Handles named types, generics, unit-annotated
// types, arrays, slices, tuples, references, Option/Result shorthands,
// and qualified paths.

use super::core::Parser;
use crate::lexer::TokenKind;
use crate::ast::*;
use crate::diagnostics::{Diagnostic, Span};

impl Parser {
    /// Parse a type expression.
    pub(super) fn parse_type(&mut self) -> Result<TypeExpr, Diagnostic> {
        // Check for reference types
        if self.check(TokenKind::Ref) {
            return self.parse_ref_type();
        }
        if self.check(TokenKind::Mut) && self.peek_ahead(1) == TokenKind::Ref {
            return self.parse_mut_ref_type();
        }

        // Check for array type: [T; N]
        if self.check(TokenKind::LBracket) {
            return self.parse_array_type();
        }

        // Check for tuple type: (T, U, ...)
        if self.check(TokenKind::LParen) {
            return self.parse_tuple_type();
        }

        // Self type
        if self.check(TokenKind::SelfUpper) {
            let span = self.current_span();
            self.advance();
            return Ok(TypeExpr::SelfType { span });
        }

        // Named type (possibly with generics or unit annotation)
        self.parse_named_type()
    }

    fn parse_named_type(&mut self) -> Result<TypeExpr, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        let name = self.expect_identifier("type name");

        // Check for path: Module::Type
        if self.check(TokenKind::ColonColon) {
            let mut segments = vec![name];
            while self.match_token(TokenKind::ColonColon) {
                segments.push(self.expect_identifier("type path segment"));
            }

            let generics = if self.check(TokenKind::Lt) {
                self.parse_type_args()?
            } else {
                Vec::new()
            };

            let end = self.previous_span();
            return Ok(TypeExpr::Path {
                segments,
                generics,
                span: start.merge(end),
            });
        }

        // Check for generic arguments or unit annotation: Type<...>
        if self.check(TokenKind::Lt) {
            // Distinguish between generics and unit annotation.
            // If the contents are a single lowercase identifier (or compound unit expression),
            // treat it as a unit annotation. Otherwise, treat as generics.
            // Heuristic: if the first token inside < > is a lowercase identifier and
            // there is no comma, it is likely a unit.

            // For simplicity, try to parse as generic args first.
            let saved_pos = self.pos;
            match self.try_parse_unit_annotation(&name, start) {
                Some(unit_type) => return Ok(unit_type),
                None => {
                    self.pos = saved_pos;
                    let generics = self.parse_type_args()?;
                    let end = self.previous_span();
                    return Ok(TypeExpr::Named {
                        name,
                        generics,
                        span: start.merge(end),
                    });
                }
            }
        }

        Ok(TypeExpr::Named {
            name,
            generics: Vec::new(),
            span: start,
        })
    }

    fn try_parse_unit_annotation(&mut self, base: &str, start: Span) -> Option<TypeExpr> {
        // Look ahead: if < is followed by a lowercase identifier and then >,
        // this is a unit annotation.
        if self.check(TokenKind::Lt) {
            let saved = self.pos;
            self.advance(); // <

            if self.check(TokenKind::Identifier) {
                let ident = self.current().lexeme.clone();
                if ident.chars().next().map_or(false, |c: char| c.is_lowercase()) {
                    self.advance();

                    // Might have compound units like meters_per_second
                    if self.check(TokenKind::Gt) {
                        self.advance();
                        return Some(TypeExpr::UnitType {
                            base: base.to_string(),
                            unit: ident,
                            span: start.merge(self.previous_span()),
                        });
                    }
                }
            }

            // Not a unit annotation, rewind
            self.pos = saved;
        }
        None
    }

    fn parse_type_args(&mut self) -> Result<Vec<TypeExpr>, crate::diagnostics::Diagnostic> {
        self.expect(TokenKind::Lt);
        let mut args = Vec::new();

        loop {
            if self.check(TokenKind::Gt) {
                break;
            }
            args.push(self.parse_type()?);
            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }

        self.expect(TokenKind::Gt);
        Ok(args)
    }

    fn parse_ref_type(&mut self) -> Result<TypeExpr, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        self.expect(TokenKind::Ref);
        let inner = self.parse_type()?;
        let end = inner.span();
        Ok(TypeExpr::Reference {
            inner: Box::new(inner),
            is_mut: false,
            span: start.merge(end),
        })
    }

    fn parse_mut_ref_type(&mut self) -> Result<TypeExpr, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        self.expect(TokenKind::Mut);
        self.expect(TokenKind::Ref);
        let inner = self.parse_type()?;
        let end = inner.span();
        Ok(TypeExpr::Reference {
            inner: Box::new(inner),
            is_mut: true,
            span: start.merge(end),
        })
    }

    fn parse_array_type(&mut self) -> Result<TypeExpr, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        self.expect(TokenKind::LBracket);
        let element = self.parse_type()?;
        self.expect(TokenKind::Semicolon);
        let size = self.parse_expr()?;
        self.expect(TokenKind::RBracket);
        let end = self.previous_span();

        Ok(TypeExpr::Array {
            element: Box::new(element),
            size: Box::new(size),
            span: start.merge(end),
        })
    }

    fn parse_tuple_type(&mut self) -> Result<TypeExpr, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        self.expect(TokenKind::LParen);

        let mut elements = Vec::new();
        while !self.check(TokenKind::RParen) && !self.at_end() {
            elements.push(self.parse_type()?);
            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }

        self.expect(TokenKind::RParen);
        let end = self.previous_span();

        Ok(TypeExpr::Tuple {
            elements,
            span: start.merge(end),
        })
    }
}
