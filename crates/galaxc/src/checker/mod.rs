// Type checker module. Walks the AST, resolves types, checks consistency,
// verifies exhaustive pattern matching, and produces a typed AST.

mod env;
mod check;

pub use check::{check, TypeChecker};
