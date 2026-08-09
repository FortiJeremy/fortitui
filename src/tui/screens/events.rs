//! Events / state-transition screen (spec §36).
//!
//! In-memory log of detected state changes, most recent first. Severity is
//! represented by color + text label so it stays readable without color.

use crate::tui::screens::header;
use crate::tui::state::AppState;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

fn severity_style(sev: &str) -> Style {
    if sev.eq_ignore_ascii_case("CRITICAL") {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    } else if sev.eq_ignore_ascii_case("WARNING") {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Green)
    }
}

fn fmt_time(ts: u64) -> String {
    let h = (ts % 86400) / 3600;
    let m = (ts % 3600) / 60;
    let s = ts % 60;
    format!("{h:02}:{m:02}:{s:02}")
}

pub fn draw(state: &AppState, frame: &mut Frame) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // status
            Constraint::Min(3),    // event list
            Constraint::Length(1), // hint
        ])
        .split(area);

    let status = Line::from(Span::styled(
        format!("{} recent events (in-memory)", state.events.len()),
        Style::default().fg(Color::DarkGray),
    ));
    frame.render_widget(
        Paragraph::new(status).block(Block::default().borders(Borders::NONE)),
        chunks[0],
    );

    let b = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled("RECENT EVENTS", header()));
    let inner = b.inner(chunks[1]);
    frame.render_widget(b, chunks[1]);

    let mut lines: Vec<Line> = Vec::new();
    if state.events.is_empty() {
        lines.push(Line::from(Span::styled(
            "No state transitions detected yet.",
            Style::default().fg(Color::Yellow),
        )));
    } else {
        for e in state.events.iter().rev() {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{} ", fmt_time(e.timestamp)),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{:<8} ", e.severity.to_uppercase()),
                    severity_style(&e.severity),
                ),
                Span::styled(e.description.clone(), Style::default()),
            ]));
        }
    }
    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::default())
            .wrap(Wrap { trim: true }),
        inner,
    );

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            " [Esc] back   [?] help   [r] refresh",
            Style::default().fg(Color::DarkGray),
        ))),
        chunks[2],
    );
}
