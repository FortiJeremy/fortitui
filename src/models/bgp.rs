//! BGP normalized data.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BgpNeighbor {
    pub address: String,
    pub local_as: Option<u32>,
    pub remote_as: Option<u32>,
    pub state: String, // ESTABLISHED / IDLE / etc.
    pub uptime_secs: Option<u64>,
    pub rx_prefixes: u64,
    pub tx_prefixes: u64,
    pub family: String, // ipv4 / ipv6
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BgpState {
    pub router_id: Option<String>,
    pub local_as: Option<u32>,
    pub neighbors: Vec<BgpNeighbor>,
}
