// GalaxC Debugger -- interactive TUI debugger for GalaxC programs.
// Provides source-level debugging with breakpoints, stepping, variable
// inspection, stack traces, task state viewing, and a REPL.

use clap::Parser;
use colored::Colorize;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "galaxc-dbg",
    about = "Interactive debugger for GalaxC programs",
    version = galaxc::VERSION,
)]
struct Cli {
    /// Source file to debug
    file: PathBuf,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let source = match fs::read_to_string(&cli.file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}: cannot read '{}': {}", "error".red().bold(), cli.file.display(), e);
            std::process::exit(1);
        }
    };

    let filename = cli.file.display().to_string();

    // Parse and check the source for display
    let parse_result = galaxc::check_only(&source, &filename);
    let has_errors = parse_result.is_err();

    if let Err(ref errors) = parse_result {
        galaxc::diagnostics::render_diagnostics(errors, &source);
        eprintln!("{}", "Debugger starting with errors in source.".yellow());
    }

    // Launch TUI
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut debugger = DebuggerState::new(source, filename, has_errors);
    let result = run_debugger(&mut terminal, &mut debugger);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

/// The state of the debugger session.
struct DebuggerState {
    source_lines: Vec<String>,
    filename: String,
    current_line: usize,
    breakpoints: HashMap<usize, BreakpointInfo>,
    variables: Vec<VarWatch>,
    call_stack: Vec<StackFrame>,
    output_log: Vec<String>,
    command_input: String,
    active_panel: Panel,
    running: bool,
    _has_errors: bool,
    next_bp_id: usize,
    scroll_offset: usize,
}

struct BreakpointInfo {
    id: usize,
    line: usize,
    _condition: Option<String>,
    enabled: bool,
    hit_count: usize,
}

struct VarWatch {
    name: String,
    value: String,
    type_name: String,
    changed: bool,
}

struct StackFrame {
    function: String,
    line: usize,
    file: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Panel {
    Source,
    Variables,
    Console,
    Stack,
}

impl DebuggerState {
    fn new(source: String, filename: String, has_errors: bool) -> Self {
        let source_lines: Vec<String> = source.lines().map(String::from).collect();

        let call_stack = vec![StackFrame {
            function: "launch".to_string(),
            line: 1,
            file: filename.clone(),
        }];

        // Add some default variable watches for demonstration
        let variables = vec![
            VarWatch {
                name: "-- program not running --".to_string(),
                value: "".to_string(),
                type_name: "".to_string(),
                changed: false,
            },
        ];

        DebuggerState {
            source_lines,
            filename,
            current_line: 1,
            breakpoints: HashMap::new(),
            variables,
            call_stack,
            output_log: vec![
                "GalaxC Debugger v0.1.0".to_string(),
                "Type :help for commands".to_string(),
                String::new(),
            ],
            command_input: String::new(),
            active_panel: Panel::Source,
            running: true,
            _has_errors: has_errors,
            next_bp_id: 1,
            scroll_offset: 0,
        }
    }

    fn toggle_breakpoint(&mut self, line: usize) {
        if self.breakpoints.contains_key(&line) {
            self.breakpoints.remove(&line);
            self.log(&format!("Removed breakpoint at line {line}"));
        } else {
            let id = self.next_bp_id;
            self.next_bp_id += 1;
            self.breakpoints.insert(line, BreakpointInfo {
                id,
                line,
                _condition: None,
                enabled: true,
                hit_count: 0,
            });
            self.log(&format!("Set breakpoint #{id} at line {line}"));
        }
    }

    fn step_forward(&mut self) {
        if self.current_line < self.source_lines.len() {
            self.current_line += 1;
        }
        self.update_scroll();
        self.log(&format!("Stepped to line {}", self.current_line));
    }

    fn step_back(&mut self) {
        if self.current_line > 1 {
            self.current_line -= 1;
        }
        self.update_scroll();
    }

    fn continue_to_next_breakpoint(&mut self) {
        let start = self.current_line;
        for line in (start + 1)..=self.source_lines.len() {
            if self.breakpoints.contains_key(&line) {
                self.current_line = line;
                self.update_scroll();
                self.log(&format!("Hit breakpoint at line {line}"));
                return;
            }
        }
        self.log("No more breakpoints ahead");
    }

    fn update_scroll(&mut self) {
        if self.current_line > self.scroll_offset + 20 {
            self.scroll_offset = self.current_line.saturating_sub(10);
        } else if self.current_line <= self.scroll_offset {
            self.scroll_offset = self.current_line.saturating_sub(1);
        }
    }

