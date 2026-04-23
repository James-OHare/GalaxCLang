// GalaxC Compiler Library
// Core compilation pipeline: lexing, parsing, type checking, IR, and C code generation.

pub mod lexer;
pub mod ast;
pub mod parser;
pub mod types;
pub mod checker;
pub mod ir;
pub mod codegen;
pub mod diagnostics;

/// Compiler version, injected from Cargo.toml at build time.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Result type used throughout the compiler. Wraps the diagnostics error type.
pub type CompileResult<T> = Result<T, diagnostics::CompileError>;

/// Top-level compilation entry point.
/// Takes source code and a file name, returns generated C code or a list of errors.
pub fn compile(source: &str, filename: &str) -> Result<String, Vec<diagnostics::Diagnostic>> {
    let mut all_diagnostics = Vec::new();

    // Phase 1: Lexing
    let tokens = match lexer::tokenize(source, filename) {
        Ok(tokens) => tokens,
        Err(errors) => {
            all_diagnostics.extend(errors);
            return Err(all_diagnostics);
        }
    };

    // Phase 2: Parsing
    let ast = match parser::parse(tokens, source, filename) {
        Ok(ast) => ast,
        Err(errors) => {
            all_diagnostics.extend(errors);
            return Err(all_diagnostics);
        }
    };

    // Phase 3: Type Checking
    let typed_ast = match checker::check(&ast, filename) {
        Ok(checked) => checked,
        Err(errors) => {
            all_diagnostics.extend(errors);
            return Err(all_diagnostics);
        }
    };

    // Phase 4: IR Generation
    let ir = ir::lower(&typed_ast);

    // Phase 5: C Code Generation
    let c_code = codegen::generate(&ir, filename);

    Ok(c_code)
}

/// Type-check only, without generating code. Used by `galaxc check`.
pub fn check_only(source: &str, filename: &str) -> Result<(), Vec<diagnostics::Diagnostic>> {
    let tokens = match lexer::tokenize(source, filename) {
        Ok(tokens) => tokens,
        Err(errors) => return Err(errors),
    };

    let ast = match parser::parse(tokens, source, filename) {
        Ok(ast) => ast,
        Err(errors) => return Err(errors),
    };

    match checker::check(&ast, filename) {
        Ok(_) => Ok(()),
        Err(errors) => Err(errors),
    }
}

/// Emit the intermediate representation as a human-readable string.
pub fn emit_ir(source: &str, filename: &str) -> Result<String, Vec<diagnostics::Diagnostic>> {
    let tokens = match lexer::tokenize(source, filename) {
        Ok(tokens) => tokens,
        Err(errors) => return Err(errors),
    };

    let ast = match parser::parse(tokens, source, filename) {
        Ok(ast) => ast,
        Err(errors) => return Err(errors),
    };

    let typed_ast = match checker::check(&ast, filename) {
        Ok(checked) => checked,
        Err(errors) => return Err(errors),
    };

    let ir = ir::lower(&typed_ast);
    Ok(ir::display(&ir))
}
