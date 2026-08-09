//! System overview screen (spec §18).
//!
//! Full system health/details from the normalized `SystemStatus` model.

use crate::tui::screens::header;
use crate::tui::state::AppState;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

fn fmt_uptime(secs: u64) -> String {
    let d = secs / 86400;
    let h = (secs % 86400) / 3600;
    let m = (secs % 3600) / 60;
    format!("{d}d {h}h {m}m")
}

fn row(label: &str, value: String, val_color: Color) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label:<10}"), Style::default().fg(Color::DarkGray)),
        Span::styled(value, Style::default().fg(val_color)),
    ])
}

pub fn draw(state: &AppState, frame: &mut Frame) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(area);

    let b = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled("SYSTEM", header()));
    let inner = b.inner(chunks[0]);
    frame.render_widget(b, chunks[0]);

    let mut lines: Vec<Line> = Vec::new();
    match &state.system {
        Some(s) => {
            lines.push(row("Hostname", s.hostname.clone(), Color::Cyan));
            lines.push(row(
                "Model",
                format!("{} ({})", s.model, s.serial),
                Color::White,
            ));
            lines.push(row(
                "FortiOS",
                format!("{} build {}", s.fortios, s.build),
                Color::White,
            ));
            lines.push(row("Uptime", fmt_uptime(s.uptime_secs), Color::White));
            lines.push(row(
                "CPU",
                format!("{:.1}%", s.cpu_percent),
                pct_color(s.cpu_percent),
            ));
            lines.push(row(
                "Memory",
                format!("{:.1}%", s.memory_percent),
                pct_color(s.memory_percent),
            ));
            lines.push(row(
                "Disk",
                format!("{:.1}%", s.disk_percent),
                pct_color(s.disk_percent),
            ));
            lines.push(row("Sessions", s.sessions.to_string(), Color::White));
            lines.push(row("VDOMs", s.vdoms.to_string(), Color::White));
            lines.push(row(
                "HA",
                s.ha_state
                    .clone()
                    .unwrap_or_else(|| "standalone".to_string()),
                Color::White,
            ));
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

    frame.render_widget(Paragraph::new(lines).block(Block::default()), inner);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            " [Esc] back   [?] help   [r] refresh",
            Style::default().fg(Color::DarkGray),
        ))),
        chunks[1],
    );
}

fn pct_color(pct: f32) -> Color {
    if pct >= 90.0 {
        Color::Red
    } else if pct >= 80.0 {
        Color::Yellow
    } else {
        Color::Green
    }
}
