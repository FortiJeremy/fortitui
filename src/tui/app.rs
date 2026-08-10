//! Main TUI application: event loop, background refresh, and key dispatch.
//!
//! The backend is generic (not `dyn FortiGateBackend`) because the trait uses
//! `async fn`, which is not object-safe. Phase 1's only concrete type is
//! `DirectBackend`.

use super::detect;
use super::event::{Event, EventLoop};
use super::screens::{self, Screen};
use super::state::SharedState;
use crate::backend::traits::{AddressFamily, FortiGateBackend};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use std::time::{SystemTime, UNIX_EPOCH};

/// UI tick cadence in ms; drives background refresh + redraw (spec §37).
const TICK_MS: u64 = 2000;

/// Upper bound on the in-memory event log (spec §36).
const MAX_EVENTS: usize = 200;

/// Entry point for the interactive TUI (used by `cli` on no-subcommand).
pub async fn run<B>(backend: B, profile: String) -> Result<()>
where
    B: FortiGateBackend + Clone + Send + Sync + 'static,
{
    let mut app = App::new(backend, profile);
    app.run().await
}

/// The interactive application.
pub struct App<B: FortiGateBackend> {
    backend: B,
    state: SharedState,
    screen: Screen,
    profile: String,
    quit: bool,
    /// Screen that contextual help is currently describing (spec §63).
    help_subject: Screen,
}

