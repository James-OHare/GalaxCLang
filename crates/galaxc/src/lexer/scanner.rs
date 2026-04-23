// Scanner -- the character-level engine that produces tokens from source text.
// Handles whitespace, comments, string/char literals, numbers, identifiers,
// keywords, and all operators/punctuation. Reports precise error locations
// for malformed input.

use super::token::{Token, TokenKind};
use crate::diagnostics::{Diagnostic, Span};

pub struct Scanner<'src> {
    source: &'src str,
    bytes: &'src [u8],
    filename: String,
    pos: usize,
    /// Track whether the previous meaningful token could end a statement,
    /// so we know when a newline acts as a statement separator.
    prev_can_end_statement: bool,
}

impl<'src> Scanner<'src> {
    pub fn new(source: &'src str, filename: &str) -> Self {
        Scanner {
            source,
            bytes: source.as_bytes(),
            filename: filename.to_string(),
            pos: 0,
            prev_can_end_statement: false,
        }
    }

    /// Produce the next token from the source. Returns Err for unrecoverable
    /// lexer errors (the scanner still advances past the bad character).
    pub fn next_token(&mut self) -> Result<Token, Diagnostic> {
        self.skip_whitespace_and_comments()?;

        if self.at_end() {
            return Ok(Token::new(
                TokenKind::Eof,
                Span::point(self.pos),
                "",
            ));
        }

        let start = self.pos;
        let ch = self.advance();

        let kind = match ch {
            // Newlines -- only emit if the previous token could end a statement
            b'\n' => {
                if self.prev_can_end_statement {
                    self.prev_can_end_statement = false;
                    return Ok(Token::new(
                        TokenKind::Newline,
                        Span::new(start, self.pos),
                        "\n",
                    ));
                }
                // Otherwise skip and get next token
                return self.next_token();
            }

            b'\r' => {
                // Skip CR, handle CRLF
                if self.peek() == Some(b'\n') {
                    self.advance();
                }
                if self.prev_can_end_statement {
                    self.prev_can_end_statement = false;
                    return Ok(Token::new(
                        TokenKind::Newline,
                        Span::new(start, self.pos),
                        "\n",
                    ));
                }
                return self.next_token();
            }

            // Single-character tokens
            b'(' => TokenKind::LParen,
            b')' => {
                self.prev_can_end_statement = true;
                return Ok(Token::new(
                    TokenKind::RParen,
                    Span::new(start, self.pos),
                    ")",
                ));
            }
            b'[' => TokenKind::LBracket,
            b']' => {
                self.prev_can_end_statement = true;
                return Ok(Token::new(
                    TokenKind::RBracket,
                    Span::new(start, self.pos),
                    "]",
                ));
            }
            b'{' => TokenKind::LBrace,
            b'}' => {
                self.prev_can_end_statement = true;
                return Ok(Token::new(
                    TokenKind::RBrace,
                    Span::new(start, self.pos),
                    "}",
                ));
            }
            b',' => TokenKind::Comma,
            b';' => TokenKind::Semicolon,
            b'~' => TokenKind::Tilde,
            b'@' => TokenKind::At,
            b'?' => {
                self.prev_can_end_statement = true;
                return Ok(Token::new(
                    TokenKind::Question,
                    Span::new(start, self.pos),
                    "?",
                ));
            }

            // Two-character operators and ambiguous single characters
            b'+' => {
                if self.match_char(b'+') {
                    TokenKind::PlusPlus
                } else if self.match_char(b'=') {
                    TokenKind::PlusAssign
                } else {
                    TokenKind::Plus
                }
            }

            b'-' => {
                if self.match_char(b'>') {
                    TokenKind::Arrow
                } else if self.match_char(b'=') {
                    TokenKind::MinusAssign
                } else {
                    TokenKind::Minus
                }
            }

            b'*' => {
                if self.match_char(b'=') {
                    TokenKind::StarAssign
                } else {
                    TokenKind::Star
                }
            }

            b'/' => {
                if self.match_char(b'=') {
                    TokenKind::SlashAssign
                } else {
                    TokenKind::Slash
                }
            }

            b'%' => {
                if self.match_char(b'=') {
                    TokenKind::PercentAssign
                } else {
                    TokenKind::Percent
                }
            }

            b'=' => {
                if self.match_char(b'=') {
                    TokenKind::Eq
                } else if self.match_char(b'>') {
                    TokenKind::FatArrow
                } else {
                    TokenKind::Assign
                }
            }

            b'!' => {
                if self.match_char(b'=') {
                    TokenKind::NotEq
                } else if self.match_char(b'!') {
                    TokenKind::BangBang
                } else {
                    return Err(Diagnostic::error("unexpected character '!'")
                        .with_span(Span::new(start, self.pos))
                        .with_file(&self.filename)
                        .with_help("did you mean '!=' or '!!'?"));
                }
            }

            b'<' => {
                if self.match_char(b'=') {
                    TokenKind::LtEq
                } else if self.match_char(b'<') {
                    TokenKind::ShiftLeft
                } else {
                    TokenKind::Lt
                }
            }

            b'>' => {
                if self.match_char(b'=') {
                    TokenKind::GtEq
                } else if self.match_char(b'>') {
                    TokenKind::ShiftRight
                } else {
                    TokenKind::Gt
                }
            }

            b'&' => TokenKind::Ampersand,
            b'|' => TokenKind::Pipe,
            b'^' => TokenKind::Caret,

            b':' => {
                if self.match_char(b':') {
                    TokenKind::ColonColon
                } else {
                    TokenKind::Colon
                }
            }

            b'.' => {
                if self.match_char(b'.') {
                    TokenKind::DotDot
                } else {
                    TokenKind::Dot
                }
            }

            // String literals
            b'"' => return self.scan_string(start),

            // Character literals
            b'\'' => return self.scan_char(start),

            // Numbers
            b'0'..=b'9' => return self.scan_number(start),

            // Identifiers and keywords
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => return self.scan_identifier(start),

            other => {
                return Err(Diagnostic::error(format!(
                    "unexpected character '{}'",
                    other as char
                ))
                .with_span(Span::new(start, self.pos))
                .with_file(&self.filename));
            }
        };

        let lexeme = &self.source[start..self.pos];
        let token = Token::new(kind, Span::new(start, self.pos), lexeme);

        // Track whether this token can end a statement
        self.prev_can_end_statement = matches!(
            kind,
            TokenKind::Identifier
                | TokenKind::IntLiteral
                | TokenKind::FloatLiteral
                | TokenKind::StringLiteral
                | TokenKind::CharLiteral
                | TokenKind::True
                | TokenKind::False
                | TokenKind::None_
                | TokenKind::End
                | TokenKind::Break
                | TokenKind::Continue
                | TokenKind::RParen
                | TokenKind::RBracket
                | TokenKind::RBrace
                | TokenKind::Question
                | TokenKind::SelfLower
                | TokenKind::SelfUpper
                | TokenKind::FatArrow
        );

        Ok(token)
    }

