// Parser core -- token management, error recovery, and the top-level
// program parsing entry point.

use crate::lexer::{Token, TokenKind};
use crate::ast::*;
use crate::diagnostics::{Diagnostic, Span};

/// The parser state. Holds the token stream, current position, accumulated
/// errors, and source text for diagnostic rendering.
pub struct Parser {
    pub(super) tokens: Vec<Token>,
    pub(super) pos: usize,
    _source: String,
    filename: String,
    errors: Vec<Diagnostic>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>, source: &str, filename: &str) -> Self {
        // Filter out newline tokens between certain token pairs where they
        // are not meaningful (inside parentheses, brackets, braces). The
        // lexer emits them conservatively; we clean up here.
        let cleaned = tokens;

        Parser {
            tokens: cleaned,
            pos: 0,
            _source: source.to_string(),
            filename: filename.to_string(),
            errors: Vec::new(),
        }
    }

    /// Parse the complete program.
    pub fn parse_program(&mut self) -> Result<Program, Vec<Diagnostic>> {
        let start_span = self.current_span();

        // Module declaration (optional)
        let module_decl = if self.check(TokenKind::Orbit) {
            Some(self.parse_module_decl())
        } else {
            None
        };

        // Imports
        let mut imports = Vec::new();
        while self.check(TokenKind::Dock) {
            self.skip_newlines();
            if self.check(TokenKind::Dock) {
                imports.push(self.parse_import());
            } else {
                break;
            }
        }

        // Top-level items
        let mut items = Vec::new();
        self.skip_newlines();
        while !self.at_end() {
            self.skip_newlines();
            if self.at_end() {
                break;
            }
            match self.parse_item() {
                Ok(item) => items.push(item),
                Err(diag) => {
                    self.errors.push(diag);
                    self.synchronize();
                }
            }
            self.skip_newlines();
        }

        let end_span = self.previous_span();

        if !self.errors.is_empty() {
            return Err(std::mem::take(&mut self.errors));
        }

        Ok(Program {
            module_decl,
            imports,
            items,
            span: start_span.merge(end_span),
        })
    }

    // -- Module and import parsing --

    fn parse_module_decl(&mut self) -> ModuleDecl {
        let start = self.current_span();
        self.expect(TokenKind::Orbit);
        let path = self.parse_module_path();
        self.expect_line_end();
        ModuleDecl {
            span: start.merge(path.span),
            path,
        }
    }

    pub(super) fn parse_import(&mut self) -> ImportDecl {
        let start = self.current_span();
        self.expect(TokenKind::Dock);
        let path = self.parse_module_path();

        // Check for selective import: dock core.collections.{Vec, Map}
        let names = if self.match_token(TokenKind::Dot) && self.check(TokenKind::LBrace) {
            self.advance(); // consume {
            let mut selected = Vec::new();
            loop {
                if self.check(TokenKind::RBrace) {
                    break;
                }
                let name = self.expect_identifier("import name");
                selected.push(name);
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
            self.expect(TokenKind::RBrace);
            ImportNames::Specific(selected)
        } else {
            ImportNames::Module
        };

        // Check for alias: dock mission.telemetry as telem
        let alias = if self.match_token(TokenKind::As) {
            Some(self.expect_identifier("import alias"))
        } else {
            None
        };

        let end = self.previous_span();
        self.expect_line_end();

        ImportDecl {
            path,
            names,
            alias,
            span: start.merge(end),
        }
    }

    pub(super) fn parse_module_path(&mut self) -> ModulePath {
        let start = self.current_span();
        let mut segments = vec![self.expect_identifier("module name")];

        while self.match_token(TokenKind::Dot) {
            // Check if next is an identifier (not { for selective import)
            if self.check(TokenKind::Identifier) {
                segments.push(self.expect_identifier("module segment"));
            } else {
                // Put the dot back conceptually -- the caller will handle {
                self.pos -= 1;
                break;
            }
        }

        let end = self.previous_span();
        ModulePath {
            segments,
            span: start.merge(end),
        }
    }

    // -- Token management --

    pub(super) fn current(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or_else(|| {
            self.tokens.last().expect("token stream must contain at least EOF")
        })
    }

    pub(super) fn current_kind(&self) -> TokenKind {
        self.current().kind
    }

    pub(super) fn current_span(&self) -> Span {
        self.current().span
    }

    pub(super) fn previous_span(&self) -> Span {
        if self.pos > 0 {
            self.tokens[self.pos - 1].span
        } else {
            Span::point(0)
        }
    }

    pub(super) fn advance(&mut self) -> &Token {
        if !self.at_end() {
            self.pos += 1;
        }
        &self.tokens[self.pos - 1]
    }

    pub(super) fn at_end(&self) -> bool {
        self.current_kind() == TokenKind::Eof
    }

    pub(super) fn check(&self, kind: TokenKind) -> bool {
        self.current_kind() == kind
    }

    pub(super) fn match_token(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    pub(super) fn expect(&mut self, kind: TokenKind) -> Span {
        if self.check(kind) {
            let span = self.current_span();
            self.advance();
            span
        } else {
            let span = self.current_span();
            self.errors.push(
                Diagnostic::error(format!("expected {kind}, found {}", self.current_kind()))
                    .with_span(span)
                    .with_file(&self.filename),
            );
            span
        }
    }

    pub(super) fn expect_identifier(&mut self, context: &str) -> String {
        if self.check(TokenKind::Identifier) {
            let name = self.current().lexeme.clone();
            self.advance();
            name
        } else {
            let span = self.current_span();
            self.errors.push(
                Diagnostic::error(format!(
                    "expected identifier for {context}, found {}",
                    self.current_kind()
                ))
                .with_span(span)
                .with_file(&self.filename),
            );
            "<error>".to_string()
        }
    }

    pub(super) fn skip_newlines(&mut self) {
        while self.check(TokenKind::Newline) {
            self.advance();
        }
    }

    pub(super) fn expect_line_end(&mut self) {
        if self.check(TokenKind::Newline) || self.check(TokenKind::Eof) {
            if self.check(TokenKind::Newline) {
                self.advance();
            }
        }
        // Don't error if the line doesn't end with a newline in some contexts
    }

    /// Error recovery: skip tokens until we find something that looks like
    /// the start of a new statement or declaration.
    pub(super) fn synchronize(&mut self) {
        while !self.at_end() {
            if self.current_kind() == TokenKind::Newline {
                self.advance();
                if self.current_kind().is_statement_start() {
                    return;
                }
                continue;
            }
            if self.current_kind().is_statement_start() {
                return;
            }
            self.advance();
        }
    }

    pub(super) fn error(&self, message: impl Into<String>) -> Diagnostic {
        Diagnostic::error(message)
            .with_span(self.current_span())
            .with_file(&self.filename)
    }

    /// Peek ahead by n tokens without consuming.
    pub(super) fn peek_ahead(&self, n: usize) -> TokenKind {
        self.tokens
            .get(self.pos + n)
            .map(|t| t.kind)
            .unwrap_or(TokenKind::Eof)
    }

// previous_lexeme removed

}
