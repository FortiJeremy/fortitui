//! Interfaces screen (spec §19-20).
//!
//! Stateful table of interfaces; Enter opens a detail pane with counters and a
//! live in-memory throughput graph for the selected interface.

use crate::models::{InterfaceStatus, LinkState};
use crate::tui::screens::{header, link_tag, matches_search};
use crate::tui::state::AppState;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use ratatui::Frame;

/// Level block characters for an ASCII bar.
const BARS: &[char] = &[' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

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

fn fmt_bps(bps: u64) -> String {
    if bps >= 1_000_000_000 {
        format!("{:.1} Gbps", bps as f64 / 1e9)
    } else if bps >= 1_000_000 {
        format!("{:.1} Mbps", bps as f64 / 1e6)
    } else if bps >= 1_000 {
        format!("{:.0} Kbps", bps as f64 / 1e3)
    } else {
        format!("{bps} bps")
    }
}

/// Render the last samples of a series as block bars scaled to its max.
fn sparkline(vals: Vec<u64>) -> String {
    let max = vals.iter().copied().max().unwrap_or(1).max(1);
    vals.iter()
        .map(|&v| {
            let idx = ((v as f64 / max as f64) * 8.0).round() as usize;
            BARS[idx.min(8)]
        })
        .collect()
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

    draw_status(state, frame, chunks[0]);

    if state.iface_detail {
        draw_detail(state, frame, chunks[1]);
    } else {
        draw_list(state, frame, chunks[1]);
    }

    draw_hint(state, frame, chunks[2]);
}

fn draw_status(state: &AppState, frame: &mut Frame, area: ratatui::layout::Rect) {
    let line = if let Some(err) = &state.interfaces_err {
        Line::from(Span::styled(
            format!("Error loading interfaces: {err}"),
            Style::default().fg(Color::Red),
        ))
    } else if let Some(ifs) = &state.interfaces {
        let up = ifs.iter().filter(|i| i.link_state == LinkState::Up).count();
        Line::from(Span::styled(
            format!(
                "{} interfaces, {up} up  |  ↑/↓ select, Enter detail, Esc back",
                ifs.len()
            ),
            Style::default().fg(Color::DarkGray),
        ))
    } else {
        Line::from(Span::styled(
            "Loading interfaces...",
            Style::default().fg(Color::Yellow),
        ))
    };
    frame.render_widget(
        Paragraph::new(line).block(Block::default().borders(Borders::NONE)),
        area,
    );
}

fn draw_list(state: &AppState, frame: &mut Frame, area: ratatui::layout::Rect) {
    let b = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled("INTERFACES", header()));
    let header_row = Row::new(vec![
        "NAME", "STATE", "ADDRESS", "SPEED", "RX", "TX", "ERR", "DROP",
    ])
    .style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );
    let needle = state.search.clone();
    let rows: Vec<Row> = state
        .interfaces
        .as_ref()
        .map(|ifs| {
            ifs.iter()
                .filter(|i| {
                    let alias = i.alias.as_deref().unwrap_or("");
                    matches_search(
                        &needle,
                        &[
                            i.name.as_str(),
                            alias,
                            i.ipv4.as_deref().unwrap_or(""),
                            i.ipv6.as_deref().unwrap_or(""),
                        ],
                    )
                })
                .enumerate()
                .map(|(idx, i)| row(i, idx == state.iface_sel))
                .collect()
        })
        .unwrap_or_default();
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
    frame.render_widget(table, area);
}

fn row(i: &InterfaceStatus, selected: bool) -> Row<'static> {
    let (state, st) = link_tag(i.link_state);
    let base = if selected {
        Style::default().add_modifier(Modifier::REVERSED)
    } else {
        Style::default()
    };
    Row::new(vec![
        cell(i.name.clone(), base.fg(Color::Cyan)),
        cell(state.to_string(), base.patch(st)),
        cell(i.ipv4.clone().unwrap_or_else(|| "--".into()), base),
        cell(
            i.speed_mbps
                .map(|s| {
                    if s >= 1000 {
                        format!("{} Gbps", s / 1000)
                    } else {
                        format!("{s} Mbps")
                    }
                })
                .unwrap_or_else(|| "--".into()),
            base,
        ),
        cell(fmt_bytes(i.rx_bytes), base),
        cell(fmt_bytes(i.tx_bytes), base),
        cell(i.errors.to_string(), base.patch(err_style(i.errors))),
        cell(i.drops.to_string(), base.fg(Color::DarkGray)),
    ])
}