    // -- Scanning helpers --

    fn scan_string(&mut self, start: usize) -> Result<Token, Diagnostic> {
        let mut value = String::new();

        while !self.at_end() {
            match self.peek() {
                Some(b'"') => {
                    self.advance();
                    self.prev_can_end_statement = true;
                    let lexeme = &self.source[start..self.pos];
                    return Ok(Token::new(
                        TokenKind::StringLiteral,
                        Span::new(start, self.pos),
                        lexeme,
                    ));
                }
                Some(b'\\') => {
                    self.advance();
                    match self.peek() {
                        Some(b'n') => { self.advance(); value.push('\n'); }
                        Some(b't') => { self.advance(); value.push('\t'); }
                        Some(b'r') => { self.advance(); value.push('\r'); }
                        Some(b'\\') => { self.advance(); value.push('\\'); }
                        Some(b'"') => { self.advance(); value.push('"'); }
                        Some(b'0') => { self.advance(); value.push('\0'); }
                        Some(c) => {
                            let esc_pos = self.pos;
                            self.advance();
                            return Err(Diagnostic::error(format!(
                                "unknown escape sequence '\\{}'",
                                c as char
                            ))
                            .with_span(Span::new(esc_pos - 1, self.pos))
                            .with_file(&self.filename));
                        }
                        None => {
                            return Err(Diagnostic::error("unterminated escape sequence")
                                .with_span(Span::new(self.pos - 1, self.pos))
                                .with_file(&self.filename));
                        }
                    }
                }
                Some(b'\n') => {
                    return Err(Diagnostic::error("unterminated string literal")
                        .with_span(Span::new(start, self.pos))
                        .with_file(&self.filename)
                        .with_help("string literals cannot span multiple lines"));
                }
                Some(c) => {
                    self.advance();
                    value.push(c as char);
                }
                None => break,
            }
        }

        Err(Diagnostic::error("unterminated string literal")
            .with_span(Span::new(start, self.pos))
            .with_file(&self.filename))
    }

