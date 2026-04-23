// Lexer module -- tokenizes GalaxC source code into a stream of tokens.
// Hand-written for maximum performance and error-message quality.

mod token;
mod scanner;

pub use token::{Token, TokenKind};
pub use scanner::Scanner;

use crate::diagnostics::Diagnostic;

/// Tokenize a complete source string. Returns all tokens (including EOF)
/// or a list of lexer-level diagnostics on failure.
pub fn tokenize(source: &str, filename: &str) -> Result<Vec<Token>, Vec<Diagnostic>> {
    let mut scanner = Scanner::new(source, filename);
    let mut tokens = Vec::new();
    let mut errors = Vec::new();

    loop {
        match scanner.next_token() {
            Ok(token) => {
                let is_eof = token.kind == TokenKind::Eof;
                tokens.push(token);
                if is_eof {
                    break;
                }
            }
            Err(diag) => {
                errors.push(diag);
                // Continue scanning to report multiple errors
            }
        }
    }

    if errors.is_empty() {
        Ok(tokens)
    } else {
        Err(errors)
    }
}
