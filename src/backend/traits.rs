//! The `FortiGateBackend` trait — the central abstraction.
//!
//! All methods are async and return normalized application models (see `models`).
//! Phase 1 implements `DirectBackend` against a single FortiGate.

use crate::backend::capabilities::Capabilities;
use crate::models::{BgpState, InterfaceStatus, IpsecTunnel, Route, SdwanState, SystemStatus};
use anyhow::Result;

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

#[allow(async_fn_in_trait)]
pub trait FortiGateBackend: Send + Sync {
    async fn system_status(&self) -> Result<SystemStatus>;
    async fn interfaces(&self) -> Result<Vec<InterfaceStatus>>;
    async fn sdwan(&self) -> Result<SdwanState>;
    async fn vpn(&self) -> Result<Vec<IpsecTunnel>>;
    async fn routes(&self, family: AddressFamily) -> Result<Vec<Route>>;
    async fn bgp(&self) -> Result<BgpState>;
    async fn capabilities(&self) -> Result<Capabilities>;
}
