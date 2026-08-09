//! Module docs: define the FortiGate client API endpoints used, per the
//! FortiOS 8.0 monitor API reference (drop/FortiOS_8/).

/// System endpoints (monitor/system.json).
pub mod system {
    /// GET /system/status — model, hostname, serial, version.
    pub const STATUS: &str = "/system/status";
    /// GET /system/resource/usage — cpu, mem, disk, session counters.
    pub const RESOURCE_USAGE: &str = "/system/resource/usage";
    /// GET /system/interface — keyed map of interfaces with counters.
    pub const INTERFACE: &str = "/system/interface";
}

/// SD-WAN endpoints (monitor/virtual-wan.json).
pub mod sdwan {
    /// GET /virtual-wan/members — list of SD-WAN members w/ live stats.
    pub const MEMBERS: &str = "/virtual-wan/members";
    /// GET /virtual-wan/health-check — health checks per member.
    pub const HEALTH_CHECK: &str = "/virtual-wan/health-check";
    /// GET /virtual-wan/sladb — SLA database (large).
    pub const SLADB: &str = "/virtual-wan/sladb";
}

/// VPN endpoints (monitor/vpn.json).
pub mod vpn {
    /// GET /vpn/ipsec — active IPsec tunnels.
    pub const IPSEC: &str = "/vpn/ipsec";
}

/// Routing endpoints (monitor/router.json).
pub mod router {
    /// GET /router/ipv4 — IPv4 routing table.
    pub const IPV4: &str = "/router/ipv4";
    /// GET /router/ipv6 — IPv6 routing table.
    pub const IPV6: &str = "/router/ipv6";
    /// GET /router/lookup?destination=... — route lookup.
    pub const LOOKUP: &str = "/router/lookup";
    /// GET /router/bgp/neighbors — BGP neighbors.
    pub const BGP_NEIGHBORS: &str = "/router/bgp/neighbors";
    /// GET /router/sdwan/routes — SD-WAN routing/zone next-hops.
    pub const SDWAN_ROUTES: &str = "/router/sdwan/routes";
}

/// Firewall endpoints (monitor/firewall.json).
pub mod firewall {
    /// GET /firewall/policy — firewall policies with hit/byte stats.
    pub const POLICY: &str = "/firewall/policy";
    /// GET /firewall/sessions — active sessions (filterable via query).
    pub const SESSIONS: &str = "/firewall/sessions";
}
