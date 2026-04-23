// GalaxC CLI -- the command-line interface for the compiler toolchain.
// Provides build, check, run, emit-c, emit-ir, fmt, test, init, and version commands.

use clap::{Parser, Subcommand};
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Parser)]
#[command(
    name = "galaxc",
    about = "The GalaxC programming language compiler",
    long_about = "GalaxC: Code that survives the void.\n\n\
                  A statically typed, compiled language for mission-critical software.\n\
                  Compiles .gxc source files to native binaries via C code generation.",
    version = galaxc::VERSION,
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a .gxc source file to a native binary
    Build {
        /// Source file to compile
        file: PathBuf,
        /// Output binary name (defaults to source filename without extension)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// C compiler to use (defaults to cc)
        #[arg(long, default_value = "cc")]
        cc: String,
    },

    /// Type-check a source file without generating code
    Check {
        /// Source file to check
        file: PathBuf,
    },

    /// Compile and immediately run a .gxc source file
    Run {
        /// Source file to run
        file: PathBuf,
    },

    /// Emit generated C code to stdout or a file
    #[command(name = "emit-c")]
    EmitC {
        /// Source file to compile
        file: PathBuf,
        /// Output file (defaults to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Emit the intermediate representation
    #[command(name = "emit-ir")]
    EmitIr {
        /// Source file to compile
        file: PathBuf,
    },

    /// Format a .gxc source file
    Fmt {
        /// Source file to format
        file: PathBuf,
    },

    /// Run tests
    Test {
        /// Test file or directory
        path: Option<PathBuf>,
    },

    /// Initialize a new GalaxC project
    Init {
        /// Project name
        name: String,
    },

    /// Print version information
    Version,
}

fn main() {
    let cli = Cli::parse();

    let exit_code = match cli.command {
        Commands::Build { file, output, cc } => cmd_build(&file, output.as_deref(), &cc),
        Commands::Check { file } => cmd_check(&file),
        Commands::Run { file } => cmd_run(&file),
        Commands::EmitC { file, output } => cmd_emit_c(&file, output.as_deref()),
        Commands::EmitIr { file } => cmd_emit_ir(&file),
        Commands::Fmt { file } => cmd_fmt(&file),
        Commands::Test { path } => cmd_test(path.as_deref()),
        Commands::Init { name } => cmd_init(&name),
        Commands::Version => cmd_version(),
    };

    std::process::exit(exit_code);
}

fn cmd_build(file: &Path, output: Option<&Path>, cc: &str) -> i32 {
    let source = match read_source(file) {
        Some(s) => s,
        None => return 1,
    };

    let filename = file.display().to_string();
    print_phase("Compiling", &filename);

    // Generate C code
    let c_code = match galaxc::compile(&source, &filename) {
        Ok(code) => code,
        Err(errors) => {
            galaxc::diagnostics::render_diagnostics(&errors, &source);
            return 1;
        }
    };

    // Write C to a temporary file
    let c_path = file.with_extension("generated.c");
    if let Err(e) = fs::write(&c_path, &c_code) {
        print_error(&format!("failed to write generated C: {e}"));
        return 1;
    }

    // Determine output binary name
    let out_path = output
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let stem = file.file_stem().unwrap_or_default();
            let mut p = PathBuf::from(stem);
            if cfg!(windows) {
                p.set_extension("exe");
            }
            p
        });

    // Find and invoke a C compiler
    let cc_cmd = find_cc(cc);
    print_phase("Linking", &out_path.display().to_string());

    let status = Command::new(&cc_cmd)
        .arg(&c_path)
        .arg("-o")
        .arg(&out_path)
        .arg("-std=c11")
        .arg("-O2")
        .arg("-lm")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    // Clean up generated C file
    let _ = fs::remove_file(&c_path);

    match status {
        Ok(s) if s.success() => {
            print_success(&format!(
                "Built {}",
                out_path.display()
            ));
            0
        }
        Ok(s) => {
            print_error(&format!(
                "C compiler exited with code {}",
                s.code().unwrap_or(-1)
            ));
            1
        }
        Err(e) => {
            print_error(&format!("failed to run C compiler '{}': {}", cc_cmd, e));
            print_hint("Install gcc, clang, or MSVC and ensure it is on your PATH");
            1
        }
    }
}

