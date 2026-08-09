//! Configuration, profile, and credential handling.
//!
//! Config format: YAML (Q9). Profiles live under a platform-conventional dir.
//! Secrets are referenced by name (keychain) or supplied via env, never stored
//! in the profile file.

pub mod credentials;
pub mod profiles;

use anyhow::{anyhow, Result};
use std::io::Write;
use std::path::PathBuf;

/// Platform-conventional config directory.
/// Linux: ~/.config/fortitui/  (respects $XDG_CONFIG_HOME)
pub fn profile_dir() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(xdg).join("fortitui");
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".config").join("fortitui");
    }
    PathBuf::from(".fortitui")
}

fn profiles_file() -> PathBuf {
    profile_dir().join("profiles.yaml")
}

/// List profile names.
pub fn list_profiles() -> Result<Vec<String>> {
    let cfg = profiles::ProfilesConfig::load(&profiles_file())?;
    Ok(cfg.profiles.keys().cloned().collect())
}

/// Load a profile by name.
pub fn load_profile(name: &str) -> Result<profiles::Profile> {
    let cfg = profiles::ProfilesConfig::load(&profiles_file())?;
    cfg.profiles
        .get(name)
        .cloned()
        .ok_or_else(|| anyhow!("profile '{name}' not found"))
}

/// Remove a profile by name.
pub fn remove_profile(name: &str) -> Result<()> {
    let mut cfg = profiles::ProfilesConfig::load(&profiles_file())?;
    cfg.profiles.remove(name);
    profiles::ProfilesConfig::save(&cfg, &profiles_file())?;
    Ok(())
}

/// Interactive profile creation (simple line-based prompts).
pub fn interactive_add() -> Result<String> {
    let mut host = String::new();
    let mut port = String::new();
    let mut token = String::new();
    let mut name = String::new();

    print!("FortiGate host (IP or hostname): ");
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut host)?;

    print!("HTTPS port [443]: ");
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut port)?;

    print!("API token: ");
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut token)?;

    let host = host.trim().to_string();
    let port: u16 = port.trim().parse().unwrap_or(443);
    let token = token.trim().to_string();
    if host.is_empty() || token.is_empty() {
        return Err(anyhow!("host and token are required"));
    }

    // Profile name defaults to host if not given.
    print!("Profile name [{}]: ", host);
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut name)?;
    let name = name.trim();
    let name = if name.is_empty() {
        host.clone()
    } else {
        name.to_string()
    };

    let profile = profiles::Profile {
        r#type: "direct".to_string(),
        host,
        port,
        verify_tls: true,
        credential: name.clone(),
        token: None,
    };

    // Store token in keychain if feature enabled; otherwise discard (env-based).
    #[cfg(feature = "keyring")]
    credentials::store(&name, &token)?;
    #[cfg(not(feature = "keyring"))]
    {
        eprintln!("note: keyring disabled; token NOT persisted. Set FORTITUI_TOKEN=<api-key> when running.");
    }

    let mut cfg = profiles::ProfilesConfig {
        profiles: Default::default(),
    };
    if profiles_file().exists() {
        cfg = profiles::ProfilesConfig::load(&profiles_file())?;
    }
    cfg.profiles.insert(name.clone(), profile);
    profiles::ProfilesConfig::save(&cfg, &profiles_file())?;
    Ok(name)
}
