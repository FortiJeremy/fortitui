//! TUI screens. Each screen renders only from the normalized `AppState`.
//!
//! State classification (spec §59) is represented by a symbol + color + text so
//! it remains readable without color.

pub mod dashboard;
pub mod help;

use crate::tui::state::AppState;
use ratatui::style::{Color, Modifier, Style};
use ratatui::Frame;

/// The currently active screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Screen {
    #[default]
    Dashboard,
    Help,
}

impl Screen {
    pub fn title(self) -> &'static str {
        match self {
            Screen::Dashboard => "Dashboard",
            Screen::Help => "Help",
        }
    }
}

/// Render the active screen into the frame.
pub fn draw(screen: Screen, state: &AppState, profile: &str, frame: &mut Frame) {
    match screen {
        Screen::Dashboard => dashboard::draw(state, profile, frame),
        Screen::Help => help::draw(frame),
    }
}

/// A textual state tag with an associated style, per spec §59.
pub(crate) fn tag(up: bool) -> (&'static str, Style) {
    if up {
        ("● UP", Style::default().fg(Color::Green))
    } else {
        ("✕ DOWN", Style::default().fg(Color::Red))
    }
}

/// Bold label style for section headers.
pub(crate) fn header() -> Style {
    Style::default().add_modifier(Modifier::BOLD)
}
