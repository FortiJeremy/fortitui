//! SD-WAN normalized data.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SdwanMember {
    pub name: String,
    pub interface: String,
    pub zone: Option<String>,
    pub gateway: Option<String>,
    pub state: String, // ACTIVE / STANDBY / DOWN
    pub latency_ms: Option<f32>,
    pub jitter_ms: Option<f32>,
    pub packet_loss_pct: Option<f32>,
    pub sla: Option<String>, // PASS / FAIL
    pub tx_rate_bps: Option<u64>,
    pub rx_rate_bps: Option<u64>,
    pub sessions: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SdwanHealthCheck {
    pub name: String,
    pub member: String,
    pub latency_ms: Option<f32>,
    pub jitter_ms: Option<f32>,
    pub packet_loss_pct: Option<f32>,
    pub status: Option<String>, // PASS / FAIL
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SdwanState {
    pub members: Vec<SdwanMember>,
    pub health_checks: Vec<SdwanHealthCheck>,
    pub active_member: Option<String>,
}
