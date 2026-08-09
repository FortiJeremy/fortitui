//! Shared, in-memory application state.
//!
//! Holds the most recent normalized data per domain plus a small rolling list
//! of detected events. A `SharedState` (`Arc<Mutex<..>>`) is shared between
//! background refresh tasks (which write) and the render loop (which reads).
//!
//! This state is deliberately in-memory only (spec §36, §61) — no database.

use crate::models::{
    BgpState, Event, FirewallPolicy, FirewallSession, InterfaceStatus, IpsecTunnel, Route,
    SdwanState, SystemStatus,
};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Default)]
pub struct AppState {
    // Latest snapshot per domain. Error strings hold a friendly message when
    // the last refresh for that domain failed (spec §39).
    pub system: Option<SystemStatus>,
    pub system_err: Option<String>,
    pub interfaces: Option<Vec<InterfaceStatus>>,
    pub interfaces_err: Option<String>,
    pub sdwan: Option<SdwanState>,
    pub sdwan_err: Option<String>,
    pub vpn: Option<Vec<IpsecTunnel>>,
    pub vpn_err: Option<String>,
    pub routes: Option<Vec<Route>>,
    pub routes_err: Option<String>,
    pub bgp: Option<BgpState>,
    pub bgp_err: Option<String>,
    pub sessions: Option<Vec<FirewallSession>>,
    pub sessions_err: Option<String>,
    pub policies: Option<Vec<FirewallPolicy>>,
    pub policies_err: Option<String>,
    /// Rolling in-memory event/state-transition log (spec §36).
    pub events: Vec<Event>,
    /// Unix seconds of the last completed refresh.
    pub last_refresh: Option<u64>,
}

impl AppState {
    /// Append an event, trimming to a bounded window (default 200).
    pub fn push_event(&mut self, event: Event, max: usize) {
        self.events.push(event);
        if self.events.len() > max {
            self.events.drain(..self.events.len() - max);
        }
    }
}

/// Thread-safe handle shared between refresh tasks and the renderer.
pub type SharedState = Arc<Mutex<AppState>>;

/// Wrap a fresh [`AppState`] in a shared handle.
pub fn shared() -> SharedState {
    Arc::new(Mutex::new(AppState::default()))
}