impl<B: FortiGateBackend + Clone + Send + Sync + 'static> App<B> {
    pub fn new(backend: B, profile: String) -> Self {
        Self {
            backend,
            state: super::state::shared(),
            screen: Screen::Dashboard,
            profile,
            quit: false,
            help_subject: Screen::Dashboard,
        }
    }

    /// Main loop. Initializes the terminal, drives events + refresh, and
    /// restores the terminal on exit (never leaves raw/alt-screen mode).
    pub async fn run(&mut self) -> Result<()> {
        let mut terminal = ratatui::init();
        let mut events = EventLoop::new(TICK_MS);

        self.refresh();
        loop {
            match events.next().await {
                Some(Event::Key(k)) => self.dispatch(k),
                Some(Event::Tick) => self.refresh(),
                _ => {}
            }
            if self.quit {
                break;
            }
            terminal.draw(|f| {
                let state = self.state.lock().unwrap();
                screens::draw(self.screen, &state, &self.profile, self.help_subject, f);
            })?;
        }
        ratatui::restore();
        Ok(())
    }

    /// Keyboard dispatch (spec §16).
    fn dispatch(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        // Overlay/input modes swallow keys: palette > search > route lookup.
        let (palette, search, input) = {
            let st = self.state.lock().unwrap();
            (st.palette, st.search_mode, st.input_mode)
        };
        if palette {
            self.handle_palette(key);
            return;
        }
        if search {
            self.handle_search(key);
            return;
        }
        if input {
            self.handle_input(key);
            return;
        }
        match key.code {
            KeyCode::Char('q') => self.quit = true,
            KeyCode::Char('?') => self.toggle_help(),
            KeyCode::Char(':') => self.open_palette(),
            KeyCode::Char('/') => self.open_search(),
            KeyCode::Char('l') => self.handle_l(),
            KeyCode::Esc => self.handle_esc(),
            KeyCode::Char('r') => self.refresh(),
            KeyCode::Char('i') => self.navigate(Screen::Interfaces),
            KeyCode::Char('o') => self.navigate(Screen::System),
            KeyCode::Char('s') => self.navigate(Screen::Sdwan),
            KeyCode::Char('f') => self.navigate(Screen::Sessions),
            KeyCode::Char('F') => self.navigate(Screen::Policies),
            KeyCode::Char('v') => self.navigate(Screen::Ipsec),
            KeyCode::Char('g') => self.navigate(Screen::Routing),
            KeyCode::Char('e') => self.navigate(Screen::Events),
            KeyCode::Char('d') => self.navigate(Screen::Diagnostics),
            KeyCode::Up | KeyCode::Down | KeyCode::Enter => self.list_nav(key),
            _ => {}
        }
    }

    /// Toggle contextual help for the current screen (spec §63).
    fn toggle_help(&mut self) {
        if self.screen == Screen::Help {
            self.screen = self.help_subject;
        } else {
            self.help_subject = self.screen;
            self.screen = Screen::Help;
        }
    }

    /// Open the command palette (spec §17, D2).
    fn open_palette(&mut self) {
        let mut st = self.state.lock().unwrap();
        st.palette = true;
        st.palette_sel = 0;
        st.search.clear();
    }

    /// Open the search/filter bar (spec §64, D1).
    fn open_search(&mut self) {
        let mut st = self.state.lock().unwrap();
        st.search_mode = true;
        st.search.clear();
    }

    /// `l`-key behaviour: route lookup on Routing, rolling trend on SD-WAN (C7).
    fn handle_l(&mut self) {
        match self.screen {
            Screen::Routing => self.start_lookup(),
            Screen::Sdwan => {
                let mut st = self.state.lock().unwrap();
                st.sdwan_trend = !st.sdwan_trend;
            }
            _ => {}
        }
    }

    /// Esc: close any open detail/overlay, then leave help, then go home.
    fn handle_esc(&mut self) {
        let closed = {
            let mut st = self.state.lock().unwrap();
            if st.iface_detail {
                st.iface_detail = false;
                true
            } else if st.vpn_detail {
                st.vpn_detail = false;
                true
            } else if st.sdwan_trend {
                st.sdwan_trend = false;
                true
            } else {
                false
            }
        };
        if closed {
            return;
        }
        if self.screen == Screen::Help {
            self.screen = self.help_subject;
            return;
        }
        self.navigate(Screen::Dashboard);
    }

    /// Up/Down move a list selection; Enter toggles detail. Applies to the
    /// Interfaces and IPsec lists.
    fn list_nav(&mut self, key: KeyEvent) {
        match self.screen {
            Screen::Interfaces => self.interfaces_nav(key),
            Screen::Ipsec => self.vpn_nav(key),
            _ => {}
        }
    }

    /// Up/Down/Enter for the IPsec list; Enter opens the crypto detail (C9).
    fn vpn_nav(&mut self, key: KeyEvent) {
        let mut st = self.state.lock().unwrap();
        match key.code {
            KeyCode::Up => {
                if !st.vpn_detail {
                    st.vpn_sel = st.vpn_sel.saturating_sub(1);
                }
            }
            KeyCode::Down => {
                if !st.vpn_detail {
                    let len = st.vpn.as_ref().map(|v| v.len()).unwrap_or(0);
                    if len > 0 && st.vpn_sel + 1 < len {
                        st.vpn_sel += 1;
                    }
                }
            }
            KeyCode::Enter => {
                let has = st.vpn.as_ref().is_some_and(|v| !v.is_empty());
                if has {
                    st.vpn_detail = !st.vpn_detail;
                }
            }
            _ => {}
        }
    }

    /// Command palette input: type to filter, ↑/↓ to select, Enter to run.
    fn handle_palette(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.state.lock().unwrap().palette = false;
            }
            KeyCode::Enter => self.run_palette(),
            KeyCode::Up => {
                self.state.lock().unwrap().palette_sel = self.palette_sel().saturating_sub(1);
            }
            KeyCode::Down => {
                let mut st = self.state.lock().unwrap();
                let n = palette_commands(&st.search).len();
                if n > 0 && st.palette_sel + 1 < n {
                    st.palette_sel += 1;
                }
            }
            KeyCode::Backspace => {
                self.state.lock().unwrap().search.pop();
            }
            KeyCode::Char(c) if c.is_alphanumeric() || matches!(c, ' ' | '-' | '_' | '/') => {
                self.state.lock().unwrap().search.push(c);
            }
            _ => {}
        }
    }

    fn palette_sel(&self) -> usize {
        self.state.lock().unwrap().palette_sel
    }

    /// Execute the currently selected palette command.
    fn run_palette(&mut self) {
        let sel = self.state.lock().unwrap().palette_sel;
        let cmds = {
            let st = self.state.lock().unwrap();
            palette_commands(&st.search)
        };
        self.state.lock().unwrap().palette = false;
        self.state.lock().unwrap().search.clear();
        let Some(action) = cmds.get(sel).map(|(_, a)| *a) else {
            return;
        };
        match action {
            PaletteAction::Navigate(s) => self.navigate(s),
            PaletteAction::Refresh => self.refresh(),
            PaletteAction::Quit => self.quit = true,
        }
    }

    /// Search/filter input (D1). The filter is applied at render time per
    /// screen; Enter/Esc closes the bar.
    fn handle_search(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => {
                let mut st = self.state.lock().unwrap();
                st.search_mode = false;
            }
            KeyCode::Backspace => {
                self.state.lock().unwrap().search.pop();
            }
            KeyCode::Char(c)
                if c.is_alphanumeric() || matches!(c, '.' | ':' | '/' | '-' | '_' | ' ' | '*') =>
            {
                self.state.lock().unwrap().search.push(c);
            }
            _ => {}
        }
    }

    /// Up/Down move the interface selection; Enter toggles the detail pane.
    fn interfaces_nav(&mut self, key: KeyEvent) {
        if self.screen != Screen::Interfaces {
            return;
        }
        let mut st = self.state.lock().unwrap();
        match key.code {
            KeyCode::Up => {
                if !st.iface_detail {
                    st.iface_sel = st.iface_sel.saturating_sub(1);
                }
            }
            KeyCode::Down => {
                if !st.iface_detail {
                    let len = st.interfaces.as_ref().map(|v| v.len()).unwrap_or(0);
                    if len > 0 && st.iface_sel + 1 < len {
                        st.iface_sel += 1;
                    }
                }
            }
            KeyCode::Enter => {
                let has = st.interfaces.as_ref().is_some_and(|v| !v.is_empty());
                if has {
                    st.iface_detail = !st.iface_detail;
                }
            }
            _ => {}
        }
    }

    /// Route lookup input mode handler (spec §27).
    fn handle_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                let mut st = self.state.lock().unwrap();
                st.input_mode = false;
                st.input.clear();
            }
            KeyCode::Backspace => {
                self.state.lock().unwrap().input.pop();
            }
            KeyCode::Enter => {
                let dst = {
                    let mut st = self.state.lock().unwrap();
                    let d = st.input.clone();
                    st.input_mode = false;
                    st.input.clear();
                    d
                };
                self.run_lookup(&dst);
            }
            KeyCode::Char(c)
                if c.is_alphanumeric() || matches!(c, '.' | ':' | '/' | '-' | '_' | ' ') =>
            {
                self.state.lock().unwrap().input.push(c);
            }
            _ => {}
        }
    }

    /// Open the route lookup input on the Routing screen.
    fn start_lookup(&mut self) {
        if self.screen != Screen::Routing {
            return;
        }
        let mut st = self.state.lock().unwrap();
        st.input_mode = true;
        st.input.clear();
        st.lookup = None;
        st.lookup_err = None;
    }

    /// Async route lookup for a destination, writing results to shared state.
    fn run_lookup(&self, destination: &str) {
        if destination.trim().is_empty() {
            let mut st = self.state.lock().unwrap();
            st.lookup = Some(Vec::new());
            return;
        }
        let b = self.backend.clone();
        let s = self.state.clone();
        let dst = destination.to_string();
        tokio::spawn(async move {
            let r = b.route_lookup(&dst).await;
            let mut st = s.lock().unwrap();
            match r {
                Ok(v) => {
                    st.lookup = Some(v);
                    st.lookup_err = None;
                }
                Err(e) => {
                    st.lookup = None;
                    st.lookup_err = Some(e.to_string());
                }
            }
        });
    }

    /// Switch screens, immediately refreshing the target's data so it never
    /// sits empty waiting for the next tick.
    fn navigate(&mut self, screen: Screen) {
        self.screen = screen;
        self.refresh();
    }

    /// Launch independent refresh tasks so one slow endpoint never stalls the
    /// UI (spec §38). Each task writes only its own slot in the shared state.
    ///
    /// Only the current screen's data is refreshed (spec §37, §42) — e.g. the
    /// heavy `/firewall/sessions` payload is never pulled to render a dashboard.
    fn refresh(&self) {
        match self.screen {
            Screen::Dashboard => {
                self.spawn_system();
                self.spawn_sdwan();
                self.spawn_vpn();
                self.spawn_routes();
                self.spawn_bgp();
            }
            Screen::System => self.spawn_system(),
            Screen::Interfaces => self.spawn_interfaces(),
            Screen::Sdwan => self.spawn_sdwan(),
            Screen::Sessions => self.spawn_sessions(),
            Screen::Policies => self.spawn_policies(),
            Screen::Ipsec => self.spawn_vpn(),
            Screen::Routing => {
                self.spawn_routes();
                self.spawn_routes6();
                self.spawn_bgp();
            }
            // Keep detection feeds alive while viewing events.
            Screen::Events => {
                self.spawn_system();
                self.spawn_interfaces();
                self.spawn_sdwan();
            }
            Screen::Help => {}
            Screen::Diagnostics => {}
        }

        let s = self.state.clone();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        tokio::spawn(async move {
            s.lock().unwrap().last_refresh = Some(now);
        });
    }

    fn spawn_system(&self) {
        let b = self.backend.clone();
        let s = self.state.clone();
        tokio::spawn(async move {
            let r = b.system_status().await;
            let mut st = s.lock().unwrap();
            match r {
                Ok(v) => {
                    if let Some(prev) = st.system.as_ref() {
                        for e in detect::detect_system(prev, &v) {
                            st.push_event(e, MAX_EVENTS);
                        }
                    }
                    st.system = Some(v);
                }
                Err(e) => st.system_err = Some(e.to_string()),
            }
        });
    }

    fn spawn_interfaces(&self) {
        let b = self.backend.clone();
        let s = self.state.clone();
        tokio::spawn(async move {
            let r = b.interfaces().await;
            let mut st = s.lock().unwrap();
            match r {
                Ok(v) => {
                    let prev: &[crate::models::InterfaceStatus] =
                        st.interfaces.as_deref().unwrap_or(&[]);
                    let evs = detect::detect_interfaces(prev, &v);
                    for e in evs {
                        st.push_event(e, MAX_EVENTS);
                    }
                    st.update_iface_rates(&v);
                    st.interfaces = Some(v);
                }
                Err(e) => st.interfaces_err = Some(e.to_string()),
            }
        });
    }

    fn spawn_sdwan(&self) {
        let b = self.backend.clone();
        let s = self.state.clone();
        tokio::spawn(async move {
            let r = b.sdwan().await;
            let mut st = s.lock().unwrap();
            match r {
                Ok(v) => {
                    let evs = if let Some(prev) = st.sdwan.as_ref() {
                        let mut e = detect::detect_sdwan(prev, &v);
                        e.extend(detect::detect_sdwan_active(prev, &v));
                        e
                    } else {
                        Vec::new()
                    };
                    for e in evs {
                        st.push_event(e, MAX_EVENTS);
                    }
                    st.update_sdwan_history(&v);
                    st.sdwan = Some(v);
                }
                Err(e) => st.sdwan_err = Some(e.to_string()),
            }
        });
    }

    fn spawn_vpn(&self) {
        let b = self.backend.clone();
        let s = self.state.clone();
        tokio::spawn(async move {
            let r = b.vpn().await;
            let mut st = s.lock().unwrap();
            match r {
                Ok(v) => st.vpn = Some(v),
                Err(e) => st.vpn_err = Some(e.to_string()),
            }
        });
    }

    fn spawn_routes(&self) {
        let b = self.backend.clone();
        let s = self.state.clone();
        tokio::spawn(async move {
            let r = b.routes(AddressFamily::Ipv4).await;
            let mut st = s.lock().unwrap();
            match r {
                Ok(v) => st.routes = Some(v),
                Err(e) => st.routes_err = Some(e.to_string()),
            }
        });
    }

    fn spawn_bgp(&self) {
        let b = self.backend.clone();
        let s = self.state.clone();
        tokio::spawn(async move {
            let r = b.bgp().await;
            let mut st = s.lock().unwrap();
            match r {
                Ok(v) => st.bgp = Some(v),
                Err(e) => st.bgp_err = Some(e.to_string()),
            }
        });
    }

    fn spawn_routes6(&self) {
        let b = self.backend.clone();
        let s = self.state.clone();
        tokio::spawn(async move {
            let r = b.routes(AddressFamily::Ipv6).await;
            let mut st = s.lock().unwrap();
            match r {
                Ok(v) => st.routes6 = Some(v),
                Err(e) => st.routes6_err = Some(e.to_string()),
            }
        });
    }

    fn spawn_sessions(&self) {
        let b = self.backend.clone();
        let s = self.state.clone();
        tokio::spawn(async move {
            let r = b.sessions(Default::default()).await;
            let mut st = s.lock().unwrap();
            match r {
                Ok(v) => st.sessions = Some(v),
                Err(e) => st.sessions_err = Some(e.to_string()),
            }
        });
    }

    fn spawn_policies(&self) {
        let b = self.backend.clone();
        let s = self.state.clone();
        tokio::spawn(async move {
            let r = b.policies().await;
            let mut st = s.lock().unwrap();
            match r {
                Ok(v) => st.policies = Some(v),
                Err(e) => st.policies_err = Some(e.to_string()),
            }
        });
    }
}

