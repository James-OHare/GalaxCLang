// Declaration parsing -- functions, structs, enums, abilities, impl blocks,
// tasks, protected objects, constants, and extern blocks.

use super::core::Parser;
use crate::lexer::TokenKind;
use crate::ast::*;
use crate::diagnostics::Span;

impl Parser {
    /// Parse any top-level item. Dispatches based on the current token,
    /// handling annotations and visibility modifiers first.
    pub(super) fn parse_item(&mut self) -> Result<Item, crate::diagnostics::Diagnostic> {
        self.skip_newlines();

        // Collect annotations
        let annotations = self.parse_annotations();

        // Check for visibility modifier
        let is_pub = self.match_token(TokenKind::Pub);

        let item = match self.current_kind() {
            TokenKind::Op => {
                let mut func = self.parse_function()?;
                func.annotations = annotations;
                func.is_pub = is_pub;
                Item::Function(func)
            }
            TokenKind::Struct => {
                let mut s = self.parse_struct_decl()?;
                s.is_pub = is_pub;
                Item::Struct(s)
            }
            TokenKind::Enum => {
                let mut e = self.parse_enum_decl()?;
                e.is_pub = is_pub;
                Item::Enum(e)
            }
            TokenKind::Ability => {
                let mut a = self.parse_ability_decl()?;
                a.is_pub = is_pub;
                Item::Ability(a)
            }
            TokenKind::Impl => Item::ImplBlock(self.parse_impl_block()?),
            TokenKind::Const => {
                let mut c = self.parse_const_decl()?;
                c.is_pub = is_pub;
                Item::Constant(c)
            }
            TokenKind::Task => {
                // Distinguish between task declaration and task body
                if self.peek_ahead(1) == TokenKind::Body {
                    Item::TaskBody(self.parse_task_body()?)
                } else {
                    let mut t = self.parse_task_decl()?;
                    t.is_pub = is_pub;
                    Item::TaskDecl(t)
                }
            }
            TokenKind::Protected => {
                let mut p = self.parse_protected_decl()?;
                p.is_pub = is_pub;
                Item::ProtectedDecl(p)
            }
            TokenKind::Unit => Item::UnitDecl(self.parse_unit_decl()?),
            TokenKind::Extern => Item::ExternBlock(self.parse_extern_block()?),
            TokenKind::StaticAssert => Item::StaticAssert(self.parse_static_assert()?),
            _ => {
                return Err(self.error(format!(
                    "expected declaration, found {}",
                    self.current_kind()
                )));
            }
        };

        Ok(item)
    }

    // -- Annotations --

    pub(super) fn parse_annotations(&mut self) -> Vec<Annotation> {
        let mut annotations = Vec::new();
        while self.check(TokenKind::At) {
            annotations.push(self.parse_annotation());
            self.skip_newlines();
        }
        annotations
    }

    fn parse_annotation(&mut self) -> Annotation {
        let start = self.current_span();
        self.expect(TokenKind::At);
        let name = self.expect_identifier("annotation name");
        let mut args = Vec::new();

        if self.match_token(TokenKind::LParen) {
            loop {
                if self.check(TokenKind::RParen) {
                    break;
                }
                let arg_start = self.current_span();

                // Check for key: value form
                let key = if self.check(TokenKind::Identifier)
                    && self.peek_ahead(1) == TokenKind::Colon
                {
                    let k = self.expect_identifier("annotation key");
                    self.expect(TokenKind::Colon);
                    Some(k)
                } else {
                    None
                };

                let value = self.parse_expr().unwrap_or_else(|_| {
                    Expr::Literal(LiteralExpr {
                        value: LiteralValue::None,
                        span: self.current_span(),
                    })
                });

                let arg_end = self.previous_span();
                args.push(AnnotationArg {
                    key,
                    value,
                    span: arg_start.merge(arg_end),
                });

                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
            self.expect(TokenKind::RParen);
        }

        let end = self.previous_span();
        Annotation {
            name,
            args,
            span: start.merge(end),
        }
    }

    // -- Functions --

    pub(super) fn parse_function(
        &mut self,
    ) -> Result<FunctionDecl, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        self.expect(TokenKind::Op);
        let name = self.expect_identifier("function name");

        // Generic parameters
        let generics = if self.check(TokenKind::Lt) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };

