//! Firewall / session normalized data.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FirewallPolicy {
    pub id: u32,
    pub name: String,
    pub source: Vec<String>,
    pub destination: Vec<String>,
    pub service: Vec<String>,
    pub action: String, // ACCEPT / DENY
    pub hit_count: u64,
    pub bytes: u64,
    pub sessions: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FirewallSession {
    pub src: String,
    pub dst: String,
    pub src_port: u16,
    pub dst_port: u16,
    pub proto: String,
    pub policy: Option<u32>,
    pub interface: Option<String>,
    pub bytes: u64,
    pub packets: u64,
    pub age_secs: u64,
}
