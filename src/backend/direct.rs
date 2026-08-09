//! The Phase 1 direct FortiGate backend.
//!
//! Connects to a single FortiGate over HTTPS and normalizes API responses into
//! the application models. Read-only.

use crate::backend::capabilities::{caps, Capabilities};
use crate::backend::traits::{AddressFamily, FortiGateBackend};
use crate::config::credentials;
use crate::config::profiles::Profile;
use crate::fortigate::{endpoints, normalize, FortiGateClient};
use crate::models::{BgpState, InterfaceStatus, IpsecTunnel, Route, SdwanState, SystemStatus};
use anyhow::Result;

#[derive(Clone)]
pub struct DirectBackend {
    client: FortiGateClient,
}

impl DirectBackend {
    /// Build a DirectBackend from a profile, resolving the token.
    pub async fn from_profile(profile: Profile, insecure: bool) -> Result<Self> {
        let token = profile
            .token
            .clone()
            .or_else(|| credentials::get(&profile.credential).ok())
            .ok_or_else(|| anyhow::anyhow!("no API token available for {}", profile.credential))?;

        let verify_tls = profile.verify_tls && !insecure;
        let client = FortiGateClient::new(&profile.host, profile.port, token, verify_tls)?;
        Ok(Self { client })
    }
}

impl FortiGateBackend for DirectBackend {
    async fn system_status(&self) -> Result<SystemStatus> {
        let mut s = normalize::system_status(&self.client.get(endpoints::system::STATUS).await?)?;
        let usage =
            normalize::resource_usage(&self.client.get(endpoints::system::RESOURCE_USAGE).await?)?;
        s.cpu_percent = usage.cpu_percent;
        s.memory_percent = usage.memory_percent;
        s.disk_percent = usage.disk_percent;
        s.sessions = usage.sessions;
        Ok(s)
    }

    async fn interfaces(&self) -> Result<Vec<InterfaceStatus>> {
        let raw = self.client.get(endpoints::system::INTERFACE).await?;
        normalize::interfaces(&raw)
    }

    async fn sdwan(&self) -> Result<SdwanState> {
        let members_raw = self.client.get(endpoints::sdwan::MEMBERS).await?;
        let members = normalize::sdwan_members(&members_raw)?;
        let hc_raw = self.client.get(endpoints::sdwan::HEALTH_CHECK).await?;
        let mut state = normalize::sdwan_health_check(&hc_raw)?;
        state.members = members;
        Ok(state)
    }

    async fn vpn(&self) -> Result<Vec<IpsecTunnel>> {
        let raw = self.client.get(endpoints::vpn::IPSEC).await?;
        normalize::ipsec_tunnels(&raw)
    }

    async fn routes(&self, family: AddressFamily) -> Result<Vec<Route>> {
        let (ep, fam) = match family {
            AddressFamily::Ipv4 => (endpoints::router::IPV4, "ipv4"),
            AddressFamily::Ipv6 => (endpoints::router::IPV6, "ipv6"),
        };
        let raw = self.client.get(ep).await?;
        normalize::routes(&raw, fam)
    }

    async fn bgp(&self) -> Result<BgpState> {
        // BGP neighbors may not be present on all units; return empty on error.
        match self.client.get(endpoints::router::BGP_NEIGHBORS).await {
            Ok(raw) => Ok(normalize::bgp_neighbors(&raw)?),
            Err(_) => Ok(BgpState::default()),
        }
    }

    async fn capabilities(&self) -> Result<Capabilities> {
        let mut caps = Capabilities::default();
        for c in [
            caps::SYSTEM,
            caps::INTERFACES,
            caps::SDWAN,
            caps::IPSEC,
            caps::ROUTING,
            caps::FIREWALL,
            caps::SESSIONS,
            caps::DIAGNOSTICS,
        ] {
            caps.available.insert(c.to_string());
        }
        Ok(caps)
    }
}