        // Parameters
        self.expect(TokenKind::LParen);
        let params = self.parse_params()?;
        self.expect(TokenKind::RParen);

        // Return type
        let return_type = if self.match_token(TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        // Body (=> ... end) or just a declaration (in abilities)
        let body = if self.match_token(TokenKind::FatArrow) {
            let has_newline = self.check(TokenKind::Newline);
            if has_newline {
                self.advance(); // consume the newline
                Some(self.parse_block_body(start)?)
            } else {
                // Same line as =>
                self.skip_newlines();
                if self.check(TokenKind::End) {
                    // => end (empty block)
                    Some(self.parse_block_body(start)?)
                } else {
                    let expr = self.parse_expr()?;
                    // Check if anything else follows on the same line
                    if self.check(TokenKind::Newline) || self.at_end() {
                        // Inline single-expression body
                        let span = expr.span();
                        Some(Block {
                            stmts: vec![Stmt::Return(ReturnStmt {
                                value: Some(expr),
                                span,
                            })],
                            span,
                        })
                    } else {
                        // Multi-statement block on one line (unusual but possible)
                        let mut stmts = vec![Stmt::Expr(ExprStmt { span: expr.span(), expr })];
                        self.skip_newlines();
                        while !self.check(TokenKind::End) && !self.at_end() {
                            stmts.push(self.parse_stmt()?);
                            self.skip_newlines();
                        }
                        let end_span = self.current_span();
                        self.expect(TokenKind::End);
                        Some(Block {
                            stmts,
                            span: start.merge(end_span),
                        })
                    }
                }
            }
        } else {
            None
        };

        let end = self.previous_span();

        Ok(FunctionDecl {
            name,
            annotations: Vec::new(), // filled in by caller
            generics,
            params,
            return_type,
            body,
            is_pub: false, // filled in by caller
            span: start.merge(end),
        })
    }

    pub(super) fn parse_params(&mut self) -> Result<Vec<Param>, crate::diagnostics::Diagnostic> {
        let mut params = Vec::new();
        if self.check(TokenKind::RParen) {
            return Ok(params);
        }

        loop {
            let param_start = self.current_span();

            // Check for `self` parameter
            if self.check(TokenKind::SelfLower) {
                let span = self.current_span();
                self.advance();
                params.push(Param {
                    name: "self".to_string(),
                    type_expr: TypeExpr::SelfType { span },
                    is_mut: false,
                    span,
                });
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
                continue;
            }

            // Check for `mut self`
            if self.check(TokenKind::Mut) && self.peek_ahead(1) == TokenKind::SelfLower {
                let span = self.current_span();
                self.advance(); // mut
                self.advance(); // self
                params.push(Param {
                    name: "self".to_string(),
                    type_expr: TypeExpr::SelfType { span },
                    is_mut: true,
                    span,
                });
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
                continue;
            }

            let is_mut = self.match_token(TokenKind::Mut);
            let name = self.expect_identifier("parameter name");
            self.expect(TokenKind::Colon);
            let type_expr = self.parse_type()?;
            let param_end = self.previous_span();

            params.push(Param {
                name,
                type_expr,
                is_mut,
                span: param_start.merge(param_end),
            });

            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }

        Ok(params)
    }

