//! Credential handling.
//!
//! Prefer OS-native keychain (`keyring`, feature-gated) or environment variables.
//! Secrets must never be logged or persisted in the profile file.

use anyhow::{anyhow, Result};

/// Read the token for a profile.
///
/// Resolution order:
///   1. FORTITUI_TOKEN env var (lab/automation)
///   2. OS keychain entry (if `keyring` feature enabled)
pub fn get(credential: &str) -> Result<String> {
    if let Ok(t) = std::env::var("FORTITUI_TOKEN") {
        if !t.is_empty() {
            return Ok(t);
        }
    }
    #[cfg(feature = "keyring")]
    {
        let entry = keyring::Entry::new("fortitui", credential)?;
        if let Ok(s) = entry.get_password() {
            return Ok(s);
        }
    }
    Err(anyhow!(
        "no token found for '{credential}'. Set FORTITUI_TOKEN=<api-key> or enable the keyring feature."
    ))
}

/// Store a secret in the OS keychain (feature-gated).
#[cfg(feature = "keyring")]
pub fn store(credential: &str, secret: &str) -> Result<()> {
    let entry = keyring::Entry::new("fortitui", credential)?;
    entry.set_password(secret)?;
    Ok(())
}

/// Redact a string for logs (never leak full secrets).
pub fn redact(s: &str) -> String {
    match s.len() {
        0 => "<empty>".to_string(),
        1..=8 => "***".to_string(),
        _ => format!("{}***", &s[..4]),
    }
}
