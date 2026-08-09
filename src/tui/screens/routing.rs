//! Routing / BGP screen (spec §26-28).
//!
//! IPv4 + IPv6 routes in one table (each row carries its family) plus a BGP
//! neighbors table. BGP may be empty on units with no BGP (PROD returns HTTP
//! 500 → backend degrades to empty) — handled gracefully.

use crate::models::Route;
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

fn route_row(r: &Route) -> Row<'static> {
    let fam = if r.family == "ipv6" {
        ("v6", Style::default().fg(Color::Magenta))
    } else {
        ("v4", Style::default().fg(Color::Cyan))
    };
    let proto = match r.protocol.as_str() {
        "connected" => Style::default().fg(Color::Green),
        "static" => Style::default().fg(Color::Yellow),
        "sd-wan" => Style::default().fg(Color::Cyan),
        _ => Style::default(),
    };
    Row::new(vec![
        cell(r.prefix.clone(), Style::default()),
        cell(fam.0.to_string(), fam.1),
        cell(r.protocol.clone(), proto),
        cell(
            r.next_hop.clone().unwrap_or_else(|| "--".into()),
            Style::default().fg(Color::White),
        ),
        cell(
            r.interface.clone().unwrap_or_else(|| "--".into()),
            Style::default(),
        ),
        cell(
            r.distance.map_or("--".into(), |d| d.to_string()),
            Style::default().fg(Color::DarkGray),
        ),
    ])
}

pub fn draw(state: &AppState, frame: &mut Frame) {
    let area = frame.area();

    // Dynamic layout: input field and/or lookup results appear only when active.
    let mut cons: Vec<Constraint> = vec![Constraint::Length(1)]; // status
    if state.input_mode {
        cons.push(Constraint::Length(3)); // input field
    }
    if state.lookup.is_some() || state.lookup_err.is_some() {
        cons.push(Constraint::Length(5)); // lookup results
    }
    cons.push(Constraint::Min(6)); // routes table
    cons.push(Constraint::Min(3)); // BGP table
    cons.push(Constraint::Length(1)); // hint

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(cons)
        .split(area);
    let mut i = 0;
    draw_status(state, frame, chunks[i]);
    i += 1;
    if state.input_mode {
        draw_lookup_input(state, frame, chunks[i]);
        i += 1;
    }
    if state.lookup.is_some() || state.lookup_err.is_some() {
        draw_lookup_results(state, frame, chunks[i]);
        i += 1;
    }
    draw_routes(state, frame, chunks[i]);
    i += 1;
    draw_bgp(state, frame, chunks[i]);
    i += 1;
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            " [Esc] back   [?] help   [l] route lookup   [r] refresh",
            Style::default().fg(Color::DarkGray),
        ))),
        chunks[i],
    );
}

fn draw_lookup_input(state: &AppState, frame: &mut Frame, area: ratatui::layout::Rect) {
    let b = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled("ROUTE LOOKUP", header()));
    let line = Line::from(vec![
        Span::styled("Destination: ", Style::default().fg(Color::Cyan)),
        Span::styled(
            format!("{}▌", state.input),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    frame.render_widget(Paragraph::new(line).block(b), area);
}

fn draw_lookup_results(state: &AppState, frame: &mut Frame, area: ratatui::layout::Rect) {
    let b = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled("LOOKUP RESULT", header()));
    let inner = b.inner(area);
    frame.render_widget(b, area);

    let mut lines: Vec<Line> = Vec::new();
    if let Some(err) = &state.lookup_err {
        lines.push(Line::from(Span::styled(
            format!("Error: {err}"),
            Style::default().fg(Color::Red),
        )));
    } else if let Some(routes) = &state.lookup {
        if routes.is_empty() {
            lines.push(Line::from(Span::styled(
                "No route found for this destination.",
                Style::default().fg(Color::Yellow),
            )));
        } else {
            for r in routes {
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{:<20}", r.prefix),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(format!("{:<10}", r.protocol), Style::default()),
                    Span::raw("→ "),
                    Span::styled(
                        r.next_hop.clone().unwrap_or_else(|| "--".into()),
                        Style::default().fg(Color::White),
                    ),
                    Span::raw(" via "),
                    Span::styled(
                        r.interface.clone().unwrap_or_else(|| "--".into()),
                        Style::default(),
                    ),
                ]));
            }
        }
    }
    frame.render_widget(Paragraph::new(lines).block(Block::default()), inner);
}