    fn scan_char(&mut self, start: usize) -> Result<Token, Diagnostic> {
        if self.at_end() {
            return Err(Diagnostic::error("unterminated character literal")
                .with_span(Span::new(start, self.pos))
                .with_file(&self.filename));
        }

        let ch = self.advance();
        let _value = if ch == b'\\' {
            if self.at_end() {
                return Err(Diagnostic::error("unterminated escape in character literal")
                    .with_span(Span::new(start, self.pos))
                    .with_file(&self.filename));
            }
            let esc = self.advance();
            match esc {
                b'n' => '\n',
                b't' => '\t',
                b'r' => '\r',
                b'\\' => '\\',
                b'\'' => '\'',
                b'0' => '\0',
                _ => {
                    return Err(Diagnostic::error(format!(
                        "unknown escape sequence '\\{}'",
                        esc as char
                    ))
                    .with_span(Span::new(start, self.pos))
                    .with_file(&self.filename));
                }
            }
        } else {
            ch as char
        };

        if self.peek() != Some(b'\'') {
            return Err(Diagnostic::error("unterminated character literal")
                .with_span(Span::new(start, self.pos))
                .with_file(&self.filename)
                .with_help("character literals must contain exactly one character"));
        }
        self.advance(); // closing quote

        self.prev_can_end_statement = true;
        let lexeme = &self.source[start..self.pos];
        Ok(Token::new(
            TokenKind::CharLiteral,
            Span::new(start, self.pos),
            lexeme,
        ))
    }