fn cmd_check(file: &Path) -> i32 {
    let source = match read_source(file) {
        Some(s) => s,
        None => return 1,
    };

    let filename = file.display().to_string();
    print_phase("Checking", &filename);

    match galaxc::check_only(&source, &filename) {
        Ok(()) => {
            print_success("No errors found");
            0
        }
        Err(errors) => {
            galaxc::diagnostics::render_diagnostics(&errors, &source);
            1
        }
    }
}

fn cmd_run(file: &Path) -> i32 {
    let temp_dir = std::env::temp_dir();
    let stem = file.file_stem().unwrap_or_default().to_string_lossy();
    let out_name = if cfg!(windows) {
        format!("gxc_run_{stem}.exe")
    } else {
        format!("gxc_run_{stem}")
    };
    let out_path = temp_dir.join(&out_name);

    let build_result = cmd_build(file, Some(&out_path), "cc");
    if build_result != 0 {
        return build_result;
    }

    print_phase("Running", &file.display().to_string());
    println!("{}", "---".dimmed());

    let status = Command::new(&out_path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    let _ = fs::remove_file(&out_path);

    match status {
        Ok(s) => s.code().unwrap_or(1),
        Err(e) => {
            print_error(&format!("failed to run program: {e}"));
            1
        }
    }
}

fn cmd_emit_c(file: &Path, output: Option<&Path>) -> i32 {
    let source = match read_source(file) {
        Some(s) => s,
        None => return 1,
    };

    let filename = file.display().to_string();

    match galaxc::compile(&source, &filename) {
        Ok(c_code) => {
            if let Some(out) = output {
                if let Err(e) = fs::write(out, &c_code) {
                    print_error(&format!("failed to write output: {e}"));
                    return 1;
                }
                print_success(&format!("Wrote C to {}", out.display()));
            } else {
                print!("{c_code}");
            }
            0
        }
        Err(errors) => {
            galaxc::diagnostics::render_diagnostics(&errors, &source);
            1
        }
    }
}

fn cmd_emit_ir(file: &Path) -> i32 {
    let source = match read_source(file) {
        Some(s) => s,
        None => return 1,
    };

    let filename = file.display().to_string();

    match galaxc::emit_ir(&source, &filename) {
        Ok(ir_text) => {
            print!("{ir_text}");
            0
        }
        Err(errors) => {
            galaxc::diagnostics::render_diagnostics(&errors, &source);
            1
        }
    }
}

fn cmd_fmt(file: &Path) -> i32 {
    let source = match read_source(file) {
        Some(s) => s,
        None => return 1,
    };

    // Basic formatter: normalize indentation to 4 spaces, trim trailing whitespace,
    // ensure single newline at EOF.
    let mut formatted = String::new();
    let mut indent_level: i32 = 0;

    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            formatted.push('\n');
            continue;
        }

        // Decrease indent before lines that close blocks
        if trimmed == "end" || trimmed.starts_with("end ") {
            indent_level = (indent_level - 1).max(0);
        }
        if trimmed.starts_with("else") {
            indent_level = (indent_level - 1).max(0);
        }

        let pad = "    ".repeat(indent_level as usize);
        formatted.push_str(&pad);
        formatted.push_str(trimmed);
        formatted.push('\n');

        // Increase indent after lines that open blocks
        if trimmed.ends_with("=>") {
            indent_level += 1;
        }
        if trimmed.starts_with("else") && trimmed.ends_with("=>") {
            // already incremented above
        }
    }

    // Ensure single trailing newline
    while formatted.ends_with("\n\n") {
        formatted.pop();
    }

    if let Err(e) = fs::write(file, &formatted) {
        print_error(&format!("failed to write formatted file: {e}"));
        return 1;
    }

    print_success(&format!("Formatted {}", file.display()));
    0
}