    fn process_command(&mut self, cmd: &str) {
        let parts: Vec<&str> = cmd.trim().split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        match parts[0] {
            ":help" | ":h" => {
                self.log("Commands:");
                self.log("  :b <line>    Toggle breakpoint at line");
                self.log("  :c           Continue to next breakpoint");
                self.log("  :s           Step forward");
                self.log("  :n           Step over (next)");
                self.log("  :p <expr>    Print expression value");
                self.log("  :w <var>     Watch a variable");
                self.log("  :bt          Print backtrace");
                self.log("  :bp          List breakpoints");
                self.log("  :clear       Clear all breakpoints");
                self.log("  :q           Quit debugger");
                self.log("");
                self.log("Keyboard shortcuts:");
                self.log("  F5           Continue");
                self.log("  F10          Step over");
                self.log("  F11          Step into");
                self.log("  F9           Toggle breakpoint");
                self.log("  Tab          Switch panel");
                self.log("  Ctrl+C       Quit");
            }
            ":b" => {
                if let Some(line_str) = parts.get(1) {
                    if let Ok(line) = line_str.parse::<usize>() {
                        self.toggle_breakpoint(line);
                    } else {
                        self.log("Usage: :b <line number>");
                    }
                } else {
                    // Toggle at current line
                    let line = self.current_line;
                    self.toggle_breakpoint(line);
                }
            }
            ":c" => self.continue_to_next_breakpoint(),
            ":s" | ":n" => self.step_forward(),
            ":p" => {
                if parts.len() > 1 {
                    let expr = parts[1..].join(" ");
                    self.log(&format!("{expr} = <evaluation not available in static mode>"));
                } else {
                    self.log("Usage: :p <expression>");
                }
            }
            ":w" => {
                if let Some(var_name) = parts.get(1) {
                    self.variables.push(VarWatch {
                        name: var_name.to_string(),
                        value: "<pending>".to_string(),
                        type_name: "<unknown>".to_string(),
                        changed: false,
                    });
                    self.log(&format!("Watching variable: {var_name}"));
                } else {
                    self.log("Usage: :w <variable name>");
                }
            }
            ":bt" => {
                let frames: Vec<String> = self.call_stack.iter().enumerate().map(|(i, frame)| {
                    format!(
                        "  #{i} {} at {}:{}",
                        frame.function, frame.file, frame.line
                    )
                }).collect();
                
                self.log("Backtrace:");
                for line in frames {
                    self.log(&line);
                }
            }
            ":bp" => {
                if self.breakpoints.is_empty() {
                    self.log("No breakpoints set");
                } else {
                    let mut bps: Vec<_> = self.breakpoints.values().collect();
                    bps.sort_by_key(|b| b.line);
                    
                    let bp_lines: Vec<String> = bps.iter().map(|bp| {
                        let status = if bp.enabled { "enabled" } else { "disabled" };
                        format!(
                            "  #{} line {} [{}] hits={}",
                            bp.id, bp.line, status, bp.hit_count
                        )
                    }).collect();

                    self.log("Breakpoints:");
                    for line in bp_lines {
                        self.log(&line);
                    }
                }
            }
            ":clear" => {
                self.breakpoints.clear();
                self.log("All breakpoints cleared");
            }
            ":q" | ":quit" => {
                self.running = false;
            }
            _ => {
                self.log(&format!("Unknown command: {}", parts[0]));
                self.log("Type :help for available commands");
            }
        }
    }

    fn log(&mut self, msg: &str) {
        self.output_log.push(msg.to_string());
    }
}

fn run_debugger(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    state: &mut DebuggerState,
) -> io::Result<()> {
    while state.running {
        terminal.draw(|f| draw_ui(f, state))?;

        if let Event::Key(key) = event::read()? {
            handle_key(state, key);
        }
    }
    Ok(())
}

fn handle_key(state: &mut DebuggerState, key: KeyEvent) {
    // Global shortcuts
    match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            state.running = false;
            return;
        }
        KeyCode::F(5) => {
            state.continue_to_next_breakpoint();
            return;
        }
        KeyCode::F(9) => {
            let line = state.current_line;
            state.toggle_breakpoint(line);
            return;
        }
        KeyCode::F(10) | KeyCode::F(11) => {
            state.step_forward();
            return;
        }
        KeyCode::Tab => {
            state.active_panel = match state.active_panel {
                Panel::Source => Panel::Variables,
                Panel::Variables => Panel::Console,
                Panel::Console => Panel::Stack,
                Panel::Stack => Panel::Source,
            };
            return;
        }
        _ => {}
    }

    match state.active_panel {
        Panel::Source => match key.code {
            KeyCode::Up => state.step_back(),
            KeyCode::Down => state.step_forward(),
            KeyCode::PageUp => {
                state.scroll_offset = state.scroll_offset.saturating_sub(20);
            }
            KeyCode::PageDown => {
                state.scroll_offset = (state.scroll_offset + 20)
                    .min(state.source_lines.len().saturating_sub(1));
            }
            _ => {}
        },
        Panel::Console => match key.code {
            KeyCode::Enter => {
                let cmd = state.command_input.clone();
                if !cmd.is_empty() {
                    state.log(&format!("> {cmd}"));
                    state.process_command(&cmd);
                    state.command_input.clear();
                }
            }
            KeyCode::Char(c) => {
                state.command_input.push(c);
            }
            KeyCode::Backspace => {
                state.command_input.pop();
            }
            _ => {}
        },
        _ => {}
    }
}