    fn scan_number(&mut self, start: usize) -> Result<Token, Diagnostic> {
        let mut is_float = false;

        // Check for hex, octal, binary prefixes
        if self.source.as_bytes().get(start) == Some(&b'0') {
            match self.peek() {
                Some(b'x') | Some(b'X') => {
                    self.advance();
                    while let Some(c) = self.peek() {
                        if c.is_ascii_hexdigit() || c == b'_' {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.prev_can_end_statement = true;
                    let lexeme = &self.source[start..self.pos];
                    return Ok(Token::new(
                        TokenKind::IntLiteral,
                        Span::new(start, self.pos),
                        lexeme,
                    ));
                }
                Some(b'b') | Some(b'B') => {
                    self.advance();
                    while let Some(c) = self.peek() {
                        if c == b'0' || c == b'1' || c == b'_' {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.prev_can_end_statement = true;
                    let lexeme = &self.source[start..self.pos];
                    return Ok(Token::new(
                        TokenKind::IntLiteral,
                        Span::new(start, self.pos),
                        lexeme,
                    ));
                }
                Some(b'o') | Some(b'O') => {
                    self.advance();
                    while let Some(c) = self.peek() {
                        if (b'0'..=b'7').contains(&c) || c == b'_' {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    self.prev_can_end_statement = true;
                    let lexeme = &self.source[start..self.pos];
                    return Ok(Token::new(
                        TokenKind::IntLiteral,
                        Span::new(start, self.pos),
                        lexeme,
                    ));
                }
                _ => {}
            }
        }

        // Decimal digits
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == b'_' {
                self.advance();
            } else {
                break;
            }
        }

        // Fractional part
        if self.peek() == Some(b'.') && self.peek_at(1).is_some_and(|c| c.is_ascii_digit()) {
            is_float = true;
            self.advance(); // consume '.'
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() || c == b'_' {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        // Exponent
        if let Some(b'e') | Some(b'E') = self.peek() {
            is_float = true;
            self.advance();
            if let Some(b'+') | Some(b'-') = self.peek() {
                self.advance();
            }
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() || c == b'_' {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        self.prev_can_end_statement = true;
        let lexeme = &self.source[start..self.pos];
        let kind = if is_float {
            TokenKind::FloatLiteral
        } else {
            TokenKind::IntLiteral
        };
        Ok(Token::new(kind, Span::new(start, self.pos), lexeme))
    }

    fn scan_identifier(&mut self, start: usize) -> Result<Token, Diagnostic> {
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == b'_' {
                self.advance();
            } else {
                break;
            }
        }

        let text = &self.source[start..self.pos];
        let kind = TokenKind::keyword(text).unwrap_or(TokenKind::Identifier);

        self.prev_can_end_statement = matches!(
            kind,
            TokenKind::Identifier
                | TokenKind::True
                | TokenKind::False
                | TokenKind::None_
                | TokenKind::End
                | TokenKind::Break
                | TokenKind::Continue
                | TokenKind::SelfLower
                | TokenKind::SelfUpper
        );

        Ok(Token::new(kind, Span::new(start, self.pos), text))
    }

    /// Skip whitespace (except newlines, which are statement separators)
    /// and comments.
    fn skip_whitespace_and_comments(&mut self) -> Result<(), Diagnostic> {
        loop {
            // Skip spaces and tabs
            while let Some(c) = self.peek() {
                if c == b' ' || c == b'\t' {
                    self.advance();
                } else {
                    break;
                }
            }

            // Check for comments
            if self.remaining() >= 3 && &self.source[self.pos..self.pos + 3] == "---" {
                // Doc comment -- preserve as a token
                let _start = self.pos;
                while !self.at_end() && self.peek() != Some(b'\n') {
                    self.advance();
                }
                // We treat doc comments as meaningful tokens, so put them back.
                // Actually, we will handle this in next_token by checking for ---
                // before entering skip_whitespace_and_comments. For now, skip them
                // and let the parser handle doc comments if needed.
                // (Doc comments are captured as DocComment tokens in the main loop
                // when we detect the --- prefix after comment skipping.)
                continue;
            }

            if self.remaining() >= 3 && &self.source[self.pos..self.pos + 3] == "--!" {
                // Module doc comment -- skip to end of line
                while !self.at_end() && self.peek() != Some(b'\n') {
                    self.advance();
                }
                continue;
            }

            if self.remaining() >= 2 && &self.source[self.pos..self.pos + 2] == "--" {
                // Line comment -- skip to end of line
                while !self.at_end() && self.peek() != Some(b'\n') {
                    self.advance();
                }
                continue;
            }

            break;
        }
        Ok(())
    }

    // -- Low-level character access --

    fn at_end(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    fn remaining(&self) -> usize {
        self.bytes.len() - self.pos
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<u8> {
        self.bytes.get(self.pos + offset).copied()
    }

    fn advance(&mut self) -> u8 {
        let ch = self.bytes[self.pos];
        self.pos += 1;
        ch
    }

    fn match_char(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize(src: &str) -> Vec<Token> {
        let mut scanner = Scanner::new(src, "test.gxc");
        let mut tokens = Vec::new();
        loop {
            let tok = scanner.next_token().expect("unexpected lexer error");
            let is_eof = tok.kind == TokenKind::Eof;
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        tokens
    }

    fn kinds(src: &str) -> Vec<TokenKind> {
        tokenize(src).into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn simple_keywords() {
        assert_eq!(
            kinds("op let var const end"),
            vec![
                TokenKind::Op,
                TokenKind::Let,
                TokenKind::Var,
                TokenKind::Const,
                TokenKind::End,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn operators() {
        assert_eq!(
            kinds("+ - * / == != => -> :: .."),
            vec![
                TokenKind::Plus,
                TokenKind::Minus,
                TokenKind::Star,
                TokenKind::Slash,
                TokenKind::Eq,
                TokenKind::NotEq,
                TokenKind::FatArrow,
                TokenKind::Arrow,
                TokenKind::ColonColon,
                TokenKind::DotDot,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn string_literal() {
        let tokens = tokenize("\"hello world\"");
        assert_eq!(tokens[0].kind, TokenKind::StringLiteral);
        assert_eq!(tokens[0].lexeme, "\"hello world\"");
    }

    #[test]
    fn integer_literals() {
        assert_eq!(kinds("42 0xFF 0b1010 1_000_000"), vec![
            TokenKind::IntLiteral,
            TokenKind::IntLiteral,
            TokenKind::IntLiteral,
            TokenKind::IntLiteral,
            TokenKind::Eof,
        ]);
    }

    #[test]
    fn float_literal() {
        assert_eq!(kinds("3.14 1.0e10 2.5E-3"), vec![
            TokenKind::FloatLiteral,
            TokenKind::FloatLiteral,
            TokenKind::FloatLiteral,
            TokenKind::Eof,
        ]);
    }

    #[test]
    fn comments_skipped() {
        assert_eq!(
            kinds("let x = 42 -- this is a comment"),
            vec![
                TokenKind::Let,
                TokenKind::Identifier,
                TokenKind::Assign,
                TokenKind::IntLiteral,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn fat_arrow_block() {
        assert_eq!(
            kinds("op foo() =>\n    return 42\nend"),
            vec![
                TokenKind::Op,
                TokenKind::Identifier,
                TokenKind::LParen,
                TokenKind::RParen,
                TokenKind::FatArrow,
                TokenKind::Newline,
                TokenKind::Return,
                TokenKind::IntLiteral,
                TokenKind::Newline,
                TokenKind::End,
                TokenKind::Eof,
            ]
        );
    }
}
