// Diagnostic reporting -- structured error/warning messages with source context.
// Renders colored, underlined source snippets pointing at the problem location.

use super::span::{Span, SourceLocation};
use colored::Colorize;

/// Severity level of a diagnostic message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticKind {
    Error,
    Warning,
    Note,
    Help,
}

impl DiagnosticKind {
    fn label(&self) -> &'static str {
        match self {
            DiagnosticKind::Error => "error",
            DiagnosticKind::Warning => "warning",
            DiagnosticKind::Note => "note",
            DiagnosticKind::Help => "help",
        }
    }
}

/// A single diagnostic message with optional source location and sub-notes.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub kind: DiagnosticKind,
    pub message: String,
    pub span: Option<Span>,
    pub filename: Option<String>,
    pub notes: Vec<String>,
    pub help: Option<String>,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>) -> Self {
        Diagnostic {
            kind: DiagnosticKind::Error,
            message: message.into(),
            span: None,
            filename: None,
            notes: Vec::new(),
            help: None,
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Diagnostic {
            kind: DiagnosticKind::Warning,
            message: message.into(),
            span: None,
            filename: None,
            notes: Vec::new(),
            help: None,
        }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    pub fn with_file(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Check whether this is an error-level diagnostic.
    pub fn is_error(&self) -> bool {
        self.kind == DiagnosticKind::Error
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind.label(), self.message)
    }
}

/// Render a list of diagnostics to stderr with colored output and source context.
pub fn render_diagnostics(diagnostics: &[Diagnostic], source: &str) {
    for diag in diagnostics {
        render_single(diag, source);
    }

    // Summary line
    let error_count = diagnostics.iter().filter(|d| d.is_error()).count();
    let warning_count = diagnostics
        .iter()
        .filter(|d| d.kind == DiagnosticKind::Warning)
        .count();

    if error_count > 0 || warning_count > 0 {
        let mut parts = Vec::new();
        if error_count > 0 {
            parts.push(format!(
                "{} error{}",
                error_count,
                if error_count == 1 { "" } else { "s" }
            ));
        }
        if warning_count > 0 {
            parts.push(format!(
                "{} warning{}",
                warning_count,
                if warning_count == 1 { "" } else { "s" }
            ));
        }
        eprintln!("{}", parts.join(", "));
    }
}

/// Render a single diagnostic to stderr.
fn render_single(diag: &Diagnostic, source: &str) {
    // Header: "error: message" or "warning: message"
    let header = match diag.kind {
        DiagnosticKind::Error => format!("{}: {}", "error".red().bold(), diag.message.bold()),
        DiagnosticKind::Warning => {
            format!("{}: {}", "warning".yellow().bold(), diag.message.bold())
        }
        DiagnosticKind::Note => format!("{}: {}", "note".blue().bold(), diag.message),
        DiagnosticKind::Help => format!("{}: {}", "help".green().bold(), diag.message),
    };
    eprintln!("{header}");

    // Source context with underline
    if let Some(span) = diag.span {
        let filename = diag.filename.as_deref().unwrap_or("<input>");
        let loc = SourceLocation::from_offset(source, span.start, filename);
        let end_loc = SourceLocation::from_offset(source, span.end, filename);

        eprintln!(
            "  {} {}",
            "-->".blue().bold(),
            format!("{filename}:{}:{}", loc.line, loc.column).dimmed()
        );

        // Show the source lines involved
        let lines: Vec<&str> = source.lines().collect();
        let start_line = loc.line.saturating_sub(1);
        let end_line = end_loc.line.min(lines.len());
        let gutter_width = format!("{end_line}").len();

        // Blank gutter line
        eprintln!("{:>gutter_width$} {}", "", "|".blue().bold());

        for line_num in start_line..end_line {
            let line_content = lines.get(line_num).unwrap_or(&"");
            let display_num = line_num + 1;

            eprintln!(
                "{:>gutter_width$} {} {}",
                display_num.to_string().blue().bold(),
                "|".blue().bold(),
                line_content
            );

            // Underline the relevant portion on this line
            if display_num == loc.line && display_num == end_loc.line {
                // Single-line span
                let underline_start = loc.column - 1;
                let underline_len = (end_loc.column - loc.column).max(1);
                let padding = " ".repeat(underline_start);
                let underline = "^".repeat(underline_len);
                eprintln!(
                    "{:>gutter_width$} {} {}{}",
                    "",
                    "|".blue().bold(),
                    padding,
                    underline.red().bold()
                );
            } else if display_num == loc.line {
                // Start of multi-line span
                let underline_start = loc.column - 1;
                let underline_len = line_content.len().saturating_sub(underline_start).max(1);
                let padding = " ".repeat(underline_start);
                let underline = "^".repeat(underline_len);
                eprintln!(
                    "{:>gutter_width$} {} {}{}",
                    "",
                    "|".blue().bold(),
                    padding,
                    underline.red().bold()
                );
            } else if display_num == end_loc.line {
                // End of multi-line span
                let underline_len = end_loc.column.saturating_sub(1).max(1);
                let underline = "^".repeat(underline_len);
                eprintln!(
                    "{:>gutter_width$} {} {}",
                    "",
                    "|".blue().bold(),
                    underline.red().bold()
                );
            }
        }

        // Blank gutter line
        eprintln!("{:>gutter_width$} {}", "", "|".blue().bold());
    }

    // Attached notes
    for note in &diag.notes {
        eprintln!("  {} {}: {}", "=".blue().bold(), "note".bold(), note);
    }

    // Help suggestion
    if let Some(ref help) = diag.help {
        eprintln!("  {} {}: {}", "=".blue().bold(), "help".green().bold(), help);
    }

    eprintln!();
}
