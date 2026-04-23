// Statement parsing -- let, var, assign, if, match, for, while, loop,
// return, break, continue, select, and expression statements.

use super::core::Parser;
use crate::lexer::TokenKind;
use crate::ast::*;
// Span import removed

impl Parser {
    /// Parse a single statement. Dispatches based on the leading token.
    pub(super) fn parse_stmt(&mut self) -> Result<Stmt, crate::diagnostics::Diagnostic> {
        self.skip_newlines();

        match self.current_kind() {
            TokenKind::Let => self.parse_let_stmt(),
            TokenKind::Var => self.parse_var_stmt(),
            TokenKind::If => self.parse_if_stmt(),
            TokenKind::Match => self.parse_match_stmt(),
            TokenKind::For => self.parse_for_stmt(),
            TokenKind::While => self.parse_while_stmt(),
            TokenKind::Loop => self.parse_loop_stmt(),
            TokenKind::Return => self.parse_return_stmt(),
            TokenKind::Break => self.parse_break_stmt(),
            TokenKind::Continue => self.parse_continue_stmt(),
            TokenKind::Select => self.parse_select_stmt(),

            // Nested item declarations
            TokenKind::At | TokenKind::Pub | TokenKind::Op | TokenKind::Struct
            | TokenKind::Enum | TokenKind::Ability | TokenKind::Impl | TokenKind::Const
            | TokenKind::Task | TokenKind::Protected => {
                let item = self.parse_item()?;
                Ok(Stmt::Item(Box::new(item)))
            }

            // Everything else is an expression statement (possibly with assignment)
            _ => self.parse_expr_or_assign_stmt(),
        }
    }

    fn parse_let_stmt(&mut self) -> Result<Stmt, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        self.expect(TokenKind::Let);
        let name = self.expect_identifier("variable name");

        let type_expr = if self.match_token(TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(TokenKind::Assign);
        let value = self.parse_expr()?;
        let end = self.previous_span();
        self.expect_line_end();

        Ok(Stmt::Let(LetStmt {
            name,
            type_expr,
            value,
            span: start.merge(end),
        }))
    }

    fn parse_var_stmt(&mut self) -> Result<Stmt, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        self.expect(TokenKind::Var);
        let name = self.expect_identifier("variable name");

        let type_expr = if self.match_token(TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(TokenKind::Assign);
        let value = self.parse_expr()?;
        let end = self.previous_span();
        self.expect_line_end();

        Ok(Stmt::Var(VarStmt {
            name,
            type_expr,
            value,
            span: start.merge(end),
        }))
    }

    fn parse_if_stmt(&mut self) -> Result<Stmt, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        self.expect(TokenKind::If);
        let condition = self.parse_expr()?;
        self.expect(TokenKind::FatArrow);
        let then_block = self.parse_block_body(start)?;

        let mut else_ifs = Vec::new();
        let mut else_block = None;

        // Look for `else if` and `else` chains. The `end` from parse_block_body
        // already consumed the first `end`, but GalaxC uses a single `end` for
        // the whole if/else chain. We need to handle this differently.
        //
        // Actually, each branch ends with `end`, and else/else-if come after.
        // Let me re-read the grammar: the `end` closes the whole if.
        // Correction: in GalaxC, branches within an if share one `end`:
        //   if cond =>
        //       ...
        //   else if cond =>
        //       ...
        //   else =>
        //       ...
        //   end
        //
        // So the block_body parser should NOT consume `end` here. We need a
        // variant that stops at `else` too.

        // For now (pragmatic approach): we already consumed `end` in block_body.
        // Check if there is an else/else-if following. If the pattern is
        // canonical, the user places else on the line after end. So we
        // re-check after the end.

        self.skip_newlines();
        while self.check(TokenKind::Else) {
            self.advance(); // consume `else`
            if self.match_token(TokenKind::If) {
                // else if
                let cond = self.parse_expr()?;
                self.expect(TokenKind::FatArrow);
                let block = self.parse_block_body(start)?;
                else_ifs.push((cond, block));
            } else {
                // else
                self.expect(TokenKind::FatArrow);
                else_block = Some(self.parse_block_body(start)?);
                break;
            }
            self.skip_newlines();
        }

        let end = self.previous_span();

