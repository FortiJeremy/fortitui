//! Backend capabilities — what a given backend can actually do.
//!
//! The TUI uses this to decide which screens/features to expose, so it never
//! assumes every backend supports every operation.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Capabilities {
    pub available: BTreeSet<String>,
}

impl Capabilities {
    pub fn has(&self, cap: &str) -> bool {
        self.available.contains(cap)
    }
}

/// Capability identifiers shared by backends.
pub mod caps {
    pub const SYSTEM: &str = "system";
    pub const INTERFACES: &str = "interfaces";
    pub const SDWAN: &str = "sdwan";
    pub const IPSEC: &str = "ipsec";
    pub const ROUTING: &str = "routing";
    pub const BGP: &str = "bgp";
    pub const OSPF: &str = "ospf";
    pub const FIREWALL: &str = "firewall";
    pub const SESSIONS: &str = "sessions";
    pub const DIAGNOSTICS: &str = "diagnostics";
    pub const LOGS: &str = "logs";
}
