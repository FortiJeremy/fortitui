//! FortiGate REST API client and endpoint definitions.
//!
//! `DirectBackend` uses this to talk to a FortiGate over `/api/v2/monitor`,
//! then normalizes raw JSON into application models (see `crate::models`).

pub mod client;
pub mod endpoints;
pub mod normalize;

pub use client::FortiGateClient;
