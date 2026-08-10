//! SD-WAN screen (spec §21-24).
//!
//! Members table (state/zone/gateway/latency/jitter/loss/SLA/traffic) and a
//! health-checks table, both from the normalized `SdwanState` model. The
//! active/content member is highlighted.

use crate::models::SdwanMember;
use crate::tui::screens::{header, matches_search};
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
    let trend = state.sdwan_trend;
    let chunks = if trend {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // status line
                Constraint::Length(6), // members table
                Constraint::Min(3),    // health checks table
                Constraint::Length(8), // rolling trend (C7)
                Constraint::Length(1), // hint
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // status line
                Constraint::Length(6), // members table
                Constraint::Min(3),    // health checks table
                Constraint::Length(1), // hint
            ])
            .split(area)
    };

    draw_status(state, frame, chunks[0]);
    draw_members(state, frame, chunks[1]);
    draw_health(state, frame, chunks[2]);
    if trend {
        draw_trend(state, frame, chunks[3]);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                " [Esc] close trend   [?] help   [r] refresh   [l] toggle trend",
                Style::default().fg(Color::DarkGray),
            ))),
            chunks[4],
        );
    } else {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                " [Esc] back   [?] help   [r] refresh   [l] rolling trend (C7)   (* = active)",
                Style::default().fg(Color::DarkGray),
            ))),
            chunks[3],
        );
    }
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
    let needle = state.search.clone();
    let members: Vec<SdwanMember> = members
        .into_iter()
        .filter(|m| {
            matches_search(
                &needle,
                &[
                    &m.name,
                    &m.interface,
                    m.zone.as_deref().unwrap_or(""),
                    m.gateway.as_deref().unwrap_or(""),
                    &m.state,
                ],
            )
        })
        .collect();
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
    let needle = state.search.clone();
    let checks: Vec<_> = checks
        .into_iter()
        .filter(|c| {
            matches_search(
                &needle,
                &[&c.name, &c.member, c.status.as_deref().unwrap_or("")],
            )
        })
        .collect();
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

/// Rolling in-memory performance trend for the active SD-WAN member (spec §23,
/// C7). Shows latency/jitter/loss as ASCII sparklines plus min/avg/max.
fn draw_trend(state: &AppState, frame: &mut Frame, area: Rect) {
    let b = Block::default().borders(Borders::ALL).title(Span::styled(
        "ROLLING PERFORMANCE TREND (~60 min)",
        header(),
    ));
    let inner = b.inner(area);
    frame.render_widget(b, area);

    let name = state
        .sdwan
        .as_ref()
        .and_then(|sd| sd.active_member.clone())
        .or_else(|| state.sdwan_history.keys().next().cloned())
        .unwrap_or_default();
    let hist = state.sdwan_history.get(&name);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(vec![
        Span::styled("Member: ", Style::default().fg(Color::DarkGray)),
        Span::styled(name.clone(), Style::default().fg(Color::Cyan)),
    ]));

    if let Some(h) = hist {
        let widths = inner.width.saturating_sub(24) as usize;
        let lat: Vec<f32> = h.samples.iter().map(|s| s.1).collect();
        let jit: Vec<f32> = h.samples.iter().map(|s| s.2).collect();
        let loss: Vec<f32> = h.samples.iter().map(|s| s.3).collect();
        for (label, vals) in [("latency", &lat), ("jitter", &jit), ("loss", &loss)] {
            let (mn, avg, mx) = sample_stats(vals);
            lines.push(Line::from(vec![
                Span::styled(format!("  {label:<8} "), Style::default().fg(Color::Yellow)),
                Span::styled(sparkline(vals, widths), Style::default().fg(Color::Green)),
                Span::styled(
                    format!("  min {mn}  avg {avg}  max {mx}"),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }
    } else {
        lines.push(Line::from(Span::styled(
            "  No history yet — samples accumulate on each refresh tick.",
            Style::default().fg(Color::DarkGray),
        )));
    }

    frame.render_widget(Paragraph::new(lines), inner);
}

/// `(min, avg, max)` formatted for a float series.
fn sample_stats(v: &[f32]) -> (String, String, String) {
    if v.is_empty() {
        return ("--".into(), "--".into(), "--".into());
    }
    let mn = v.iter().copied().fold(f32::INFINITY, f32::min);
    let mx = v.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let avg = v.iter().sum::<f32>() / v.len() as f32;
    let f = |x: f32| format!("{x:.1}");
    (f(mn), f(avg), f(mx))
}

/// ASCII block sparkline from a float series, bucketed into `width` bars.
fn sparkline(values: &[f32], width: usize) -> String {
    const BARS: &[char] = &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    if values.is_empty() || width == 0 {
        return String::new();
    }
    let max = values.iter().copied().fold(0.0f32, f32::max);
    let n = values.len();
    let step = 1usize.max(n.div_ceil(width));
    let mut s = String::new();
    for i in (0..n).step_by(step).take(width) {
        let v = values[i];
        let idx = if max <= 0.0 {
            0
        } else {
            ((v / max) * (BARS.len() - 1) as f32).round() as usize
        };
        s.push(BARS[idx.min(BARS.len() - 1)]);
    }
    s
}
