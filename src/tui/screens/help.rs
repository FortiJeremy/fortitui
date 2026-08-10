//! Contextual help (spec §63, D3).
//!
//! `?` shows help describing the screen the operator was on (global keys +
//! screen-specific keys), rather than one fixed page. `Esc` returns to the
//! subject screen.

use crate::tui::screens::Screen;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

/// Global keys shown on every help page.
const GLOBAL_KEYS: &[(&str, &str)] = &[
    ("q", "Quit"),
    ("?", "Toggle contextual help"),
    (":", "Command palette"),
    ("/", "Search / filter"),
    ("Esc", "Back / close overlay"),
    ("r", "Refresh data"),
];

/// Screen-specific key bindings (contextual help).
fn screen_keys(screen: Screen) -> &'static [(&'static str, &'static str)] {
    match screen {
        Screen::Dashboard => &[
            ("i", "Interfaces"),
            ("o", "System"),
            ("s", "SD-WAN"),
            ("v", "IPsec VPN"),
            ("g", "Routing / BGP"),
            ("f", "Sessions"),
            ("F", "Firewall policies"),
            ("e", "Events"),
            ("d", "Diagnostics"),
        ],
        Screen::Interfaces => &[
            ("↑/↓", "Select interface"),
            ("Enter", "Toggle detail + live graph"),
            ("/", "Filter by name/state"),
            ("Esc", "Close detail / back"),
        ],
        Screen::System => &[("/", "Filter"), ("Esc", "Back")],
        Screen::Sdwan => &[
            ("↑/↓", "Select member"),
            ("l", "Toggle rolling latency/loss/jitter trend (C7)"),
            ("/", "Filter members/health checks"),
            ("Esc", "Close trend / back"),
        ],
        Screen::Sessions => &[
            ("/", "Filter sessions (src/dst/proto/policy)"),
            ("F", "Firewall policies"),
            ("Esc", "Back"),
        ],
        Screen::Policies => &[("/", "Filter policies"), ("Esc", "Back")],
        Screen::Ipsec => &[
            ("↑/↓", "Select tunnel"),
            ("Enter", "Toggle Phase 1/2 + cryptography detail (C9)"),
            ("/", "Filter tunnels"),
            ("Esc", "Close detail / back"),
        ],
        Screen::Routing => &[
            ("l", "Route lookup for a destination"),
            ("/", "Filter routes"),
            ("Esc", "Back"),
        ],
        Screen::Events => &[("/", "Filter events"), ("Esc", "Back")],
        Screen::Help | Screen::Diagnostics => &[("Esc", "Back to previous screen")],
    }
}

pub fn draw(subject: Screen, frame: &mut Frame) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(area);

    let b = Block::default().borders(Borders::ALL).title(Span::styled(
        format!("HELP — {}", subject.title()),
        crate::tui::screens::header(),
    ));
    let inner = b.inner(chunks[0]);
    frame.render_widget(b, chunks[0]);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        "FortiTUI — keyboard-driven console for FortiGate",
        Style::default().fg(Color::Cyan),
    )));
    lines.push(Line::from(""));

    lines.push(Line::from(Span::styled(
        format!("{} SCREEN KEYS", subject.title().to_uppercase()),
        Style::default().fg(Color::Yellow),
    )));
    for (k, v) in screen_keys(subject) {
        lines.push(Line::from(vec![
            Span::styled(format!("  {k:<16}"), Style::default().fg(Color::Green)),
            Span::styled(*v, Style::default()),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "GLOBAL KEYS",
        Style::default().fg(Color::Yellow),
    )));
    for (k, v) in GLOBAL_KEYS {
        lines.push(Line::from(vec![
            Span::styled(format!("  {k:<16}"), Style::default().fg(Color::Green)),
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
