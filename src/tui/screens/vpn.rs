//! IPsec / VPN screen (spec §24-25).
//!
//! Tunnel list with Phase 1/2 state, remote gateway, IKE version, traffic and
//! uptime, from the normalized `IpsecTunnel` model. The endpoint may be empty
//! on boxes with no IPsec — handled gracefully.

use crate::models::IpsecTunnel;
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

fn uptime(secs: Option<u64>) -> String {
    match secs {
        Some(s) if s >= 86400 => format!("{}d", s / 86400),
        Some(s) if s >= 3600 => format!("{}h{}m", s / 3600, (s % 3600) / 60),
        Some(s) if s >= 60 => format!("{}m", s / 60),
        Some(s) => format!("{s}s"),
        None => "--".to_string(),
    }
}

/// Phase-1 state → tag + style (FortiGate reports "up" / "down" / others).
fn p1(t: &IpsecTunnel) -> (String, Style) {
    match t.phase1_state.as_deref() {
        Some("up") | Some("Up") | Some("UP") => {
            ("UP".to_string(), Style::default().fg(Color::Green))
        }
        Some("down") | Some("Down") | Some("DOWN") => {
            ("DOWN".to_string(), Style::default().fg(Color::Red))
        }
        Some(other) => (other.to_uppercase(), Style::default().fg(Color::Yellow)),
        None => ("--".to_string(), Style::default().fg(Color::DarkGray)),
    }
}

fn row(t: &IpsecTunnel) -> Row<'static> {
    let (state, st) = p1(t);
    Row::new(vec![
        cell(t.name.clone(), Style::default().fg(Color::Cyan)),
        cell(state, st),
        cell(
            t.phase2_state.clone().unwrap_or_else(|| "--".into()),
            Style::default().fg(Color::DarkGray),
        ),
        cell(
            t.remote_gateway.clone().unwrap_or_else(|| "--".into()),
            Style::default(),
        ),
        cell(
            t.ike_version.clone().unwrap_or_else(|| "--".into()),
            Style::default(),
        ),
        cell(fmt_bytes(t.rx_bytes), Style::default()),
        cell(fmt_bytes(t.tx_bytes), Style::default()),
        cell(uptime(t.uptime_secs), Style::default().fg(Color::DarkGray)),
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

    let status = if let Some(err) = &state.vpn_err {
        Line::from(Span::styled(
            format!("Error loading IPsec: {err}"),
            Style::default().fg(Color::Red),
        ))
    } else if let Some(v) = &state.vpn {
        let up = v
            .iter()
            .filter(|t| {
                t.phase1_state
                    .as_deref()
                    .is_some_and(|s| s.eq_ignore_ascii_case("up"))
            })
            .count();
        Line::from(Span::styled(
            format!("{} tunnels, {up} up", v.len()),
            Style::default().fg(Color::DarkGray),
        ))
    } else {
        Line::from(Span::styled(
            "Loading IPsec...",
            Style::default().fg(Color::Yellow),
        ))
    };
    frame.render_widget(
        Paragraph::new(status).block(Block::default().borders(Borders::NONE)),
        chunks[0],
    );

    let b = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled("IPSEC TUNNELS", header()));
    let header_row = Row::new(vec![
        "NAME",
        "P1",
        "P2",
        "REMOTE GW",
        "IKE",
        "RX",
        "TX",
        "UPTIME",
    ])
    .style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );
    let rows: Vec<Row> = state
        .vpn
        .as_ref()
        .map(|v| v.iter().map(row).collect())
        .unwrap_or_default();
    let table = Table::new(
        rows,
        [
            Constraint::Length(18),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(16),
            Constraint::Length(6),
            Constraint::Length(9),
            Constraint::Length(9),
            Constraint::Length(8),
        ],
    )
    .header(header_row)
    .block(b);
    frame.render_widget(table, chunks[1]);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            " [Esc] back   [?] help   [r] refresh",
            Style::default().fg(Color::DarkGray),
        ))),
        chunks[2],
    );
}