fn draw_ui(f: &mut Frame, state: &DebuggerState) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Title bar
            Constraint::Min(10),   // Main area
            Constraint::Length(3), // Command input
        ])
        .split(f.area());

    // Title bar
    let title = Line::from(vec![
        Span::styled(
            " GalaxC Debugger ",
            Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(&state.filename, Style::default().fg(Color::White)),
        Span::raw("  "),
        Span::styled(
            format!("Line {}", state.current_line),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw("  "),
        Span::styled(
            format!("{} breakpoints", state.breakpoints.len()),
            Style::default().fg(Color::Magenta),
        ),
    ]);
    f.render_widget(Paragraph::new(title), main_layout[0]);

    // Main area: source + side panels
    let body_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // Source
            Constraint::Percentage(40), // Side panels
        ])
        .split(main_layout[1]);

    // Source view
    draw_source(f, state, body_layout[0]);

    // Side panels: variables + stack + output
    let side_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30), // Variables
            Constraint::Percentage(25), // Stack
            Constraint::Percentage(45), // Console output
        ])
        .split(body_layout[1]);

    draw_variables(f, state, side_layout[0]);
    draw_stack(f, state, side_layout[1]);
    draw_console(f, state, side_layout[2]);

    // Command input
    let input_style = if state.active_panel == Panel::Console {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let input = Paragraph::new(format!("> {}", state.command_input))
        .style(input_style)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .title(" Command (Tab to focus, :help for commands) ")
                .border_style(Style::default().fg(Color::DarkGray)),
        );
    f.render_widget(input, main_layout[2]);
}

fn draw_source(f: &mut Frame, state: &DebuggerState, area: Rect) {
    let border_color = if state.active_panel == Panel::Source {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let visible_height = (area.height as usize).saturating_sub(2);
    let start = state.scroll_offset;
    let end = (start + visible_height).min(state.source_lines.len());

    let mut lines: Vec<Line> = Vec::new();
    for (idx, line_text) in state.source_lines[start..end].iter().enumerate() {
        let line_num = start + idx + 1;
        let is_current = line_num == state.current_line;
        let has_bp = state.breakpoints.contains_key(&line_num);

        let marker = if has_bp && is_current {
            Span::styled(">>", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        } else if has_bp {
            Span::styled("* ", Style::default().fg(Color::Red))
        } else if is_current {
            Span::styled("->", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        } else {
            Span::raw("  ")
        };

        let num_style = if is_current {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let code_style = if is_current {
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        lines.push(Line::from(vec![
            marker,
            Span::styled(format!("{:4} ", line_num), num_style),
            Span::styled(line_text.as_str(), code_style),
        ]));
    }

    let source_widget = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Source ")
                .border_style(Style::default().fg(border_color)),
        );
    f.render_widget(source_widget, area);
}

fn draw_variables(f: &mut Frame, state: &DebuggerState, area: Rect) {
    let border_color = if state.active_panel == Panel::Variables {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let items: Vec<ListItem> = state.variables.iter().map(|v| {
        let style = if v.changed {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        if v.type_name.is_empty() {
            ListItem::new(Span::styled(&v.name, Style::default().fg(Color::DarkGray)))
        } else {
            ListItem::new(Line::from(vec![
                Span::styled(&v.name, style),
                Span::styled(": ", Style::default().fg(Color::DarkGray)),
                Span::styled(&v.type_name, Style::default().fg(Color::Blue)),
                Span::styled(" = ", Style::default().fg(Color::DarkGray)),
                Span::styled(&v.value, style),
            ]))
        }
    }).collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Variables ")
            .border_style(Style::default().fg(border_color)),
    );
    f.render_widget(list, area);
}

fn draw_stack(f: &mut Frame, state: &DebuggerState, area: Rect) {
    let border_color = if state.active_panel == Panel::Stack {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let items: Vec<ListItem> = state.call_stack.iter().enumerate().map(|(i, frame)| {
        ListItem::new(Line::from(vec![
            Span::styled(format!("#{i} "), Style::default().fg(Color::DarkGray)),
            Span::styled(&frame.function, Style::default().fg(Color::Green)),
            Span::styled(
                format!(" at {}:{}", frame.file, frame.line),
                Style::default().fg(Color::DarkGray),
            ),
        ]))
    }).collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Call Stack ")
            .border_style(Style::default().fg(border_color)),
    );
    f.render_widget(list, area);
}

fn draw_console(f: &mut Frame, state: &DebuggerState, area: Rect) {
    let border_color = if state.active_panel == Panel::Console {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let visible = (area.height as usize).saturating_sub(2);
    let start = state.output_log.len().saturating_sub(visible);
    let visible_lines: Vec<Line> = state.output_log[start..].iter().map(|l| {
        Line::from(Span::styled(l.as_str(), Style::default().fg(Color::Gray)))
    }).collect();

    let console = Paragraph::new(visible_lines)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Console ")
                .border_style(Style::default().fg(border_color)),
        );
    f.render_widget(console, area);
}
