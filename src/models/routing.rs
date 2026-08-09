//! Routing normalized data.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Route {
    pub prefix: String, // e.g. 10.20.0.0/16
    pub family: String, // ipv4 / ipv6
    pub protocol: String,
    pub next_hop: Option<String>,
    pub interface: Option<String>,
    pub distance: Option<u32>,
    pub metric: Option<u32>,
    pub active: bool,
}
