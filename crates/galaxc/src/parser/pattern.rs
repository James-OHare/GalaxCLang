// Pattern parsing for match arms, let destructuring, and related contexts.

use super::core::Parser;
use crate::lexer::TokenKind;
use crate::ast::*;

impl Parser {
    /// Parse a pattern (used in match arms and destructuring).
    pub(super) fn parse_pattern(&mut self) -> Result<Pattern, crate::diagnostics::Diagnostic> {
        match self.current_kind() {
            // Wildcard
            TokenKind::Identifier if self.current().lexeme == "_" => {
                let span = self.current_span();
                self.advance();
                Ok(Pattern::Wildcard { span })
            }

            // Literal patterns
            TokenKind::IntLiteral => {
                let span = self.current_span();
                let text = self.current().lexeme.clone();
                self.advance();
                let value = text.replace('_', "").parse::<i64>().unwrap_or(0);
                Ok(Pattern::Literal {
                    value: LiteralValue::Int(value),
                    span,
                })
            }

            TokenKind::FloatLiteral => {
                let span = self.current_span();
                let text = self.current().lexeme.clone();
                self.advance();
                let value = text.replace('_', "").parse::<f64>().unwrap_or(0.0);
                Ok(Pattern::Literal {
                    value: LiteralValue::Float(value),
                    span,
                })
            }

            TokenKind::StringLiteral => {
                let span = self.current_span();
                let raw = self.current().lexeme.clone();
                self.advance();
                let s = raw[1..raw.len() - 1].to_string();
                Ok(Pattern::Literal {
                    value: LiteralValue::String(s),
                    span,
                })
            }

            TokenKind::True => {
                let span = self.current_span();
                self.advance();
                Ok(Pattern::Literal {
                    value: LiteralValue::Bool(true),
                    span,
                })
            }

            TokenKind::False => {
                let span = self.current_span();
                self.advance();
                Ok(Pattern::Literal {
                    value: LiteralValue::Bool(false),
                    span,
                })
            }

            TokenKind::None_ => {
                let span = self.current_span();
                self.advance();
                Ok(Pattern::Literal {
                    value: LiteralValue::None,
                    span,
                })
            }

            // Tuple pattern
            TokenKind::LParen => {
                let start = self.current_span();
                self.advance();
                let mut elements = Vec::new();
                while !self.check(TokenKind::RParen) && !self.at_end() {
                    elements.push(self.parse_pattern()?);
                    if !self.match_token(TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RParen);
                Ok(Pattern::Tuple {
                    elements,
                    span: start.merge(self.previous_span()),
                })
            }

            // Identifier -- could be a simple binding or a qualified variant path
            TokenKind::Identifier => {
                let start = self.current_span();
                let name = self.expect_identifier("pattern");

                // Check for qualified path: Type::Variant(fields)
                if self.check(TokenKind::ColonColon) {
                    let mut path = vec![name];
                    while self.match_token(TokenKind::ColonColon) {
                        path.push(self.expect_identifier("variant name"));
                    }

                    // Check for fields
                    let fields = if self.match_token(TokenKind::LParen) {
                        self.parse_pattern_fields()?
                    } else {
                        Vec::new()
                    };

                    let end = self.previous_span();
                    Ok(Pattern::Variant {
                        path,
                        fields,
                        span: start.merge(end),
                    })
                } else if self.check(TokenKind::LParen)
                    && name.starts_with(|c: char| c.is_uppercase())
                {
                    // Variant without path prefix: Variant(fields)
                    self.advance();
                    let fields = self.parse_pattern_fields()?;
                    let end = self.previous_span();
                    Ok(Pattern::Variant {
                        path: vec![name],
                        fields,
                        span: start.merge(end),
                    })
                } else {
                    // Simple binding
                    Ok(Pattern::Binding { name, span: start })
                }
            }

            // Result/Option variant keywords
            TokenKind::Ok_ | TokenKind::Err_ | TokenKind::Some_ => {
                let start = self.current_span();
                let name = self.current().lexeme.clone();
                self.advance();
                self.expect(TokenKind::LParen);
                let fields = self.parse_pattern_fields()?;
                let end = self.previous_span();
                Ok(Pattern::Variant {
                    path: vec!["Result".to_string(), name],
                    fields,
                    span: start.merge(end),
                })
            }

            _ => Err(self.error(format!(
                "expected pattern, found {}",
                self.current_kind()
            ))),
        }
    }

    fn parse_pattern_fields(
        &mut self,
    ) -> Result<Vec<PatternField>, crate::diagnostics::Diagnostic> {
        let mut fields = Vec::new();

        while !self.check(TokenKind::RParen) && !self.at_end() {
            let f_start = self.current_span();
            let pattern = self.parse_pattern()?;
            let f_end = self.previous_span();

            fields.push(PatternField {
                name: None,
                pattern,
                span: f_start.merge(f_end),
            });

            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }

        self.expect(TokenKind::RParen);
        Ok(fields)
    }
}
