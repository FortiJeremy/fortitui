//! Command-line interface definition and dispatch.
//!
//! Subcommands:
//! - profile add / list / remove / test
//! - status / interfaces / sdwan / vpn / routes  (non-TUI, JSON-capable)
//!
//! Global flags: `--profile`, `--profile-select`, `--debug`, `--version`, `--config`.

use crate::backend::traits::{AddressFamily, FortiGateBackend};
use crate::backend::DirectBackend;
use crate::config;
use crate::models::LinkState;
use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
#[cfg(feature = "keyring")]
use std::io::Write;

/// FortiTUI — terminal console for FortiGate.
#[derive(Parser, Debug)]
#[command(name = "fortitui", version, about)]
pub struct Args {
    /// Profile to connect to.
    #[arg(long, global = true)]
    pub profile: Option<String>,

    /// Interactive profile selection (placeholder for now).
    #[arg(long, global = true)]
    pub profile_select: bool,

    /// Enable debug logging.
    #[arg(long, global = true)]
    pub debug: bool,

    /// Emit machine-readable JSON (for non-TUI commands).
    #[arg(long, global = true)]
    pub json: bool,

    /// Skip TLS certificate verification (lab/testing only).
    #[arg(long, global = true)]
    pub insecure: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Manage connection profiles.
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },
    /// Store or clear a profile's API token in the OS keychain.
    Credential {
        #[command(subcommand)]
        action: CredentialAction,
    },
    /// Show system status (non-TUI).
    Status,
    /// Show interface overview (non-TUI).
    Interfaces,
    /// Show SD-WAN overview (non-TUI).
    Sdwan,
    /// Show IPsec VPN overview (non-TUI).
    Vpn,
    /// Show routing table (non-TUI).
    Routes,
    /// Show active firewall sessions (non-TUI).
    Sessions,
    /// Show firewall policy counters (non-TUI).
    Policies,
    /// Best-route lookup for a destination (non-TUI).
    Lookup {
        /// Destination IP (IPv4 or IPv6).
        destination: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum ProfileAction {
    /// Create a new profile (prompts for host, port, token).
    Add,
    /// List configured profiles.
    List,
    /// Remove a profile.
    Remove { name: String },
    /// Test connectivity to a profile.
    Test { name: String },
}

#[derive(Subcommand, Debug)]
pub enum CredentialAction {
    /// Store a profile's API token in the OS keychain (prompts for the token).
    Set { profile: String },
    /// Remove a profile's API token from the OS keychain.
    Unset { profile: String },
}

/// Initialize structured logging. Only emits in debug mode.
pub fn init_tracing(debug: bool) {
    use tracing_subscriber::{fmt, EnvFilter};
    if debug {
        let filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("fortitui=debug"));
        fmt()
            .with_env_filter(filter)
            .with_max_level(tracing::Level::DEBUG)
            .init();
    }
}

/// Top-level command dispatch.
pub async fn run(args: Args) -> Result<()> {
    if args.profile_select {
        return Err(anyhow!(
            "interactive profile selection is not yet implemented"
        ));
    }

    let command = args.command; // move command out of args
    let insecure = args.insecure;
    let json = args.json;
    match command {
        Some(Command::Profile { action }) => run_profile(insecure, action).await,
        Some(Command::Credential { action }) => run_credential(action).await,
        Some(Command::Status) => run_connect(args.profile, insecure, json, "status").await,
        Some(Command::Interfaces) => run_connect(args.profile, insecure, json, "interfaces").await,
        Some(Command::Sdwan) => run_connect(args.profile, insecure, json, "sdwan").await,
        Some(Command::Vpn) => run_connect(args.profile, insecure, json, "vpn").await,
        Some(Command::Routes) => run_connect(args.profile, insecure, json, "routes").await,
        Some(Command::Sessions) => run_connect(args.profile, insecure, json, "sessions").await,
        Some(Command::Policies) => run_connect(args.profile, insecure, json, "policies").await,
        Some(Command::Lookup { destination }) => {
            run_lookup(args.profile, insecure, json, &destination).await
        }
        None => {
            let name = match args.profile.clone() {
                Some(n) => n,
                None => select_default_profile()?,
            };
            tracing::debug!("entering TUI with profile '{name}'");
            let b = backend(&name, args.insecure).await?;
            crate::tui::run(b, name).await
        }
    }
}

