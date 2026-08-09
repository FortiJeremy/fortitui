//! SD-WAN screen (spec §21-24).
//!
//! Members table (state/zone/gateway/latency/jitter/loss/SLA/traffic) and a
//! health-checks table, both from the normalized `SdwanState` model. The
//! active/content member is highlighted.

use crate::models::SdwanMember;
use crate::tui::screens::header;
use crate::tui::state::AppState;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use ratatui::Frame;

fn cell(value: String, style: Style) -> Cell<'static> {
    Cell::from(Span::styled(value, style))
}

fn ms(v: Option<f32>) -> String {
    v.map_or("--".into(), |x| format!("{x:.0}ms"))
}

fn pct(v: Option<f32>) -> String {
    v.map_or("--".into(), |x| format!("{x:.1}%"))
}

fn sla_style(sla: Option<&str>) -> Style {
    match sla {
        Some("PASS") => Style::default().fg(Color::Green),
        Some("FAIL") => Style::default().fg(Color::Red),
        _ => Style::default().fg(Color::Yellow),
    }
}

fn member_row(m: &SdwanMember, active: bool) -> Row<'static> {
    let (state, st) = match m.state.as_str() {
        "ACTIVE" => (
            "ACTIVE",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        "STANDBY" => ("STANDBY", Style::default().fg(Color::Cyan)),
        _ => ("DOWN", Style::default().fg(Color::Red)),
    };
    let name = if active {
        Cell::from(Span::styled(
            format!("{}*", m.name),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ))
    } else {
        cell(m.name.clone(), Style::default().fg(Color::Cyan))
    };
    Row::new(vec![
        name,
        cell(state.to_string(), st),
        cell(m.interface.clone(), Style::default()),
        cell(
            m.zone.clone().unwrap_or_else(|| "--".into()),
            Style::default(),
        ),
        cell(
            m.gateway.clone().unwrap_or_else(|| "--".into()),
            Style::default(),
        ),
        cell(ms(m.latency_ms), Style::default()),
        cell(ms(m.jitter_ms), Style::default()),
        cell(pct(m.packet_loss_pct), Style::default()),
        cell(
            m.sla.clone().unwrap_or_else(|| "--".into()),
            sla_style(m.sla.as_deref()),
        ),
    ])
}

pub fn draw(state: &AppState, frame: &mut Frame) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // status line
            Constraint::Length(6), // members table
            Constraint::Min(3),    // health checks table
            Constraint::Length(1), // hint
        ])
        .split(area);

    draw_status(state, frame, chunks[0]);
    draw_members(state, frame, chunks[1]);
    draw_health(state, frame, chunks[2]);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            " [Esc] back   [?] help   [r] refresh   (* = selected/active member)",
            Style::default().fg(Color::DarkGray),
        ))),
        chunks[3],
    );
}

fn draw_status(state: &AppState, frame: &mut Frame, area: Rect) {
    let line = if let Some(err) = &state.sdwan_err {
        Line::from(Span::styled(
            format!("Error loading SD-WAN: {err}"),
            Style::default().fg(Color::Red),
        ))
    } else if let Some(sd) = &state.sdwan {
        let active = sd
            .active_member
            .clone()
            .or_else(|| {
                sd.members
                    .iter()
                    .find(|m| m.state == "ACTIVE")
                    .map(|m| m.name.clone())
            })
            .unwrap_or_else(|| "--".into());
        Line::from(vec![
            Span::styled(
                format!("{} members", sd.members.len()),
                Style::default().fg(Color::DarkGray),
            ),
            Span::raw("  •  "),
            Span::styled("Active: ", Style::default().fg(Color::DarkGray)),
            Span::styled(active, Style::default().fg(Color::Green)),
        ])
    } else {
        Line::from(Span::styled(
            "Loading SD-WAN...",
            Style::default().fg(Color::Yellow),
        ))
    };
    frame.render_widget(
        Paragraph::new(line).block(Block::default().borders(Borders::NONE)),
        area,
    );
}

fn draw_members(state: &AppState, frame: &mut Frame, area: Rect) {
    let b = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled("SD-WAN MEMBERS", header()));
    let members = match &state.sdwan {
        Some(sd) => sd.members.clone(),
        None => Vec::new(),
    };
    let active = state
        .sdwan
        .as_ref()
        .and_then(|sd| sd.active_member.clone())
        .or_else(|| {
            members
                .iter()
                .find(|m| m.state == "ACTIVE")
                .map(|m| m.name.clone())
        });

    let header_row = Row::new(vec![
        "MEMBER", "STATE", "INTF", "ZONE", "GATEWAY", "LAT", "JITTER", "LOSS", "SLA",
    ])
    .style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );
    let rows: Vec<Row> = members
        .iter()
        .map(|m| member_row(m, active.as_deref() == Some(m.name.as_str())))
        .collect();
    let table = Table::new(rows, member_widths())
        .header(header_row)
        .block(b);
    frame.render_widget(table, area);
}

fn member_widths() -> [Constraint; 9] {
    [
        Constraint::Length(11),
        Constraint::Length(8),
        Constraint::Length(9),
        Constraint::Length(8),
        Constraint::Length(16),
        Constraint::Length(7),
        Constraint::Length(8),
        Constraint::Length(6),
        Constraint::Length(5),
    ]
}

fn draw_health(state: &AppState, frame: &mut Frame, area: Rect) {
    let b = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled("HEALTH CHECKS", header()));
    let checks = match &state.sdwan {
        Some(sd) => sd.health_checks.clone(),
        None => Vec::new(),
    };
    let header_row = Row::new(vec![
        "CHECK", "MEMBER", "LATENCY", "JITTER", "LOSS", "STATUS",
    ])
    .style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );
    let rows: Vec<Row> = checks
        .iter()
        .map(|c| {
            let status = c.status.as_deref().unwrap_or("--");
            let st = match status {
                "PASS" => Style::default().fg(Color::Green),
                "FAIL" => Style::default().fg(Color::Red),
                _ => Style::default().fg(Color::Yellow),
            };
            Row::new(vec![
                cell(c.name.clone(), Style::default().fg(Color::Cyan)),
                cell(c.member.clone(), Style::default()),
                cell(ms(c.latency_ms), Style::default()),
                cell(ms(c.jitter_ms), Style::default()),
                cell(pct(c.packet_loss_pct), Style::default()),
                cell(status.to_string(), st),
            ])
        })
        .collect();
    let table = Table::new(
        rows,
        [
            Constraint::Length(18),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(8),
        ],
    )
    .header(header_row)
    .block(b);
    frame.render_widget(table, area);
}