    fn parse_generic_params(
        &mut self,
    ) -> Result<Vec<GenericParam>, crate::diagnostics::Diagnostic> {
        self.expect(TokenKind::Lt);
        let mut params = Vec::new();

        loop {
            if self.check(TokenKind::Gt) {
                break;
            }

            let start = self.current_span();
            let name = self.expect_identifier("type parameter name");

            // Bounds: T: Ability1 + Ability2
            let bounds = if self.match_token(TokenKind::Colon) {
                let mut b = vec![self.parse_type()?];
                while self.match_token(TokenKind::Plus) {
                    b.push(self.parse_type()?);
                }
                b
            } else {
                Vec::new()
            };

            let end = self.previous_span();
            params.push(GenericParam {
                name,
                bounds,
                span: start.merge(end),
            });

            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }

        self.expect(TokenKind::Gt);
        Ok(params)
    }

    /// Parse the body of a block (after the => has been consumed).
    /// Reads statements until `end`.
    pub(super) fn parse_block_body(
        &mut self,
        start: Span,
    ) -> Result<Block, crate::diagnostics::Diagnostic> {
        let mut stmts = Vec::new();
        self.skip_newlines();

        while !self.check(TokenKind::End) && !self.at_end() {
            let stmt = self.parse_stmt()?;
            stmts.push(stmt);
            self.skip_newlines();
        }

        let end = self.current_span();
        self.expect(TokenKind::End);

        Ok(Block {
            stmts,
            span: start.merge(end),
        })
    }

    // -- Structs --

    fn parse_struct_decl(&mut self) -> Result<StructDecl, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        self.expect(TokenKind::Struct);
        let name = self.expect_identifier("struct name");

