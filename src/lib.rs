//! FortiTUI library crate.
//!
//! Exposes the CLI, backend, config, and FortiGate client modules so the binary
//! and integration tests share code.

pub mod backend;
pub mod cli;
pub mod config;
pub mod fortigate;
pub mod models;
