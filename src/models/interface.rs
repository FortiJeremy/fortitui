//! Interface normalized data.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LinkState {
    #[default]
    Up,
    Down,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum InterfaceType {
    #[default]
    Physical,
    Vlan,
    Tunnel,
    Aggregate,
    Loopback,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InterfaceStatus {
    pub name: String,
    pub alias: Option<String>,
    pub iftype: InterfaceType,
    pub admin_state: bool,
    pub link_state: LinkState,
    pub ipv4: Option<String>,
    pub ipv6: Option<String>,
    pub speed_mbps: Option<u64>,
    pub duplex: Option<String>,
    pub mtu: Option<u32>,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_packets: u64,
    pub tx_packets: u64,
    pub errors: u64,
    pub drops: u64,
    pub rx_rate_bps: Option<u64>,
    pub tx_rate_bps: Option<u64>,
}