        Ok(Stmt::If(IfStmt {
            condition,
            then_block,
            else_ifs,
            else_block,
            span: start.merge(end),
        }))
    }

    fn parse_match_stmt(&mut self) -> Result<Stmt, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        self.expect(TokenKind::Match);
        let subject = self.parse_expr()?;
        self.expect(TokenKind::FatArrow);
        self.skip_newlines();

        let mut arms = Vec::new();
        while !self.check(TokenKind::End) && !self.at_end() {
            self.skip_newlines();
            if self.check(TokenKind::End) {
                break;
            }

            let arm_start = self.current_span();
            let pattern = self.parse_pattern()?;

            let guard = if self.match_token(TokenKind::When) {
                Some(self.parse_expr()?)
            } else {
                None
            };

            self.expect(TokenKind::FatArrow);

            // Arm body: either a block (multi-line with `end`) or a single expression
            let body = if self.check(TokenKind::Newline) || self.check(TokenKind::End) {
                // Multi-line arm body
                let block = self.parse_block_body(arm_start)?;
                MatchArmBody::Block(block)
            } else {
                // Single expression arm
                let expr = self.parse_expr()?;
                MatchArmBody::Expr(expr)
            };

            let arm_end = self.previous_span();
            arms.push(MatchArm {
                pattern,
                guard,
                body,
                span: arm_start.merge(arm_end),
            });
            self.skip_newlines();
        }

        let end = self.current_span();
        self.expect(TokenKind::End);

        Ok(Stmt::Match(MatchStmt {
            subject,
            arms,
            span: start.merge(end),
        }))
    }

    fn parse_for_stmt(&mut self) -> Result<Stmt, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        self.expect(TokenKind::For);
        let binding = self.expect_identifier("loop variable");
        self.expect(TokenKind::In);
        let iterable = self.parse_expr()?;
        self.expect(TokenKind::FatArrow);
        let body = self.parse_block_body(start)?;

        Ok(Stmt::For(ForStmt {
            binding,
            iterable,
            body,
            span: start.merge(self.previous_span()),
        }))
    }

    fn parse_while_stmt(&mut self) -> Result<Stmt, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        self.expect(TokenKind::While);
        let condition = self.parse_expr()?;
        self.expect(TokenKind::FatArrow);
        let body = self.parse_block_body(start)?;

        Ok(Stmt::While(WhileStmt {
            condition,
            body,
            span: start.merge(self.previous_span()),
        }))
    }

    fn parse_loop_stmt(&mut self) -> Result<Stmt, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        self.expect(TokenKind::Loop);
        self.expect(TokenKind::FatArrow);
        let body = self.parse_block_body(start)?;

        Ok(Stmt::Loop(LoopStmt {
            body,
            span: start.merge(self.previous_span()),
        }))
    }

    fn parse_return_stmt(&mut self) -> Result<Stmt, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        self.expect(TokenKind::Return);

        let value = if self.check(TokenKind::Newline) || self.check(TokenKind::Eof)
            || self.check(TokenKind::End)
        {
            None
        } else {
            Some(self.parse_expr()?)
        };

        let end = self.previous_span();

        Ok(Stmt::Return(ReturnStmt {
            value,
            span: start.merge(end),
        }))
    }

    fn parse_break_stmt(&mut self) -> Result<Stmt, crate::diagnostics::Diagnostic> {
        let span = self.current_span();
        self.expect(TokenKind::Break);
        Ok(Stmt::Break(BreakStmt { span }))
    }

    fn parse_continue_stmt(&mut self) -> Result<Stmt, crate::diagnostics::Diagnostic> {
        let span = self.current_span();
        self.expect(TokenKind::Continue);
        Ok(Stmt::Continue(ContinueStmt { span }))
    }

    fn parse_select_stmt(&mut self) -> Result<Stmt, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        self.expect(TokenKind::Select);
        self.expect(TokenKind::FatArrow);
        self.skip_newlines();

        let mut arms = Vec::new();
        let mut first = true;

        while !self.check(TokenKind::End) && !self.at_end() {
            self.skip_newlines();
            if self.check(TokenKind::End) {
                break;
            }

            // After the first arm, expect `or`
            if !first {
                if !self.match_token(TokenKind::Or) {
                    break;
                }
                self.skip_newlines();
            }
            first = false;

            let arm = self.parse_select_arm()?;
            arms.push(arm);
            self.skip_newlines();
        }

        let end = self.current_span();
        self.expect(TokenKind::End);

        Ok(Stmt::Select(SelectStmt {
            arms,
            span: start.merge(end),
        }))
    }

    fn parse_select_arm(&mut self) -> Result<SelectArm, crate::diagnostics::Diagnostic> {
        if self.check(TokenKind::Accept) {
            let arm_start = self.current_span();
            self.advance();
            let entry_name = self.expect_identifier("entry name");
            self.expect(TokenKind::LParen);
            let params = self.parse_params()?;
            self.expect(TokenKind::RParen);
            self.expect(TokenKind::FatArrow);
            let body = self.parse_block_body(arm_start)?;

            Ok(SelectArm::Accept {
                entry_name,
                params,
                body,
                span: arm_start.merge(self.previous_span()),
            })
        } else if self.check(TokenKind::Delay) {
            let arm_start = self.current_span();
            self.advance();
            let duration = self.parse_expr()?;
            self.expect(TokenKind::FatArrow);
            let body = self.parse_block_body(arm_start)?;

            Ok(SelectArm::Delay {
                duration,
                body,
                span: arm_start.merge(self.previous_span()),
            })
        } else if self.check(TokenKind::When) {
            let arm_start = self.current_span();
            self.advance();
            let guard = self.parse_expr()?;
            self.expect(TokenKind::FatArrow);
            self.skip_newlines();
            let inner = self.parse_select_arm()?;

            Ok(SelectArm::When {
                guard,
                accept: Box::new(inner),
                span: arm_start.merge(self.previous_span()),
            })
        } else {
            Err(self.error("expected 'accept', 'delay', or 'when' in select statement"))
        }
    }

    /// Parse an expression statement or an assignment statement.
    /// We parse the left-hand side as an expression, then check if an
    /// assignment operator follows.
    fn parse_expr_or_assign_stmt(&mut self) -> Result<Stmt, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        let expr = self.parse_expr()?;

        // Check for assignment operators
        let op = match self.current_kind() {
            TokenKind::Assign => Some(AssignOp::Assign),
            TokenKind::PlusAssign => Some(AssignOp::AddAssign),
            TokenKind::MinusAssign => Some(AssignOp::SubAssign),
            TokenKind::StarAssign => Some(AssignOp::MulAssign),
            TokenKind::SlashAssign => Some(AssignOp::DivAssign),
            TokenKind::PercentAssign => Some(AssignOp::ModAssign),
            _ => None,
        };

        if let Some(assign_op) = op {
            self.advance(); // consume the operator
            let value = self.parse_expr()?;
            let end = self.previous_span();
            Ok(Stmt::Assign(AssignStmt {
                target: expr,
                op: assign_op,
                value,
                span: start.merge(end),
            }))
        } else {
            let span = expr.span();
            Ok(Stmt::Expr(ExprStmt { expr, span }))
        }
    }
}
