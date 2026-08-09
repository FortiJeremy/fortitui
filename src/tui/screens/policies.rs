//! Firewall policy screen (spec §35) — operational counters.

use crate::tui::screens::header;
use crate::tui::state::AppState;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use ratatui::Frame;

fn cell(value: String, style: Style) -> Cell<'static> {
    Cell::from(Span::styled(value, style))
}

fn fmt_bytes(n: u64) -> String {
    const K: f64 = 1024.0;
    const M: f64 = K * 1024.0;
    const G: f64 = M * 1024.0;
    const T: f64 = G * 1024.0;
    let n = n as f64;
    if n >= T {
        format!("{:.1}T", n / T)
    } else if n >= G {
        format!("{:.1}G", n / G)
    } else if n >= M {
        format!("{:.1}M", n / M)
    } else if n >= K {
        format!("{:.0}K", n / K)
    } else {
        format!("{n:.0}")
    }
}

pub fn draw(state: &AppState, frame: &mut Frame) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // status
            Constraint::Min(3),    // table
            Constraint::Length(1), // hint
        ])
        .split(area);

    let status = if let Some(err) = &state.policies_err {
        Line::from(Span::styled(
            format!("Error loading policies: {err}"),
            Style::default().fg(Color::Red),
        ))
    } else if let Some(p) = &state.policies {
        Line::from(Span::styled(
            format!("{} firewall policies", p.len()),
            Style::default().fg(Color::DarkGray),
        ))
    } else {
        Line::from(Span::styled(
            "Loading policies...",
            Style::default().fg(Color::Yellow),
        ))
    };
    frame.render_widget(
        Paragraph::new(status).block(Block::default().borders(Borders::NONE)),
        chunks[0],
    );

    let b = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled("FIREWALL POLICIES", header()));
    let header_row = Row::new(vec!["ID", "HITS", "BYTES", "SESSIONS"]).style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );
    let rows: Vec<Row> = state
        .policies
        .as_ref()
        .map(|v| {
            v.iter()
                .map(|p| {
                    Row::new(vec![
                        cell(p.id.to_string(), Style::default().fg(Color::Cyan)),
                        cell(p.hit_count.to_string(), Style::default()),
                        cell(fmt_bytes(p.bytes), Style::default()),
                        cell(
                            p.sessions.to_string(),
                            if p.sessions > 0 {
                                Style::default().fg(Color::Green)
                            } else {
                                Style::default().fg(Color::DarkGray)
                            },
                        ),
                    ])
                })
                .collect()
        })
        .unwrap_or_default();
    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Length(14),
            Constraint::Length(14),
            Constraint::Length(10),
        ],
    )
    .header(header_row)
    .block(b);
    frame.render_widget(table, chunks[1]);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            " [Esc] back   [?] help   [r] refresh   [f] active sessions",
            Style::default().fg(Color::DarkGray),
        ))),
        chunks[2],
    );
}