/// Choose a profile when none was passed explicitly (spec §8): a single
/// configured profile is offered automatically; otherwise the user must pick.
fn select_default_profile() -> Result<String> {
    let profiles = config::list_profiles()?;
    match profiles.len() {
        0 => Err(anyhow!(
            "no profiles configured — run `fortitui profile add` first"
        )),
        1 => Ok(profiles[0].clone()),
        _ => Err(anyhow!(
            "multiple profiles configured ({}) — pass one with `--profile <name>`",
            profiles.join(", ")
        )),
    }
}

async fn run_profile(insecure: bool, action: ProfileAction) -> Result<()> {
    match action {
        ProfileAction::List => {
            for name in config::list_profiles()? {
                println!("{name}");
            }
            Ok(())
        }
        ProfileAction::Add => {
            let name = config::interactive_add()?;
            println!("Profile '{name}' added.");
            Ok(())
        }
        ProfileAction::Remove { name } => {
            config::remove_profile(&name)?;
            println!("Profile '{name}' removed.");
            Ok(())
        }
        ProfileAction::Test { name } => {
            let backend = backend(&name, insecure).await?;
            let s = backend.system_status().await?;
            println!("OK {} {} {} ({})", s.hostname, s.model, s.fortios, s.serial);
            Ok(())
        }
    }
}

/// Store or clear a profile's token in the OS keychain.
async fn run_credential(action: CredentialAction) -> Result<()> {
    match action {
        CredentialAction::Set { profile } => {
            #[cfg(feature = "keyring")]
            {
                print!("API token for profile '{profile}': ");
                std::io::stdout().flush()?;
                let mut token = String::new();
                std::io::stdin().read_line(&mut token)?;
                let token = token.trim();
                if token.is_empty() {
                    return Err(anyhow!("token required"));
                }
                config::credentials::store(&profile, token)?;
                let env = config::credentials::env_var_for(&profile);
                println!("Stored token for profile '{profile}' in the OS keychain.");
                println!(
                    "Unset {} (or FORTITUI_TOKEN) so the keychain entry is used.",
                    env
                );
                Ok(())
            }
            #[cfg(not(feature = "keyring"))]
            {
                Err(anyhow!(
                    "keyring support disabled — rebuild with `cargo build --features keyring` \
                     (cannot set a token for profile '{profile}')"
                ))
            }
        }
        CredentialAction::Unset { profile } => {
            #[cfg(feature = "keyring")]
            {
                config::credentials::delete(&profile)?;
                println!("Removed token for profile '{profile}' from the OS keychain.");
                Ok(())
            }
            #[cfg(not(feature = "keyring"))]
            {
                Err(anyhow!(
                    "keyring support disabled — rebuild with `cargo build --features keyring` \
                     (cannot unset a token for profile '{profile}')"
                ))
            }
        }
    }
}

/// Build a DirectBackend from the named profile.
pub async fn backend(name: &str, insecure: bool) -> Result<DirectBackend> {
    let profile =
        config::load_profile(name).with_context(|| format!("failed to load profile '{name}'"))?;
    DirectBackend::from_profile(profile, name, insecure).await
}

