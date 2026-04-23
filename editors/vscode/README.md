# GalaxC for Visual Studio Code

This extension provides comprehensive language support for the GalaxC programming language, a mission-critical systems language designed for high-reliability software in space exploration and robotics.

## Features

### Syntax Highlighting
Full support for GalaxC grammar, using highly granular TextMate scopes.
- Keyword and operator recognition including mission-critical annotations.
- Unit-safe numeric type highlighting (e.g., <meters>, <newtons>).
- Granular differentiation between primitive types, custom types, and built-ins.
- Support for documentation comments (---) and standard comments (--).

### Diagnostics (Real-time Error Detection)
Integrated support for the GalaxC compiler diagnostic engine.
- Automatic checking on document save and open.
- Errors are displayed directly in the Problems view and via editor squiggles.
- Requires the 'galaxc' CLI to be installed and available in the system path.

### Intellisense and Snippets
Comprehensive code snippets for language constructs:
- Function definitions (op)
- Data structures (struct, enum)
- Control flow (match, if, for, while)
- Tasking and concurrency patterns (task, select, accept)
- Standard library utilities (console.write)

### Outline Support
The Outline view provides a structured representation of the current file, allowing for rapid navigation between operations, structs, tasks, and enums.

### Language Configuration
- Automatic indentation for block-based syntax.
- Smart auto-closing for brackets and quotes.
- Bracket matching and highlighting.
- Code folding for regions opened with '=>' and closed with 'end'.

## Installation

### From Source
1. Ensure Visual Studio Code is installed.
2. Link or copy this directory to your VS Code extensions folder:
   - Windows: %USERPROFILE%\.vscode\extensions\galaxc-lang.galaxc
   - macOS/Linux: ~/.vscode/extensions/galaxc-lang.galaxc
3. Restart Visual Studio Code.

### Requirements
To utilize real-time diagnostics, the GalaxC compiler toolchain must be installed on your system.
```powershell
# From the GalaxC repository root
cargo install --path crates/galaxc-cli
```

## Extension Settings

This extension currently relies on the system PATH to locate the 'galaxc' binary for diagnostic checking. Ensure the directory containing 'galaxc' (typically ~/.cargo/bin) is in your environment PATH.

## Technical Details

- Publisher ID: galaxc-lang
- Extension ID: galaxc-vscode
- Scope Name: source.galaxc

For more information on the GalaxC language, refer to the main repository documentation.
