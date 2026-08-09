//! Event / state-transition normalized data (in-memory only).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub timestamp: u64, // unix seconds
    pub description: String,
    pub severity: String, // INFO / WARNING / CRITICAL
}