/// Connect and print the requested view.
async fn run_connect(
    profile: Option<String>,
    insecure: bool,
    json: bool,
    view: &str,
) -> Result<()> {
    let name = profile.ok_or_else(|| anyhow!("--profile <name> is required for this command"))?;
    let b = backend(&name, insecure).await?;

    match view {
        "status" => {
            let s = b.system_status().await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&s)?);
            } else {
                println!("{} — {} {}", s.hostname, s.model, s.fortios);
                println!("  Serial:   {}", s.serial);
                println!("  CPU:      {:.0}%", s.cpu_percent);
                println!("  Memory:   {:.0}%", s.memory_percent);
                println!("  Disk:     {:.0}%", s.disk_percent);
                println!("  Sessions: {}", s.sessions);
                println!("  Uptime:   {}s", s.uptime_secs);
            }
        }
        "interfaces" => {
            let ifs = b.interfaces().await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&ifs)?);
            } else {
                for i in ifs {
                    println!(
                        "{:<12} {} {:>12} {:>14} {:>14}",
                        i.name,
                        if i.link_state == LinkState::Up {
                            "UP"
                        } else {
                            "DOWN"
                        },
                        i.ipv4.as_deref().unwrap_or("--"),
                        fmt_bytes(i.rx_bytes),
                        fmt_bytes(i.tx_bytes),
                    );
                }
            }
        }
        "sdwan" => {
            let s = b.sdwan().await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&s)?);
            } else {
                for m in s.members {
                    println!(
                        "{:<12} {:<8} lat={:<6} jit={:<5} loss={:<6} sla={}",
                        m.name,
                        m.state,
                        m.latency_ms.map_or("--".into(), |v| format!("{v:.0}ms")),
                        m.jitter_ms.map_or("--".into(), |v| format!("{v:.0}ms")),
                        m.packet_loss_pct
                            .map_or("--".into(), |v| format!("{v:.1}%")),
                        m.sla.as_deref().unwrap_or("--"),
                    );
                }
            }
        }
        "vpn" => {
            let v = b.vpn().await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&v)?);
            } else {
                for t in v {
                    println!(
                        "{:<20} {:<8} {:<6}",
                        t.name,
                        t.phase1_state.as_deref().unwrap_or("--"),
                        t.ike_version.as_deref().unwrap_or("--")
                    );
                }
            }
        }
        "routes" => {
            let r = b.routes(AddressFamily::Ipv4).await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&r)?);
            } else {
                for r in r {
                    println!(
                        "{:<18} {:<12} {:<16} {:<10} d={}",
                        r.prefix,
                        r.protocol,
                        r.next_hop.as_deref().unwrap_or("--"),
                        r.interface.as_deref().unwrap_or("--"),
                        r.distance.map_or(0, |d| d),
                    );
                }
            }
        }
        "sessions" => {
            let s = b.sessions(Default::default()).await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&s)?);
            } else {
                for s in s {
                    println!(
                        "{:<15}:{:<5} -> {:<15}:{:<5} {:<4} pol={:<4} {:<6} {:<8} {}s",
                        s.src,
                        s.src_port,
                        s.dst,
                        s.dst_port,
                        s.proto,
                        s.policy.map_or("--".into(), |p| p.to_string()),
                        fmt_bytes(s.bytes),
                        fmt_bytes(s.packets),
                        s.age_secs,
                    );
                }
            }
        }
        "policies" => {
            let p = b.policies().await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&p)?);
            } else {
                for p in p {
                    println!(
                        "{:<4} hits={:<10} bytes={:<12} sessions={}",
                        p.id,
                        p.hit_count,
                        fmt_bytes(p.bytes),
                        p.sessions,
                    );
                }
            }
        }
        _ => unreachable!(),
    }
    Ok(())
}

/// Best-route lookup for a destination (non-TUI).
async fn run_lookup(
    profile: Option<String>,
    insecure: bool,
    json: bool,
    destination: &str,
) -> Result<()> {
    let name = profile.ok_or_else(|| anyhow!("--profile <name> is required for this command"))?;
    let b = backend(&name, insecure).await?;
    let r = b.route_lookup(destination).await?;
    if json {
        println!("{}", serde_json::to_string_pretty(&r)?);
        return Ok(());
    }
    if r.is_empty() {
        println!("No route found for {destination}");
        return Ok(());
    }
    for r in r {
        println!(
            "{:<20} {:<10} {:<16} {:<10} d={} m={}",
            r.prefix,
            r.protocol,
            r.next_hop.as_deref().unwrap_or("--"),
            r.interface.as_deref().unwrap_or("--"),
            r.distance.map_or(0, |d| d),
            r.metric.map_or(0, |m| m),
        );
    }
    Ok(())
}

fn fmt_bytes(n: u64) -> String {
    const K: f64 = 1024.0;
    const M: f64 = K * 1024.0;
    const G: f64 = M * 1024.0;
    let n = n as f64;
    if n >= G {
        format!("{:.1}G", n / G)
    } else if n >= M {
        format!("{:.1}M", n / M)
    } else if n >= K {
        format!("{:.1}K", n / K)
    } else {
        format!("{n:.0}")
    }
}
