//! Interfaces screen (spec §19).
//!
//! Table of interfaces with state, address, speed, counters and errors. Detail
//! view + live throughput graph are a follow-up (C4).

use crate::models::{InterfaceStatus, LinkState};
use crate::tui::screens::{header, link_tag};
use crate::tui::state::AppState;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use ratatui::Frame;

fn fmt_bytes(n: u64) -> String {
    const K: f64 = 1024.0;
    const M: f64 = K * 1024.0;
    const G: f64 = M * 1024.0;
    let n = n as f64;
    if n >= G {
        format!("{:.1}G", n / G)
    } else if n >= M {
        format!("{:.1}M", n / M)
    } else if n >= K {
        format!("{:.0}K", n / K)
    } else {
        format!("{n:.0}")
    }
}

fn cell(value: String, style: Style) -> Cell<'static> {
    Cell::from(Span::styled(value, style))
}

pub fn draw(state: &AppState, frame: &mut Frame) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(area);

    // Status line.
    let status = if let Some(err) = &state.interfaces_err {
        Span::styled(
            format!("Error loading interfaces: {err}"),
            Style::default().fg(Color::Red),
        )
    } else if state.interfaces.is_none() {
        Span::styled("Loading interfaces...", Style::default().fg(Color::Yellow))
    } else {
        let n = state.interfaces.as_ref().map(|v| v.len()).unwrap_or(0);
        let up = state
            .interfaces
            .as_ref()
            .map(|v| v.iter().filter(|i| i.link_state == LinkState::Up).count())
            .unwrap_or(0);
        Span::styled(
            format!("{n} interfaces, {up} up"),
            Style::default().fg(Color::DarkGray),
        )
    };
    frame.render_widget(
        Paragraph::new(Line::from(status)).block(Block::default().borders(Borders::NONE)),
        chunks[0],
    );

    // Table.
    let b = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled("INTERFACES", header()));
    let header_row = Row::new(vec![
        "NAME", "STATE", "ADDRESS", "SPEED", "RX", "TX", "ERR", "DROP",
    ])
    .style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(ratatui::style::Modifier::BOLD),
    );

    let rows = match &state.interfaces {
        Some(ifs) => ifs.iter().map(|i| row(i)).collect::<Vec<_>>(),
        None => Vec::new(),
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Length(9),
            Constraint::Length(18),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(6),
            Constraint::Length(6),
        ],
    )
    .header(header_row)
    .block(b);

    frame.render_widget(table, chunks[1]);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            " [q] back   [Esc] back   [?] help   (detail view coming soon)",
            Style::default().fg(Color::DarkGray),
        ))),
        chunks[2],
    );
}

fn row(i: &InterfaceStatus) -> Row<'static> {
    let (state, st) = link_tag(i.link_state);
    Row::new(vec![
        cell(i.name.clone(), Style::default().fg(Color::Cyan)),
        cell(state.to_string(), st),
        cell(
            i.ipv4.clone().unwrap_or_else(|| "--".to_string()),
            Style::default(),
        ),
        cell(
            i.speed_mbps
                .map(|s| {
                    if s >= 1000 {
                        format!("{} Gbps", s / 1000)
                    } else {
                        format!("{s} Mbps")
                    }
                })
                .unwrap_or_else(|| "--".to_string()),
            Style::default(),
        ),
        cell(fmt_bytes(i.rx_bytes), Style::default()),
        cell(fmt_bytes(i.tx_bytes), Style::default()),
        cell(i.errors.to_string(), err_style(i.errors)),
        cell(i.drops.to_string(), Style::default().fg(Color::DarkGray)),
    ])
}

fn err_style(errors: u64) -> Style {
    if errors > 0 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    }
}
