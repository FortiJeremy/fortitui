//! TUI screens. Each screen renders only from the normalized `AppState`.
//!
//! State classification (spec §59) is represented by a symbol + color + text so
//! it remains readable without color.

pub mod dashboard;
pub mod diagnostics;
pub mod events;
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
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
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
    Events,
    Help,
    Diagnostics,
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
            Screen::Events => "Events",
            Screen::Help => "Help",
            Screen::Diagnostics => "Diagnostics",
        }
    }
}

/// Render the active screen into the frame.
///
/// `help_subject` is the screen the help is being shown for (contextual help,
/// spec §63); it is only consulted when `screen == Screen::Help`.
pub fn draw(
    screen: Screen,
    state: &AppState,
    profile: &str,
    help_subject: Screen,
    frame: &mut Frame,
) {
    match screen {
        Screen::Dashboard => dashboard::draw(state, profile, frame),
        Screen::Interfaces => interfaces::draw(state, frame),
        Screen::System => system::draw(state, frame),
        Screen::Sdwan => sdwan::draw(state, frame),
        Screen::Sessions => sessions::draw(state, frame),
        Screen::Policies => policies::draw(state, frame),
        Screen::Ipsec => vpn::draw(state, frame),
        Screen::Routing => routing::draw(state, frame),
        Screen::Events => events::draw(state, frame),
        Screen::Help => help::draw(help_subject, frame),
        Screen::Diagnostics => diagnostics::draw(frame),
    }

    // Overlays drawn on top of any screen (spec §17, §64).
    draw_overlays(state, frame);
}

/// Render the search/filter bar and command palette overlays.
fn draw_overlays(state: &AppState, frame: &mut Frame) {
    if state.search_mode {
        draw_search_bar(state, frame);
    }
    if state.palette {
        draw_palette(state, frame);
    }
}

/// A bottom-anchored "SEARCH: <query>" bar (D1, spec §64).
fn draw_search_bar(state: &AppState, frame: &mut Frame) {
    let area = frame.area();
    let bar = Rect::new(0, area.height.saturating_sub(1), area.width, 1);
    frame.render_widget(Clear, bar);
    let line = Line::from(vec![
        Span::styled(" SEARCH: ", Style::default().fg(Color::Yellow)),
        Span::styled(state.search.clone(), Style::default().fg(Color::White)),
        Span::styled(
            "  [Enter] apply   [Esc] cancel",
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    frame.render_widget(Paragraph::new(line), bar);
}

/// A centered command-palette modal listing filtered commands (D2, spec §17).
fn draw_palette(state: &AppState, frame: &mut Frame) {
    let area = frame.area();
    let w = area.width.min(60);
    let h = area.height.min(16);
    let x = (area.width.saturating_sub(w)) / 2;
    let y = area.height.saturating_sub(h) / 2;
    let modal = Rect::new(x, y, w, h);

    frame.render_widget(Clear, modal);
    let commands = crate::tui::app::palette_commands_for_draw(&state.search);
    let sel = state.palette_sel.min(commands.len().saturating_sub(1));

    let b = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled("COMMAND", Style::default().fg(Color::Yellow)));
    let inner = b.inner(modal);
    frame.render_widget(b, modal);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(vec![
        Span::styled("> ", Style::default().fg(Color::Green)),
        Span::styled(state.search.clone(), Style::default()),
    ]));
    lines.push(Line::from(""));
    for (i, label) in commands.iter().enumerate() {
        if i > h.saturating_sub(4) as usize {
            break;
        }
        let highlighted = i == sel;
        let st = if highlighted {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        lines.push(Line::from(Span::styled(format!("  {label}"), st)));
    }
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}

/// Case-insensitive substring test used to filter tables (D1).
pub(crate) fn matches_search(needle: &str, hay: &[&str]) -> bool {
    let n = needle.trim().to_lowercase();
    if n.is_empty() {
        return true;
    }
    hay.iter().any(|h| h.to_lowercase().contains(&n))
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
