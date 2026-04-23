// Abstract Syntax Tree definitions for GalaxC.
// Every syntactic construct in the language has a corresponding AST node.
// All nodes carry source spans for diagnostic reporting.

mod nodes;
mod visit;

pub use nodes::*;
pub use visit::AstVisitor;
