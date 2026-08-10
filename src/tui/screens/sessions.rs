//! Active firewall sessions screen (spec §34).
//!
//! Table of src→dst, proto, policy, interface, bytes/packets and age from the
//! normalized `FirewallSession` model. Filtering (/ then src/dst/proto) lands
//! with the search milestone (D1).

use crate::models::FirewallSession;
use crate::tui::screens::{header, matches_search};
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

fn age(secs: u64) -> String {
    if secs >= 3600 {
        format!("{}h{}m", secs / 3600, (secs % 3600) / 60)
    } else if secs >= 60 {
        format!("{}m{}s", secs / 60, secs % 60)
    } else {
        format!("{secs}s")
    }
}

fn row(s: &FirewallSession) -> Row<'static> {
    let proto = s.proto.to_uppercase();
    let proto_style = if proto == "TCP" {
        Style::default().fg(Color::Cyan)
    } else if proto == "UDP" {
        Style::default().fg(Color::Magenta)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    Row::new(vec![
        cell(
            format!("{}:{}", s.src, s.src_port),
            Style::default().fg(Color::White),
        ),
        cell(
            format!("{}:{}", s.dst, s.dst_port),
            Style::default().fg(Color::White),
        ),
        cell(proto, proto_style),
        cell(
            s.policy.map_or("--".into(), |p| p.to_string()),
            Style::default().fg(Color::Yellow),
        ),
        cell(
            s.interface.clone().unwrap_or_else(|| "--".into()),
            Style::default(),
        ),
        cell(fmt_bytes(s.bytes), Style::default()),
        cell(fmt_bytes(s.packets), Style::default()),
        cell(age(s.age_secs), Style::default().fg(Color::DarkGray)),
    ])
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

    let status = if let Some(err) = &state.sessions_err {
        Line::from(Span::styled(
            format!("Error loading sessions: {err}"),
            Style::default().fg(Color::Red),
        ))
    } else if let Some(s) = &state.sessions {
        Line::from(Span::styled(
            format!("{} active sessions", s.len()),
            Style::default().fg(Color::DarkGray),
        ))
    } else {
        Line::from(Span::styled(
            "Loading sessions...",
            Style::default().fg(Color::Yellow),
        ))
    };
    frame.render_widget(
        Paragraph::new(status).block(Block::default().borders(Borders::NONE)),
        chunks[0],
    );

    let b = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled("SESSIONS", header()));
    let header_row = Row::new(vec![
        "SOURCE",
        "DESTINATION",
        "PROTO",
        "POL",
        "IFACE",
        "BYTES",
        "PKTS",
        "AGE",
    ])
    .style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );
    let rows: Vec<Row> = state
        .sessions
        .as_ref()
        .map(|v| {
            let needle = state.search.clone();
            v.iter()
                .filter(|s| {
                    let hay = [
                        s.src.clone(),
                        s.dst.clone(),
                        s.proto.clone(),
                        s.interface.clone().unwrap_or_default(),
                        s.policy.map(|p| p.to_string()).unwrap_or_default(),
                    ];
                    let refs: Vec<&str> = hay.iter().map(|x| x.as_str()).collect();
                    matches_search(&needle, &refs)
                })
                .map(row)
                .collect()
        })
        .unwrap_or_default();
    let table = Table::new(
        rows,
        [
            Constraint::Length(22),
            Constraint::Length(22),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(14),
            Constraint::Length(9),
            Constraint::Length(8),
            Constraint::Length(8),
        ],
    )
    .header(header_row)
    .block(b);
    frame.render_widget(table, chunks[1]);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            " [Esc] back   [?] help   [r] refresh   [F] firewall policies",
            Style::default().fg(Color::DarkGray),
        ))),
        chunks[2],
    );
}
