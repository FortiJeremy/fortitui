//! Diagnostics screen (spec §30–§33, C13).
//!
//! The FortiOS 8 monitor API does **not** expose ping / traceroute / DNS /
//! packet-capture results — those require the CLI/`execute` path (gap-analysis
//! Q2). This screen documents what is available today over the REST monitor API
//! versus what is deferred, so an operator knows the current capability without
//! guessing. Implemented-over-API actions link to the screen that already
//! provides them.

use crate::tui::screens::header;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use ratatui::Frame;

/// (diagnostic, availability, notes)
const DIAGNOSTICS: &[(&str, &str, &str)] = &[
    (
        "Ping (IPv4/IPv6)",
        "NOT AVAILABLE",
        "Requires CLI `execute ping` — not exposed via the REST monitor API.",
    ),
    (
        "Traceroute",
        "NOT AVAILABLE",
        "Requires CLI `execute traceroute` — not exposed via the REST monitor API.",
    ),
    (
        "DNS lookup",
        "NOT AVAILABLE",
        "No DNS diagnostic endpoint in the monitor API.",
    ),
    (
        "Packet capture / sniffer",
        "NOT AVAILABLE",
        "Requires CLI `diagnose sniffer packet` — not exposed via the REST monitor API.",
    ),
    (
        "Route lookup",
        "AVAILABLE",
        "Routing screen → press `l` (uses /monitor/router/lookup).",
    ),
    (
        "IPsec tunnel status",
        "AVAILABLE",
        "IPsec screen → `v`, Enter for detail (uses /monitor/vpn/ipsec).",
    ),
    (
        "Session lookup",
        "AVAILABLE",
        "Sessions screen → `/` to filter (uses /monitor/firewall/sessions).",
    ),
];

fn avail_style(a: &str) -> Style {
    match a {
        "AVAILABLE" => Style::default().fg(Color::Green),
        _ => Style::default().fg(Color::Yellow),
    }
}

pub fn draw(frame: &mut Frame) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // status
            Constraint::Min(3),    // table
            Constraint::Length(1), // hint
        ])
        .split(area);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "Diagnostics reflect FortiOS 8 monitor-API capabilities; read-only.",
            Style::default().fg(Color::DarkGray),
        ))),
        chunks[0],
    );

    let b = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled("DIAGNOSTICS", header()));
    let header_row = Row::new(vec!["DIAGNOSTIC", "STATUS", "NOTES"]).style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );
    let rows: Vec<Row> = DIAGNOSTICS
        .iter()
        .map(|(name, avail, notes)| {
            Row::new(vec![
                Cell::from(Span::styled(*name, Style::default().fg(Color::Cyan))),
                Cell::from(Span::styled(*avail, avail_style(avail))),
                Cell::from(Span::styled(*notes, Style::default())),
            ])
        })
        .collect();
    let table = Table::new(
        rows,
        [
            Constraint::Length(22),
            Constraint::Length(14),
            Constraint::Min(20),
        ],
    )
    .header(header_row)
    .block(b);
    frame.render_widget(table, chunks[1]);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            " [Esc] back   [?] help   Route lookup is on the Routing screen (`l`)",
            Style::default().fg(Color::DarkGray),
        ))),
        chunks[2],
    );
}
