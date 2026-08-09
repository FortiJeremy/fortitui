//! Firewall policy screen (spec §35) — placeholder until implemented.

use crate::tui::screens::header;
use crate::tui::state::AppState;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn draw(_state: &AppState, frame: &mut Frame) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(area);
    let b = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled("FIREWALL POLICIES", header()));
    frame.render_widget(b, chunks[0]);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "Firewall policies — coming soon",
            Style::default().fg(Color::Yellow),
        ))),
        chunks[1],
    );
}
