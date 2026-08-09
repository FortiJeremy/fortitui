//! Dashboard screen (spec §15, §58).
//!
//! Four summary panels (SYSTEM, WAN/SD-WAN, VPN, ROUTING) plus a situational
//! awareness strip that surfaces abnormalities without visiting every screen.

use crate::tui::screens::{header, tag};
use crate::tui::state::AppState;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

fn panel(title: &str) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(title.to_string(), header()))
}

/// A two-column label/value line.
fn kv<'a>(label: &str, value: String, style: Style) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("{label:<10}"), Style::default().fg(Color::DarkGray)),
        Span::styled(value, style),
    ])
}

fn fmt_uptime(secs: u64) -> String {
    let d = secs / 86400;
    let h = (secs % 86400) / 3600;
    format!("{d}d {h}h")
}

pub fn draw(state: &AppState, profile: &str, frame: &mut Frame) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // awareness strip
            Constraint::Min(3),
            Constraint::Length(1), // hint bar
        ])
        .split(area);

    draw_awareness(state, profile, frame, chunks[0]);
    draw_panels(state, frame, chunks[1]);
    draw_hint(frame, chunks[2]);
}

fn draw_awareness(state: &AppState, profile: &str, frame: &mut Frame, area: Rect) {
    // Derive simple health lines for situational awareness (spec §58).
    let gray = Style::default().fg(Color::DarkGray);
    let green = Style::default().fg(Color::Green);
    let yellow = Style::default().fg(Color::Yellow);
    let mut spans: Vec<Span> = Vec::new();
    if let Some(s) = &state.system {
        let ok = s.cpu_percent < 90.0 && s.memory_percent < 85.0;
        spans.push(Span::styled("System: ", gray));
        spans.push(Span::styled(
            if ok { "healthy" } else { "degraded" },
            if ok { green } else { yellow },
        ));
        spans.push(Span::raw("   "));
    }
    if let Some(sd) = &state.sdwan {
        let deg = sd
            .members
            .iter()
            .filter(|m| m.state != "ACTIVE" && m.state != "STANDBY")
            .count();
        spans.push(Span::styled("SD-WAN: ", gray));
        spans.push(Span::styled(
            if deg == 0 { "healthy" } else { "member down" },
            if deg == 0 { green } else { yellow },
        ));
        spans.push(Span::raw("   "));
    }
    if let Some(bgp) = &state.bgp {
        let est = bgp
            .neighbors
            .iter()
            .filter(|n| n.state == "ESTABLISHED")
            .count();
        let total = bgp.neighbors.len();
        let label = format!("{est}/{total} established");
        spans.push(Span::styled("BGP: ", gray));
        spans.push(Span::styled(
            label,
            if total > 0 && est == total {
                green
            } else {
                yellow
            },
        ));
        spans.push(Span::raw("   "));
    }
    let line = Line::from(spans);
    frame.render_widget(
        Paragraph::new(line).block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(format!("FORTIGATE — {profile}"), header())),
        ),
        area,
    );
}

fn draw_panels(state: &AppState, frame: &mut Frame, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(cols[0]);
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(cols[1]);

    draw_system(state, frame, left[0]);
    draw_sdwan(state, frame, left[1]);
    draw_vpn(state, frame, right[0]);
    draw_routing(state, frame, right[1]);
}

fn draw_system(state: &AppState, frame: &mut Frame, area: Rect) {
    let b = panel("SYSTEM");
    let inner = b.inner(area);
    frame.render_widget(b, area);
    let mut lines = Vec::new();
    match &state.system {
        Some(s) => {
            lines.push(kv(
                "Hostname",
                s.hostname.clone(),
                Style::default().fg(Color::Cyan),
            ));
            lines.push(kv(
                "Model",
                format!("{} {}", s.model, s.fortios),
                Style::default(),
            ));
            lines.push(kv("Serial", s.serial.clone(), Style::default()));
            lines.push(kv("Uptime", fmt_uptime(s.uptime_secs), Style::default()));
            lines.push(kv(
                "CPU",
                format!("{:.0}%", s.cpu_percent),
                Style::default(),
            ));
            lines.push(kv(
                "Memory",
                format!("{:.0}%", s.memory_percent),
                Style::default(),
            ));
            lines.push(kv("Sessions", s.sessions.to_string(), Style::default()));
        }
        None => {
            lines.push(Line::from(Span::styled(
                if state.system_err.is_some() {
                    "Error loading system"
                } else {
                    "Loading system..."
                },
                Style::default().fg(Color::Yellow),
            )));
            if let Some(e) = &state.system_err {
                lines.push(Line::from(Span::styled(
                    e.clone(),
                    Style::default().fg(Color::Red),
                )));
            }
        }
    }
    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::default())
            .wrap(Wrap { trim: true }),
        inner,
    );
}

