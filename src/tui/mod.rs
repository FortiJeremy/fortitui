//! Interactive terminal UI (Ratatui).
//!
//! The TUI consumes only normalized models via the `FortiGateBackend` trait —
//! it never touches raw FortiGate API responses. Phase 1 is keyboard-first.

pub mod app;
pub mod event;
pub mod screens;
pub mod state;

pub use app::run;
