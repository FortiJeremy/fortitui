//! Shared, in-memory application state.
//!
//! Holds the most recent normalized data per domain plus a small rolling list
//! of detected events and per-interface throughput samples. A `SharedState`
//! (`Arc<Mutex<..>>`) is shared between background refresh tasks (which write)
//! and the render loop (which reads).
//!
//! This state is deliberately in-memory only (spec §36, §61) — no database.

use crate::models::{
    BgpState, Event, FirewallPolicy, FirewallSession, InterfaceStatus, IpsecTunnel, Route,
    SdwanState, SystemStatus,
};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Rolling in-memory throughput history for one interface (spec §20).
#[derive(Debug, Clone, Default)]
pub struct IfaceRates {
    pub last_rx_bytes: u64,
    pub last_tx_bytes: u64,
    pub last_ts: u64,
    /// `(unix_ts, rx_bps, tx_bps)` in arrival order; capped at [`MAX_RATE_SAMPLES`].
    pub history: VecDeque<(u64, u64, u64)>,
}

/// How many throughput samples to retain (60 × 2s tick ≈ 2 minutes).
const MAX_RATE_SAMPLES: usize = 60;

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

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
    pub routes6: Option<Vec<Route>>,
    pub routes6_err: Option<String>,
    pub bgp: Option<BgpState>,
    pub bgp_err: Option<String>,
    pub sessions: Option<Vec<FirewallSession>>,
    pub sessions_err: Option<String>,
    pub policies: Option<Vec<FirewallPolicy>>,
    pub policies_err: Option<String>,
    /// Rolling in-memory event/state-transition log (spec §36).
    pub events: Vec<Event>,
    /// Per-interface throughput history (spec §20).
    pub iface_rates: HashMap<String, IfaceRates>,
    /// Selected row on the interfaces list and whether detail is open.
    pub iface_sel: usize,
    pub iface_detail: bool,
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

    /// Compute per-interface throughput deltas from the latest bytes counters
    /// and append them to each interface's in-memory history (spec §20, §60).
    pub fn update_iface_rates(&mut self, ifaces: &[InterfaceStatus]) {
        let now = now();
        let mut rates = std::mem::take(&mut self.iface_rates);
        for i in ifaces {
            let prev = rates
                .get(&i.name)
                .map(|r| (r.last_rx_bytes, r.last_tx_bytes, r.last_ts));
            let cur = rates.entry(i.name.clone()).or_default();
            if let Some((prx, ptx, pts)) = prev {
                let dt = now.saturating_sub(pts);
                if let Some(rx_bps) = i
                    .rx_bytes
                    .saturating_sub(prx)
                    .saturating_mul(8)
                    .checked_div(dt)
                {
                    let tx_bps = i
                        .tx_bytes
                        .saturating_sub(ptx)
                        .saturating_mul(8)
                        .checked_div(dt)
                        .unwrap_or(0);
                    cur.history.push_back((now, rx_bps, tx_bps));
                    if cur.history.len() > MAX_RATE_SAMPLES {
                        cur.history.pop_front();
                    }
                }
            }
            cur.last_rx_bytes = i.rx_bytes;
            cur.last_tx_bytes = i.tx_bytes;
            cur.last_ts = now;
        }
        self.iface_rates = rates;
    }
}

/// Thread-safe handle shared between refresh tasks and the renderer.
pub type SharedState = Arc<Mutex<AppState>>;

/// Wrap a fresh [`AppState`] in a shared handle.
pub fn shared() -> SharedState {
    Arc::new(Mutex::new(AppState::default()))
}