/// Actions the command palette (D2, spec §17) can execute.
#[derive(Debug, Clone, Copy)]
enum PaletteAction {
    Navigate(Screen),
    Refresh,
    Quit,
}

/// Ordered command list for the palette, filtered case-insensitively by `q`.
fn palette_commands(q: &str) -> Vec<(&'static str, PaletteAction)> {
    use Screen::*;
    let all: &[(&str, PaletteAction)] = &[
        ("Open Dashboard", PaletteAction::Navigate(Dashboard)),
        ("Open System", PaletteAction::Navigate(System)),
        ("Open Interfaces", PaletteAction::Navigate(Interfaces)),
        ("Open SD-WAN", PaletteAction::Navigate(Sdwan)),
        ("Open IPsec VPN", PaletteAction::Navigate(Ipsec)),
        ("Open Routing / BGP", PaletteAction::Navigate(Routing)),
        ("Open Sessions", PaletteAction::Navigate(Sessions)),
        ("Open Firewall Policies", PaletteAction::Navigate(Policies)),
        ("Open Events", PaletteAction::Navigate(Events)),
        ("Open Diagnostics", PaletteAction::Navigate(Diagnostics)),
        ("Open Help", PaletteAction::Navigate(Help)),
        ("Refresh data", PaletteAction::Refresh),
        ("Quit", PaletteAction::Quit),
    ];
    let q = q.trim().to_lowercase();
    if q.is_empty() {
        all.to_vec()
    } else {
        all.iter()
            .filter(|(label, _)| label.to_lowercase().contains(&q))
            .copied()
            .collect()
    }
}

/// Palette command labels (used by the renderer for the overlay).
pub fn palette_commands_for_draw(q: &str) -> Vec<&'static str> {
    palette_commands(q).into_iter().map(|(l, _)| l).collect()
}
