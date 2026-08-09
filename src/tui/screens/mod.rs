//! TUI screens. Each screen renders only from the normalized `AppState`.
//!
//! State classification (spec §59) is represented by a symbol + color + text so
//! it remains readable without color.

pub mod dashboard;
pub mod help;
pub mod interfaces;
pub mod policies;
pub mod routing;
pub mod sdwan;
pub mod sessions;
pub mod system;
pub mod vpn;

use crate::models::LinkState;
use crate::tui::state::AppState;
use ratatui::style::{Color, Modifier, Style};
use ratatui::Frame;

/// The currently active screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Screen {
    #[default]
    Dashboard,
    Interfaces,
    System,
    Sdwan,
    Sessions,
    Policies,
    Ipsec,
    Routing,
    Help,
}

impl Screen {
    pub fn title(self) -> &'static str {
        match self {
            Screen::Dashboard => "Dashboard",
            Screen::Interfaces => "Interfaces",
            Screen::System => "System",
            Screen::Sdwan => "SD-WAN",
            Screen::Sessions => "Sessions",
            Screen::Policies => "Firewall Policies",
            Screen::Ipsec => "IPsec",
            Screen::Routing => "Routing",
            Screen::Help => "Help",
        }
    }
}

/// Render the active screen into the frame.
pub fn draw(screen: Screen, state: &AppState, profile: &str, frame: &mut Frame) {
    match screen {
        Screen::Dashboard => dashboard::draw(state, profile, frame),
        Screen::Interfaces => interfaces::draw(state, frame),
        Screen::System => system::draw(state, frame),
        Screen::Sdwan => sdwan::draw(state, frame),
        Screen::Sessions => sessions::draw(state, frame),
        Screen::Policies => policies::draw(state, frame),
        Screen::Ipsec => vpn::draw(state, frame),
        Screen::Routing => routing::draw(state, frame),
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

/// Map a `LinkState` to a tag + style.
pub(crate) fn link_tag(link: LinkState) -> (&'static str, Style) {
    match link {
        LinkState::Up => ("● UP", Style::default().fg(Color::Green)),
        LinkState::Down => ("✕ DOWN", Style::default().fg(Color::Red)),
        LinkState::Unknown => ("? UNKNOWN", Style::default().fg(Color::Yellow)),
    }
}

/// Bold label style for section headers.
pub(crate) fn header() -> Style {
    Style::default().add_modifier(Modifier::BOLD)
}