fn cmd_test(path: Option<&Path>) -> i32 {
    let target = path.unwrap_or_else(|| Path::new("."));
    print_phase("Testing", &target.display().to_string());

    let files = if target.is_file() {
        vec![target.to_path_buf()]
    } else {
        find_gxc_files(target)
    };

    if files.is_empty() {
        print_hint("No .gxc files found");
        return 0;
    }

    let mut passed = 0;
    let mut failed = 0;

    for file in &files {
        let source = match fs::read_to_string(file) {
            Ok(s) => s,
            Err(e) => {
                print_error(&format!("cannot read {}: {e}", file.display()));
                failed += 1;
                continue;
            }
        };

        let filename = file.display().to_string();
        match galaxc::check_only(&source, &filename) {
            Ok(()) => {
                println!("  {} {}", "PASS".green(), file.display());
                passed += 1;
            }
            Err(errors) => {
                println!("  {} {}", "FAIL".red(), file.display());
                galaxc::diagnostics::render_diagnostics(&errors, &source);
                failed += 1;
            }
        }
    }

    println!();
    println!(
        "{} passed, {} failed, {} total",
        passed.to_string().green(),
        failed.to_string().red(),
        files.len()
    );

    if failed > 0 { 1 } else { 0 }
}

fn cmd_init(name: &str) -> i32 {
    let project_dir = Path::new(name);

    if project_dir.exists() {
        print_error(&format!("directory '{}' already exists", name));
        return 1;
    }

    let src_dir = project_dir.join("src");
    if let Err(e) = fs::create_dir_all(&src_dir) {
        print_error(&format!("failed to create project directory: {e}"));
        return 1;
    }

    let main_content = format!(
        "--! {} -- a GalaxC project\n\n\
         orbit main\n\n\
         @effect(io)\n\
         op launch() =>\n\
         \x20   console.write(\"GalaxC online. All systems nominal.\")\n\
         end\n",
        name
    );

    if let Err(e) = fs::write(src_dir.join("main.gxc"), main_content) {
        print_error(&format!("failed to write main.gxc: {e}"));
        return 1;
    }

    // Write a minimal project config
    let config = format!(
        "[project]\n\
         name = \"{name}\"\n\
         version = \"0.1.0\"\n\
         entry = \"src/main.gxc\"\n"
    );
    if let Err(e) = fs::write(project_dir.join("galaxc.toml"), config) {
        print_error(&format!("failed to write galaxc.toml: {e}"));
        return 1;
    }

    print_success(&format!("Created new GalaxC project '{name}'"));
    0
}

fn cmd_version() -> i32 {
    println!("galaxc {}", galaxc::VERSION);
    println!("GalaxC: Code that survives the void.");
    0
}

// -- Helpers --

fn read_source(path: &Path) -> Option<String> {
    match fs::read_to_string(path) {
        Ok(s) => Some(s),
        Err(e) => {
            print_error(&format!("cannot read '{}': {}", path.display(), e));
            None
        }
    }
}

fn find_cc(preferred: &str) -> String {
    // Try the preferred compiler first
    if which::which(preferred).is_ok() {
        return preferred.to_string();
    }

    // Fall back through common C compilers
    for candidate in &["gcc", "clang", "cc", "cl"] {
        if which::which(candidate).is_ok() {
            return candidate.to_string();
        }
    }

    // Last resort
    preferred.to_string()
}

fn find_gxc_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|e| e == "gxc") {
                files.push(path);
            } else if path.is_dir() {
                files.extend(find_gxc_files(&path));
            }
        }
    }
    files.sort();
    files
}

fn print_phase(phase: &str, target: &str) {
    eprintln!("{:>12} {}", phase.green().bold(), target);
}

fn print_success(message: &str) {
    eprintln!("{:>12} {}", "Success".green().bold(), message);
}

fn print_error(message: &str) {
    eprintln!("{}: {}", "error".red().bold(), message);
}

fn print_hint(message: &str) {
    eprintln!("{}: {}", "hint".cyan().bold(), message);
}