fn draw_sdwan(state: &AppState, frame: &mut Frame, area: Rect) {
    let b = panel("WAN / SD-WAN");
    let inner = b.inner(area);
    frame.render_widget(b, area);
    let mut lines: Vec<Line> = Vec::new();
    match &state.sdwan {
        Some(sd) if !sd.members.is_empty() => {
            for m in &sd.members {
                let (st, stt) = if m.state == "ACTIVE" || m.state == "STANDBY" {
                    tag(true)
                } else {
                    tag(false)
                };
                lines.push(Line::from(vec![
                    Span::raw(format!("{:<10} ", m.name)),
                    Span::styled(format!("{:<8} ", st), stt),
                    Span::styled(
                        format!(
                            "lat={:<6} loss={:<5} sla={}",
                            m.latency_ms.map_or("--".into(), |v| format!("{v:.0}ms")),
                            m.packet_loss_pct
                                .map_or("--".into(), |v| format!("{v:.1}%")),
                            m.sla.as_deref().unwrap_or("--"),
                        ),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));
            }
        }
        Some(_) => lines.push(Line::from(Span::styled(
            "SD-WAN enabled but no members",
            Style::default().fg(Color::Yellow),
        ))),
        None => lines.push(Line::from(Span::styled(
            if state.sdwan_err.is_some() {
                "Error loading SD-WAN"
            } else {
                "Loading SD-WAN..."
            },
            Style::default().fg(Color::Yellow),
        ))),
    }
    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::default())
            .wrap(Wrap { trim: true }),
        inner,
    );
}

fn draw_vpn(state: &AppState, frame: &mut Frame, area: Rect) {
    let b = panel("VPN");
    let inner = b.inner(area);
    frame.render_widget(b, area);
    let mut lines: Vec<Line> = Vec::new();
    match &state.vpn {
        Some(tunnels) if !tunnels.is_empty() => {
            let up = tunnels
                .iter()
                .filter(|t| t.phase1_state.as_deref() == Some("up"))
                .count();
            lines.push(kv(
                "IPsec",
                format!("{up}/{} up", tunnels.len()),
                Style::default(),
            ));
            for t in tunnels.iter().take(4) {
                let upn = t.phase1_state.as_deref() == Some("up");
                let (st, stt) = tag(upn);
                lines.push(Line::from(vec![
                    Span::raw(format!("  {:<18} ", t.name)),
                    Span::styled(format!("{:<8}", st), stt),
                    Span::styled(
                        format!("  {}", t.ike_version.as_deref().unwrap_or("--")),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));
            }
        }
        Some(_) => lines.push(Line::from(Span::styled(
            "No IPsec tunnels",
            Style::default().fg(Color::Yellow),
        ))),
        None => lines.push(Line::from(Span::styled(
            if state.vpn_err.is_some() {
                "Error loading VPN"
            } else {
                "Loading VPN..."
            },
            Style::default().fg(Color::Yellow),
        ))),
    }
    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::default())
            .wrap(Wrap { trim: true }),
        inner,
    );
}

fn draw_routing(state: &AppState, frame: &mut Frame, area: Rect) {
    let b = panel("ROUTING");
    let inner = b.inner(area);
    frame.render_widget(b, area);
    let mut lines: Vec<Line> = Vec::new();
    let routes = state.routes.clone().unwrap_or_default();
    lines.push(kv(
        "IPv4 routes",
        routes.len().to_string(),
        Style::default(),
    ));
    let bgp = match &state.bgp {
        Some(b) => b,
        None => {
            lines.push(Line::from(Span::styled(
                "Loading BGP...",
                Style::default().fg(Color::Yellow),
            )));
            frame.render_widget(
                Paragraph::new(lines)
                    .block(Block::default())
                    .wrap(Wrap { trim: true }),
                inner,
            );
            return;
        }
    };
    let est = bgp
        .neighbors
        .iter()
        .filter(|n| n.state == "ESTABLISHED")
        .count();
    lines.push(kv(
        "BGP",
        format!("{est}/{} established", bgp.neighbors.len()),
        Style::default(),
    ));
    for n in bgp.neighbors.iter().take(3) {
        let (st, stt) = tag(n.state == "ESTABLISHED");
        lines.push(Line::from(vec![
            Span::raw(format!("  {:<16} ", n.address)),
            Span::styled(format!("{:<8}", st), stt),
        ]));
    }
    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::default())
            .wrap(Wrap { trim: true }),
        inner,
    );
}

fn draw_hint(frame: &mut Frame, area: Rect) {
    let hint = Line::from(Span::styled(
        " [q] quit   [?] help   [r] refresh   [Esc] back   [i] interfaces   [s] SD-WAN   [v] VPN   [g] routing   [d] diagnostics",
        Style::default().fg(Color::DarkGray),
    ));
    frame.render_widget(Paragraph::new(hint), area);
}
