//! IPsec / VPN normalized data.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IpsecTunnel {
    pub name: String,
    pub phase1_state: Option<String>,
    pub phase2_state: Option<String>,
    pub remote_gateway: Option<String>,
    pub local_gateway: Option<String>,
    pub ike_version: Option<String>,
    pub encryption: Option<String>,
    pub authentication: Option<String>,
    pub rekey_secs: Option<u32>,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub uptime_secs: Option<u64>,
    pub last_change: Option<String>,
    // PQC visibility (version-dependent)
    pub key_exchange: Option<String>,
    pub pqc_signature: Option<String>,
    pub pqc_ppk: Option<bool>,
}
