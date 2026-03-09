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

#[derive(Default)]
struct App {
    messages: Vec<Line<'static>>,
    input: String,
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
                        Constraint::Min(3),    // prompt + hints area
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
                    Style::default()
                        .add_modifier(Modifier::BOLD),
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
                    Style::default()
                        .add_modifier(Modifier::BOLD),
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

            // Prompt + hint area (mimic Claude Code bottom region).
            let mut prompt_lines: Vec<Line> = Vec::new();
            if app.input.is_empty() {
                prompt_lines.push(Line::from(vec![
                    Span::raw("> "),
                    Span::styled(
                        r#"Try "refactor <filepath>""#,
                        Style::default().fg(Color::Cyan),
                    ),
                ]));
            } else {
                prompt_lines.push(Line::from(format!("> {}", app.input)));
            }
            prompt_lines.push(Line::from(Span::styled(
                "? for shortcuts",
                Style::default().fg(Color::Cyan),
            )));

            let prompt_block =
                Block::default().borders(Borders::TOP);
            let prompt_para = Paragraph::new(prompt_lines).block(prompt_block);
            frame.render_widget(prompt_para, chunks[1]);
        })?;

        if event::poll(std::time::Duration::from_millis(200))? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        return Ok(())
                    }
                    KeyCode::Char(ch) => {
                        app.input.push(ch);
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Enter => {
                        if !app.input.trim().is_empty() {
                            let line = Line::from(format!("user > {}", app.input.trim()));
                            app.messages.push(line);
                            app.input.clear();
                        }
                    }
                    _ => {}
                },
                Event::Resize(_, _) => {
                    // Just redraw on next loop iteration.
                }
                _ => {}
            }
        }
    }
}

