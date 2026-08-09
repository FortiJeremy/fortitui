//! Global + contextual help screen (spec §16, §63).

use crate::tui::screens::header;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

const KEYS: &[(&str, &str)] = &[
    ("q", "Quit"),
    ("?", "Toggle help"),
    ("Esc", "Back / previous screen"),
    ("r", "Refresh data"),
    ("Tab / Shift+Tab", "Next / previous panel"),
    ("↑ ↓ ← →", "Navigate"),
    ("Enter", "Select"),
    ("/", "Search"),
];

const SCREENS: &[(&str, &str)] = &[
    ("Dashboard", "Overall health (default screen)"),
    ("i", "Interfaces"),
    ("s", "SD-WAN"),
    ("v", "VPN / IPsec"),
    ("g", "Routing / BGP"),
    ("d", "Diagnostics"),
    ("f", "Firewall / Sessions"),
    ("e", "Events"),
];

pub fn draw(frame: &mut Frame) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(area);

    let b = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled("HELP", header()));
    let inner = b.inner(chunks[0]);
    frame.render_widget(b, chunks[0]);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        "FortiTUI — keyboard-driven console for FortiGate",
        Style::default().fg(Color::Cyan),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "GLOBAL KEYS",
        Style::default().fg(Color::Yellow),
    )));
    for (k, v) in KEYS {
        lines.push(Line::from(vec![
            Span::styled(format!("  {k:<14}"), Style::default().fg(Color::Green)),
            Span::styled(*v, Style::default()),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "SCREENS",
        Style::default().fg(Color::Yellow),
    )));
    for (k, v) in SCREENS {
        lines.push(Line::from(vec![
            Span::styled(format!("  {k:<14}"), Style::default().fg(Color::Green)),
            Span::styled(*v, Style::default()),
        ]));
    }

    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::default())
            .wrap(Wrap { trim: true }),
        inner,
    );

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            " [q] quit   [Esc] back",
            Style::default().fg(Color::DarkGray),
        ))),
        chunks[1],
    );
}
