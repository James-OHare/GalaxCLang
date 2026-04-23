// Diagnostics module -- error reporting and source location tracking.
// Provides structured error types and pretty-printed diagnostic output.

mod span;
mod report;

pub use span::{Span, SourceLocation};
pub use report::{Diagnostic, DiagnosticKind, render_diagnostics};

// thiserror import removed

/// Top-level compiler error type. Wraps one or more diagnostics.
#[derive(Debug)]
pub struct CompileError(pub Vec<Diagnostic>);

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "compilation failed with {} error(s)", self.0.len())
    }
}

impl std::error::Error for CompileError {}

impl CompileError {
    pub fn single(diag: Diagnostic) -> Self {
        CompileError(vec![diag])
    }

    pub fn many(diags: Vec<Diagnostic>) -> Self {
        CompileError(diags)
    }
}