fn draw_status(state: &AppState, frame: &mut Frame, area: Rect) {
    let v4 = state.routes.as_ref().map(|v| v.len()).unwrap_or(0);
    let v6 = state.routes6.as_ref().map(|v| v.len()).unwrap_or(0);
    let line = if state.routes.is_none() && state.routes6.is_none() {
        Line::from(Span::styled(
            "Loading routes...",
            Style::default().fg(Color::Yellow),
        ))
    } else {
        Line::from(vec![
            Span::styled(format!("{v4} IPv4"), Style::default().fg(Color::Cyan)),
            Span::raw("  •  "),
            Span::styled(format!("{v6} IPv6"), Style::default().fg(Color::Magenta)),
            Span::raw("  •  "),
            Span::styled("BGP: ", Style::default().fg(Color::DarkGray)),
            Span::styled(bgp_summary(state), Style::default().fg(Color::Green)),
        ])
    };
    frame.render_widget(
        Paragraph::new(line).block(Block::default().borders(Borders::NONE)),
        area,
    );
}

fn bgp_summary(state: &AppState) -> String {
    match &state.bgp {
        Some(b) => {
            let est = b
                .neighbors
                .iter()
                .filter(|n| n.state == "ESTABLISHED")
                .count();
            format!("{est}/{} established", b.neighbors.len())
        }
        None => "--".to_string(),
    }
}

fn draw_routes(state: &AppState, frame: &mut Frame, area: Rect) {
    let mut rows: Vec<Row> = Vec::new();
    if let Some(v) = &state.routes {
        rows.extend(v.iter().map(route_row));
    }
    if let Some(v) = &state.routes6 {
        rows.extend(v.iter().map(route_row));
    }

    let err = state.routes_err.as_ref().or(state.routes6_err.as_ref());
    let mut b = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled("ROUTES", header()));
    if let Some(e) = err {
        b = b.title_bottom(Span::styled(
            format!(" • {e}"),
            Style::default().fg(Color::Red),
        ));
    }
    let header_row = Row::new(vec!["PREFIX", "FAM", "PROTO", "NEXT-HOP", "INTF", "DIST"]).style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );
    let widths: [Constraint; 6] = [
        Constraint::Length(22),
        Constraint::Length(5),
        Constraint::Length(11),
        Constraint::Length(18),
        Constraint::Length(12),
        Constraint::Length(6),
    ];
    let table = Table::new(rows, widths).header(header_row).block(b);
    frame.render_widget(table, area);
}

fn draw_bgp(state: &AppState, frame: &mut Frame, area: Rect) {
    let b = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled("BGP NEIGHBORS", header()));
    let header_row = Row::new(vec!["NEIGHBOR", "AS", "STATE", "RX", "TX"]).style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );
    let rows: Vec<Row> = state
        .bgp
        .as_ref()
        .map(|bgp| {
            bgp.neighbors
                .iter()
                .map(|n| {
                    let st = if n.state == "ESTABLISHED" {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Yellow)
                    };
                    Row::new(vec![
                        cell(n.address.clone(), Style::default().fg(Color::Cyan)),
                        cell(
                            n.remote_as.map_or("--".into(), |a| a.to_string()),
                            Style::default(),
                        ),
                        cell(n.state.clone(), st),
                        cell(n.rx_prefixes.to_string(), Style::default()),
                        cell(n.tx_prefixes.to_string(), Style::default()),
                    ])
                })
                .collect()
        })
        .unwrap_or_default();
    let table = Table::new(
        rows,
        [
            Constraint::Length(18),
            Constraint::Length(8),
            Constraint::Length(14),
            Constraint::Length(8),
            Constraint::Length(8),
        ],
    )
    .header(header_row)
    .block(b);
    frame.render_widget(table, area);
}