        let generics = if self.check(TokenKind::Lt) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };

        self.expect(TokenKind::FatArrow);
        self.skip_newlines();

        let mut fields = Vec::new();
        while !self.check(TokenKind::End) && !self.at_end() {
            let field_start = self.current_span();
            let field_name = self.expect_identifier("field name");
            self.expect(TokenKind::Colon);
            let field_type = self.parse_type()?;
            let field_end = self.previous_span();

            fields.push(FieldDecl {
                name: field_name,
                type_expr: field_type,
                span: field_start.merge(field_end),
            });
            self.skip_newlines();
        }

        let end = self.current_span();
        self.expect(TokenKind::End);

        Ok(StructDecl {
            name,
            generics,
            fields,
            is_pub: false,
            span: start.merge(end),
        })
    }

    // -- Enums --

    fn parse_enum_decl(&mut self) -> Result<EnumDecl, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        self.expect(TokenKind::Enum);
        let name = self.expect_identifier("enum name");

        let generics = if self.check(TokenKind::Lt) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };

        self.expect(TokenKind::FatArrow);
        self.skip_newlines();

        let mut variants = Vec::new();
        while !self.check(TokenKind::End) && !self.at_end() {
            let var_start = self.current_span();
            let var_name = self.expect_identifier("variant name");

            let fields = if self.match_token(TokenKind::LParen) {
                let mut f = Vec::new();
                loop {
                    if self.check(TokenKind::RParen) {
                        break;
                    }
                    let f_start = self.current_span();
                    let f_name = self.expect_identifier("field name");
                    self.expect(TokenKind::Colon);
                    let f_type = self.parse_type()?;
                    let f_end = self.previous_span();
                    f.push(FieldDecl {
                        name: f_name,
                        type_expr: f_type,
                        span: f_start.merge(f_end),
                    });
                    if !self.match_token(TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RParen);
                f
            } else {
                Vec::new()
            };

            let var_end = self.previous_span();
            variants.push(VariantDecl {
                name: var_name,
                fields,
                span: var_start.merge(var_end),
            });
            self.skip_newlines();
        }

        let end = self.current_span();
        self.expect(TokenKind::End);

        Ok(EnumDecl {
            name,
            generics,
            variants,
            is_pub: false,
            span: start.merge(end),
        })
    }

    // -- Abilities --

    fn parse_ability_decl(&mut self) -> Result<AbilityDecl, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        self.expect(TokenKind::Ability);
        let name = self.expect_identifier("ability name");

        let generics = if self.check(TokenKind::Lt) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };

        self.expect(TokenKind::FatArrow);
        self.skip_newlines();

        let mut methods = Vec::new();
        let mut constants = Vec::new();

        while !self.check(TokenKind::End) && !self.at_end() {
            self.skip_newlines();
            if self.check(TokenKind::End) {
                break;
            }
            let annotations = self.parse_annotations();
            if self.check(TokenKind::Const) {
                let c = self.parse_const_decl()?;
                constants.push(c);
            } else if self.check(TokenKind::Op) {
                let mut f = self.parse_function()?;
                f.annotations = annotations;
                methods.push(f);
            } else {
                return Err(self.error("expected 'op' or 'const' in ability declaration"));
            }
            self.skip_newlines();
        }

        let end = self.current_span();
        self.expect(TokenKind::End);

        Ok(AbilityDecl {
            name,
            generics,
            methods,
            constants,
            is_pub: false,
            span: start.merge(end),
        })
    }

    // -- Impl blocks --

    fn parse_impl_block(&mut self) -> Result<ImplBlock, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        self.expect(TokenKind::Impl);

        let first_name = self.expect_identifier("type or ability name");

        // Check for `impl Ability for Type`
        let (ability, target) = if self.match_token(TokenKind::For) {
            let target = self.expect_identifier("type name");
            (Some(first_name), target)
        } else {
            (None, first_name)
        };

        self.expect(TokenKind::FatArrow);
        self.skip_newlines();

        let mut methods = Vec::new();
        while !self.check(TokenKind::End) && !self.at_end() {
            self.skip_newlines();
            if self.check(TokenKind::End) {
                break;
            }
            let annotations = self.parse_annotations();
            let mut f = self.parse_function()?;
            f.annotations = annotations;
            methods.push(f);
            self.skip_newlines();
        }

        let end = self.current_span();
        self.expect(TokenKind::End);

        Ok(ImplBlock {
            ability,
            target,
            methods,
            span: start.merge(end),
        })
    }

    // -- Constants --

    fn parse_const_decl(&mut self) -> Result<ConstDecl, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        self.expect(TokenKind::Const);
        let name = self.expect_identifier("constant name");

        let type_expr = if self.match_token(TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(TokenKind::Assign);
        let value = self.parse_expr()?;
        let end = self.previous_span();

        Ok(ConstDecl {
            name,
            type_expr,
            value,
            is_pub: false,
            span: start.merge(end),
        })
    }

    // -- Tasks --

    fn parse_task_decl(&mut self) -> Result<TaskDecl, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        self.expect(TokenKind::Task);
        let name = self.expect_identifier("task name");
        self.expect(TokenKind::FatArrow);
        self.skip_newlines();

        let mut entries = Vec::new();
        while !self.check(TokenKind::End) && !self.at_end() {
            self.skip_newlines();
            if self.check(TokenKind::End) {
                break;
            }
            let annotations = self.parse_annotations();
            let entry_start = self.current_span();
            self.expect(TokenKind::Op);
            let entry_name = self.expect_identifier("entry name");
            self.expect(TokenKind::LParen);
            let params = self.parse_params()?;
            self.expect(TokenKind::RParen);

            let return_type = if self.match_token(TokenKind::Arrow) {
                Some(self.parse_type()?)
            } else {
                None
            };

            let entry_end = self.previous_span();
            entries.push(TaskEntry {
                name: entry_name,
                annotations,
                params,
                return_type,
                span: entry_start.merge(entry_end),
            });
            self.skip_newlines();
        }

        let end = self.current_span();
        self.expect(TokenKind::End);

        Ok(TaskDecl {
            name,
            entries,
            is_pub: false,
            span: start.merge(end),
        })
    }

    fn parse_task_body(&mut self) -> Result<TaskBodyDecl, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        self.expect(TokenKind::Task);
        self.expect(TokenKind::Body);
        let name = self.expect_identifier("task name");

        self.expect(TokenKind::LParen);
        let params = self.parse_params()?;
        self.expect(TokenKind::RParen);

        self.expect(TokenKind::FatArrow);
        let body = self.parse_block_body(start)?;

        Ok(TaskBodyDecl {
            name,
            params,
            body,
            span: start.merge(self.previous_span()),
        })
    }

    // -- Protected objects --

    fn parse_protected_decl(
        &mut self,
    ) -> Result<ProtectedBlock, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        self.expect(TokenKind::Protected);
        let name = self.expect_identifier("protected object name");
        self.expect(TokenKind::FatArrow);
        self.skip_newlines();

        let mut fields = Vec::new();
        let mut methods = Vec::new();

        while !self.check(TokenKind::End) && !self.at_end() {
            self.skip_newlines();
            if self.check(TokenKind::End) {
                break;
            }

            if self.check(TokenKind::Var) {
                let f_start = self.current_span();
                self.advance();
                let f_name = self.expect_identifier("field name");
                self.expect(TokenKind::Colon);
                let f_type = self.parse_type()?;
                self.expect(TokenKind::Assign);
                let default = self.parse_expr()?;
                let f_end = self.previous_span();
                fields.push(ProtectedField {
                    name: f_name,
                    type_expr: f_type,
                    default,
                    span: f_start.merge(f_end),
                });
            } else {
                let annotations = self.parse_annotations();
                let mut f = self.parse_function()?;
                f.annotations = annotations;
                methods.push(f);
            }
            self.skip_newlines();
        }

        let end = self.current_span();
        self.expect(TokenKind::End);

        Ok(ProtectedBlock {
            name,
            fields,
            methods,
            is_pub: false,
            span: start.merge(end),
        })
    }

    // -- Units --

    fn parse_unit_decl(&mut self) -> Result<UnitDeclNode, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        self.expect(TokenKind::Unit);
        let name = self.expect_identifier("unit name");
        self.expect(TokenKind::Assign);

        // Read the rest of the line as the unit definition expression
        let def_start = self.pos;
        while !self.check(TokenKind::Newline) && !self.at_end() {
            self.advance();
        }
        let definition = self.tokens[def_start..self.pos]
            .iter()
            .map(|t| t.lexeme.as_str())
            .collect::<Vec<_>>()
            .join(" ");

        let end = self.previous_span();

        Ok(UnitDeclNode {
            name,
            definition,
            span: start.merge(end),
        })
    }

    // -- Extern --

    fn parse_extern_block(&mut self) -> Result<ExternBlock, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        self.expect(TokenKind::Extern);

        let abi = if self.check(TokenKind::StringLiteral) {
            let s = self.current().lexeme.clone();
            self.advance();
            // Strip quotes
            s.trim_matches('"').to_string()
        } else {
            "C".to_string()
        };

        self.expect(TokenKind::FatArrow);
        self.skip_newlines();

        let mut functions = Vec::new();
        while !self.check(TokenKind::End) && !self.at_end() {
            self.skip_newlines();
            if self.check(TokenKind::End) {
                break;
            }
            let f = self.parse_function()?;
            functions.push(f);
            self.skip_newlines();
        }

        let end = self.current_span();
        self.expect(TokenKind::End);

        Ok(ExternBlock {
            abi,
            functions,
            span: start.merge(end),
        })
    }

    // -- Static assert --

    fn parse_static_assert(
        &mut self,
    ) -> Result<StaticAssertNode, crate::diagnostics::Diagnostic> {
        let start = self.current_span();
        self.expect(TokenKind::StaticAssert);
        self.expect(TokenKind::LParen);
        let condition = self.parse_expr()?;

        let message = if self.match_token(TokenKind::Comma) {
            if self.check(TokenKind::StringLiteral) {
                let s = self.current().lexeme.clone();
                self.advance();
                Some(s.trim_matches('"').to_string())
            } else {
                None
            }
        } else {
            None
        };

        self.expect(TokenKind::RParen);
        let end = self.previous_span();

        Ok(StaticAssertNode {
            condition,
            message,
            span: start.merge(end),
        })
    }
}
