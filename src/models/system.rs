//! System-level normalized data.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemStatus {
    pub hostname: String,
    pub serial: String,
    pub model: String,
    pub fortios: String,
    pub build: String,
    pub uptime_secs: u64,
    pub cpu_percent: f32,
    pub memory_percent: f32,
    pub disk_percent: f32,
    pub sessions: u64,
    pub vdoms: u32,
    pub ha_state: Option<String>,
}
