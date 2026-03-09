use ratatui::{
    crossterm::{
        event::{self, Event, KeyCode},
        terminal::{disable_raw_mode, enable_raw_mode},
    },
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph},
};

const ONEBIRD_ASCII: &str = r#"   ___
  ('v')
 ((   ))
  -"-"-"#;

const PROVIDERS: &[&str] = &[
    "Anthropic (Claude Pro/Max)",
    "GitHub Copilot",
    "Google Cloud Code Assist (Gemini CLI)",
    "Antigravity (Gemini 3, Claude, GPT-OSS)",
    "ChatGPT Plus/Pro (Codex Subscription)",
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Home,
    Login,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum HintMode {
    Default,
    Shortcuts,
    Commands,
}

struct App {
    messages: Vec<Line<'static>>,
    input: String,
    mode: Mode,
    login_index: usize,
    hint_mode: HintMode,
}

impl Default for App {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            input: String::new(),
            mode: Mode::Home,
            login_index: 0,
            hint_mode: HintMode::Default,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut terminal = ratatui::init();

    let mut app = App::default();
    init_app(&mut app);

    let res = run_app(&mut terminal, &mut app);

    ratatui::restore();
    disable_raw_mode()?;

    res
}

fn init_app(app: &mut App) {
    // Initial conversation hints, similar to Claude Code.
    app.messages
        .push(Line::from(r#"Try "how do I log an error?""#));
    app.messages.push(Line::from(""));
    app.messages.push(Line::from("? for shortcuts"));
}

fn run_app(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|frame| {
            let size = frame.area();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(9), // welcome panel with titled border
                        Constraint::Min(3),    // prompt / login area
                    ]
                    .as_ref(),
                )
                .split(size);

            // Welcome panel (two columns, like Claude Code) with title in top-left.
            let welcome_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(Span::styled(
                    format!(" Onebird Harness v{}", env!("CARGO_PKG_VERSION")),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ));
            let inner = welcome_block.inner(chunks[0]);
            frame.render_widget(welcome_block, chunks[0]);

            let columns = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
                .split(inner);

            // Left: Onebird ASCII logo + short label.
            let mut left_lines: Vec<Line> = vec![
                Line::from(Span::styled(
                    "  Welcome back!",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
            ];
            // Preserve multi-line ASCII bird shape.
            left_lines.extend(
                ONEBIRD_ASCII
                    .lines()
                    .map(|l| Line::from(format!("  {}", l)))
                    .collect::<Vec<_>>(),
            );
            left_lines.extend(vec![
                Line::from(""),
                Line::from(Span::styled(
                    "  onebird-cli",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Line::from("  ~ /github/onebird/onebird-cli"),
            ]);
            let left = Paragraph::new(left_lines);

            // Draw inner vertical separator between left and right.
            let left_block = Block::default()
                .borders(Borders::RIGHT)
                .border_style(Style::default().fg(Color::Yellow));
            let left_inner = left_block.inner(columns[0]);
            frame.render_widget(left_block, columns[0]);
            frame.render_widget(left, left_inner);

            // Right: tips + recent activity (using default terminal colors).
            let right_lines: Vec<Line> = vec![
                Line::from(Span::styled(
                    "Tips for getting started",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    "Run /init to create a ONEBIRD.md file",
                    Style::default(),
                )),
                Line::from(Span::styled(
                    "with instructions for your AI harness",
                    Style::default(),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Recent activity",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Line::from(Span::styled(
                    "No recent activity",
                    Style::default(),
                )),
            ];
            let right = Paragraph::new(right_lines);
            // Add padding between the center vertical line and the right content,
            // so text does not stick to the separator.
            let right_block = Block::default().padding(Padding::new(1, 1, 0, 0));
            let right_inner = right_block.inner(columns[1]);
            frame.render_widget(right_block, columns[1]);
            frame.render_widget(right, right_inner);

            // Bottom area: either prompt or login selector, depending on mode.
            match app.mode {
                Mode::Home => {
                    let mut lines: Vec<Line> = Vec::new();

                    // First line: prompt itself.
                    if app.input.is_empty() {
                        lines.push(Line::from(vec![
                            Span::raw("> "),
                            Span::styled(
                                r#"Try "refactor <filepath>""#,
                                Style::default().fg(Color::Cyan),
                            ),
                        ]));
                    } else {
                        lines.push(Line::from(format!("> {}", app.input)));
                    }

                    // Following lines: hints / help.
                    match app.hint_mode {
                        HintMode::Default => {
                            lines.push(Line::from(Span::styled(
                                "? for shortcuts, / for commands",
                                Style::default().fg(Color::Cyan),
                            )));
                        }
                        HintMode::Shortcuts => {
                            lines.push(Line::from(Span::styled(
                                "Shortcuts:",
                                Style::default().fg(Color::Cyan),
                            )));
                            lines.push(Line::from("  ?         Show shortcuts"));
                            lines.push(Line::from("  /         Start a command (e.g. /login)"));
                            lines.push(Line::from("  Ctrl-C    Quit onebird-cli"));
                        }
                        HintMode::Commands => {
                            lines.push(Line::from(Span::styled(
                                "Commands:",
                                Style::default().fg(Color::Cyan),
                            )));
                            lines.push(Line::from("  /login    Select provider to login"));
                        }
                    }

                    let prompt_block = Block::default().borders(Borders::TOP);
                    let prompt_para = Paragraph::new(lines).block(prompt_block);
                    frame.render_widget(prompt_para, chunks[1]);
                }
                Mode::Login => {
                    let block = Block::default()
                        .borders(Borders::TOP)
                        .border_style(Style::default().fg(Color::Yellow));
                    let inner = block.inner(chunks[1]);
                    frame.render_widget(block, chunks[1]);

                    let mut lines: Vec<Line> = Vec::new();
                    lines.push(Line::from("Select provider to login:"));
                    lines.push(Line::from(""));
                    for (idx, provider) in PROVIDERS.iter().enumerate() {
                        let indicator = if idx == app.login_index { "→ " } else { "  " };
                        let style = if idx == app.login_index {
                            Style::default().add_modifier(Modifier::BOLD)
                        } else {
                            Style::default()
                        };
                        lines.push(Line::from(Span::styled(
                            format!("{indicator}{provider}"),
                            style,
                        )));
                    }
                    let para = Paragraph::new(lines);
                    frame.render_widget(para, inner);
                }
            }
        })?;

        if event::poll(std::time::Duration::from_millis(200))? {
            match event::read()? {
                Event::Key(key) => {
                    // Global exit shortcut.
                    if key.code == KeyCode::Char('c')
                        && key.modifiers.contains(event::KeyModifiers::CONTROL)
                    {
                        return Ok(());
                    }

                    match app.mode {
                        Mode::Home => match key.code {
                            KeyCode::Char('?') => {
                                if app.input.is_empty() {
                                    // Show shortcuts instead of inserting '?' into input.
                                    app.hint_mode = HintMode::Shortcuts;
                                } else {
                                    app.hint_mode = HintMode::Default;
                                    app.input.push('?');
                                }
                            }
                            KeyCode::Char('/') => {
                                if app.input.is_empty() {
                                    // Enter command mode, show all commands and seed '/'.
                                    app.hint_mode = HintMode::Commands;
                                } else {
                                    app.hint_mode = HintMode::Default;
                                }
                                app.input.push('/');
                            }
                            KeyCode::Char(ch) => {
                                app.hint_mode = HintMode::Default;
                                app.input.push(ch);
                            }
                            KeyCode::Backspace => {
                                app.input.pop();
                                if app.input.is_empty() {
                                    app.hint_mode = HintMode::Default;
                                }
                            }
                            KeyCode::Esc => {
                                app.input.clear();
                                app.hint_mode = HintMode::Default;
                            }
                            KeyCode::Enter => {
                                if !app.input.trim().is_empty() {
                                    let trimmed = app.input.trim().to_string();
                                    if trimmed == "/login" {
                                        app.mode = Mode::Login;
                                        app.login_index = 0;
                                        app.input.clear();
                                        app.hint_mode = HintMode::Default;
                                    } else {
                                        let line = Line::from(format!("user > {}", trimmed));
                                        app.messages.push(line);
                                        app.input.clear();
                                        app.hint_mode = HintMode::Default;
                                    }
                                }
                            }
                            _ => {}
                        },
                        Mode::Login => match key.code {
                            KeyCode::Up => {
                                if app.login_index > 0 {
                                    app.login_index -= 1;
                                }
                            }
                            KeyCode::Down => {
                                if app.login_index + 1 < PROVIDERS.len() {
                                    app.login_index += 1;
                                }
                            }
                            KeyCode::Enter => {
                                let provider = PROVIDERS[app.login_index];
                                app.messages.push(Line::from(format!(
                                    "login > {}",
                                    provider
                                )));
                                app.mode = Mode::Home;
                                app.input.clear();
                            }
                            KeyCode::Esc => {
                                app.mode = Mode::Home;
                                app.input.clear();
                            }
                            _ => {}
                        },
                    }
                }
                Event::Resize(_, _) => {
                    // Just redraw on next loop iteration.
                }
                _ => {}
            }
        }
    }
}

