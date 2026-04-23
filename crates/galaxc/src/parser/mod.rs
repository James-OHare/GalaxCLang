// Parser module -- converts a token stream into an abstract syntax tree.
// Uses recursive descent for declarations and statements, and Pratt parsing
// (operator precedence climbing) for expressions.

mod core;
mod decl;
mod stmt;
mod expr;
mod types;
mod pattern;

pub use self::core::Parser;

use crate::lexer::Token;
use crate::ast::Program;
use crate::diagnostics::Diagnostic;

/// Parse a token stream into a complete program AST.
pub fn parse(tokens: Vec<Token>, source: &str, filename: &str) -> Result<Program, Vec<Diagnostic>> {
    let mut parser = Parser::new(tokens, source, filename);
    parser.parse_program()
}
