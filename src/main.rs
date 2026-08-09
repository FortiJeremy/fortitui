//! FortiTUI — a fast, keyboard-driven terminal console for FortiGate.
//!
//! Phase 1: Direct FortiGate connectivity (FortiOS 8.0.0+). Read-only by default.

use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let args = fortitui::cli::Args::parse();
    fortitui::cli::init_tracing(args.debug);
    tracing::debug!("FortiTUI starting (debug mode)");
    fortitui::cli::run(args).await
}