fn err_style(errors: u64) -> Style {
    if errors > 0 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    }
}

fn draw_detail(state: &AppState, frame: &mut Frame, area: ratatui::layout::Rect) {
    let Some(ifs) = state.interfaces.as_ref() else {
        return;
    };
    let Some(i) = ifs.get(state.iface_sel) else {
        return;
    };
    let rates = state.iface_rates.get(&i.name);
    let rx_snap: Vec<u64> = rates
        .map(|r| r.history.iter().map(|h| h.1).collect())
        .unwrap_or_default();
    let tx_snap: Vec<u64> = rates
        .map(|r| r.history.iter().map(|h| h.2).collect())
        .unwrap_or_default();
    let rx_now = rates
        .and_then(|r| r.history.back())
        .map(|h| h.1)
        .unwrap_or(0);
    let tx_now = rates
        .and_then(|r| r.history.back())
        .map(|h| h.2)
        .unwrap_or(0);

    let b = Block::default().borders(Borders::ALL).title(Span::styled(
        format!("{} — DETAIL", i.name.to_uppercase()),
        header(),
    ));
    let inner = b.inner(area);
    frame.render_widget(b, area);

    let kv = |k: &str, v: String| {
        Line::from(vec![
            Span::styled(format!("{k:<10}"), Style::default().fg(Color::DarkGray)),
            Span::styled(v, Style::default()),
        ])
    };
    let mut lines: Vec<Line> = Vec::new();
    lines.push(kv(
        "State",
        if i.link_state == LinkState::Up {
            "UP".into()
        } else {
            "DOWN".into()
        },
    ));
    lines.push(kv("IPv4", i.ipv4.clone().unwrap_or_else(|| "--".into())));
    lines.push(kv("IPv6", i.ipv6.clone().unwrap_or_else(|| "--".into())));
    lines.push(kv(
        "Speed",
        i.speed_mbps
            .map(|s| {
                if s >= 1000 {
                    format!("{} Gbps", s / 1000)
                } else {
                    format!("{s} Mbps")
                }
            })
            .unwrap_or_else(|| "--".into()),
    ));
    lines.push(kv(
        "Duplex",
        i.duplex.clone().unwrap_or_else(|| "--".into()),
    ));
    lines.push(kv("MTU", i.mtu.map_or("--".into(), |m| m.to_string())));
    lines.push(kv("RX bytes", fmt_bytes(i.rx_bytes)));
    lines.push(kv("TX bytes", fmt_bytes(i.tx_bytes)));
    lines.push(kv("Errors", i.errors.to_string()));
    lines.push(kv("Drops", i.drops.to_string()));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("RX now  {}", fmt_bps(rx_now)),
        Style::default().fg(Color::Green),
    )));
    lines.push(Line::from(Span::styled(
        format!("  {}", sparkline(rx_snap)),
        Style::default().fg(Color::Green),
    )));
    lines.push(Line::from(Span::styled(
        format!("TX now  {}", fmt_bps(tx_now)),
        Style::default().fg(Color::Cyan),
    )));
    lines.push(Line::from(Span::styled(
        format!("  {}", sparkline(tx_snap)),
        Style::default().fg(Color::Cyan),
    )));
    frame.render_widget(Paragraph::new(lines).block(Block::default()), inner);
}

fn draw_hint(state: &AppState, frame: &mut Frame, area: ratatui::layout::Rect) {
    let txt = if state.iface_detail {
        " [Enter] list   [Esc] back  "
    } else {
        " [↑↓]/[Enter] select/view   [i] interfaces   [?] help"
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            txt,
            Style::default().fg(Color::DarkGray),
        ))),
        area,
    );
}

#[cfg(test)]
mod tests {
    use super::sparkline;
    const LEVELS: &[char] = &[' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    #[test]
    fn sparkline_length_matches_input() {
        assert_eq!(sparkline(vec![0, 1, 2, 3, 4, 5]).chars().count(), 6);
        assert_eq!(sparkline(vec![42; 100]).chars().count(), 100);
        assert!(sparkline(vec![]).is_empty());
    }

    #[test]
    fn sparkline_uses_valid_levels() {
        let s = sparkline(vec![0, 10, 50, 100, 200, 1000]);
        assert!(s.chars().all(|c| LEVELS.contains(&c)));
    }
}
