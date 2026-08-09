//! The `FortiGateBackend` trait — the central abstraction.
//!
//! All methods are async and return normalized application models (see `models`).
//! Phase 1 implements `DirectBackend` against a single FortiGate.

use crate::backend::capabilities::Capabilities;
use crate::models::{
    BgpState, FirewallPolicy, FirewallSession, InterfaceStatus, IpsecTunnel, Route, SdwanState,
    SystemStatus,
};
use anyhow::Result;
use std::future::Future;

/// Address family for routing lookups.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressFamily {
    Ipv4,
    Ipv6,
}

/// Filter for session queries.
#[derive(Debug, Clone, Default)]
pub struct SessionFilter {
    pub src: Option<String>,
    pub dst: Option<String>,
    pub proto: Option<String>,
    pub policy: Option<u32>,
}

/// RPITIT with `+ Send` so background refresh tasks (which `tokio::spawn` with a
/// `Send` bound) can await these futures from independent asynchronous tasks.
pub trait FortiGateBackend: Send + Sync {
    fn system_status(&self) -> impl Future<Output = Result<SystemStatus>> + Send;
    fn interfaces(&self) -> impl Future<Output = Result<Vec<InterfaceStatus>>> + Send;
    fn sdwan(&self) -> impl Future<Output = Result<SdwanState>> + Send;
    fn vpn(&self) -> impl Future<Output = Result<Vec<IpsecTunnel>>> + Send;
    fn routes(&self, family: AddressFamily) -> impl Future<Output = Result<Vec<Route>>> + Send;
    fn bgp(&self) -> impl Future<Output = Result<BgpState>> + Send;
    /// Active firewall sessions, optionally filtered.
    fn sessions(
        &self,
        filter: SessionFilter,
    ) -> impl Future<Output = Result<Vec<FirewallSession>>> + Send;
    /// Firewall policies with operational counters (hit/bytes/sessions).
    fn policies(&self) -> impl Future<Output = Result<Vec<FirewallPolicy>>> + Send;
    /// Best-route lookup for a destination (IPv4 or IPv6).
    fn route_lookup(&self, destination: &str) -> impl Future<Output = Result<Vec<Route>>> + Send;
    fn capabilities(&self) -> impl Future<Output = Result<Capabilities>> + Send;
}
