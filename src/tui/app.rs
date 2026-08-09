//! Main TUI application: event loop, background refresh, and key dispatch.
//!
//! The backend is generic (not `dyn FortiGateBackend`) because the trait uses
//! `async fn`, which is not object-safe. Phase 1's only concrete type is
//! `DirectBackend`.

use super::event::{Event, EventLoop};
use super::screens::{self, Screen};
use super::state::SharedState;
use crate::backend::traits::{AddressFamily, FortiGateBackend};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use std::time::{SystemTime, UNIX_EPOCH};

/// UI tick cadence in ms; drives background refresh + redraw (spec §37).
const TICK_MS: u64 = 2000;

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
}

impl<B: FortiGateBackend + Clone + Send + Sync + 'static> App<B> {
    pub fn new(backend: B, profile: String) -> Self {
        Self {
            backend,
            state: super::state::shared(),
            screen: Screen::Dashboard,
            profile,
            quit: false,
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
                screens::draw(self.screen, &state, &self.profile, f);
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
        match key.code {
            KeyCode::Char('q') => self.quit = true,
            KeyCode::Char('?') => self.toggle(Screen::Help),
            KeyCode::Esc => {
                if self.screen != Screen::Dashboard {
                    self.screen = Screen::Dashboard;
                }
            }
            KeyCode::Char('r') => self.refresh(),
            KeyCode::Char('i')
            | KeyCode::Char('s')
            | KeyCode::Char('v')
            | KeyCode::Char('g')
            | KeyCode::Char('d')
            | KeyCode::Char('f')
            | KeyCode::Char('e') => {
                // Per-domain screens land in later increments.
            }
            _ => {}
        }
    }

    fn toggle(&mut self, screen: Screen) {
        self.screen = if self.screen == screen {
            Screen::Dashboard
        } else {
            screen
        };
    }

    /// Launch independent refresh tasks so one slow endpoint never stalls the
    /// UI (spec §38). Each task writes only its own slot in the shared state.
    fn refresh(&self) {
        self.spawn_system();
        self.spawn_sdwan();
        self.spawn_vpn();
        self.spawn_routes();
        self.spawn_bgp();

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
                Ok(v) => st.system = Some(v),
                Err(e) => st.system_err = Some(e.to_string()),
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
                Ok(v) => st.sdwan = Some(v),
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
}
