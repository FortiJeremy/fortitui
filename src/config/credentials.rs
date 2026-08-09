//! Credential handling.
//!
//! Tokens are resolved **per profile**, in this order:
//!   1. `profile.token` (explicit plaintext override in the profile YAML)
//!   2. `FORTITUI_<PROFILE>` environment variable (per-profile, preferred)
//!   3. OS keychain entry `fortitui/<profile>` (feature-gated via `keyring`)
//!   4. legacy global `FORTITUI_TOKEN` fallback (single shared token)
//!
//! Secrets must never be logged or persisted in the profile file.
//!
//! The per-profile env var name is derived from the profile name, so
//! `fortitui --profile X` picks up `FORTITUI_X` automatically.

use anyhow::{anyhow, Result};
use std::env;

/// Compute the per-profile environment variable name.
///
/// `branch-01` -> `FORTITUI_BRANCH_01`, `leatherleaf` -> `FORTITUI_LEATHERLEAF`.
/// Non-alphanumeric characters (dashes, dots, etc.) become underscores.
pub fn env_var_for(profile: &str) -> String {
    let sanitized: String = profile
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect();
    format!("FORTITUI_{sanitized}")
}

/// Read the token for a profile using the resolution order above.
pub fn get(profile: &str) -> Result<String> {
    // 2. Per-profile env var first (so it can override the keychain).
    let env_name = env_var_for(profile);
    if let Ok(t) = env::var(&env_name) {
        if !t.is_empty() {
            return Ok(t);
        }
    }
    // 3. OS keychain entry (feature-gated).
    #[cfg(feature = "keyring")]
    {
        let entry = keyring::Entry::new("fortitui", profile)?;
        if let Ok(s) = entry.get_password() {
            return Ok(s);
        }
    }
    // 4. Legacy global fallback.
    if let Ok(t) = env::var("FORTITUI_TOKEN") {
        if !t.is_empty() {
            return Ok(t);
        }
    }
    Err(anyhow!(
        "no token found for profile '{profile}'. Provide one of:\n  \
         - env var {env_name} (per-profile)\n  \
         - env var FORTITUI_TOKEN (global fallback)\n  \
         - `fortitui credential set {profile}` (keyring feature)"
    ))
}

/// Store a secret in the OS keychain (feature-gated).
#[cfg(feature = "keyring")]
pub fn store(profile: &str, secret: &str) -> Result<()> {
    let entry = keyring::Entry::new("fortitui", profile)?;
    entry.set_password(secret)?;
    Ok(())
}

/// Delete a secret from the OS keychain (feature-gated). A missing entry is
/// not an error.
#[cfg(feature = "keyring")]
pub fn delete(profile: &str) -> Result<()> {
    let entry = keyring::Entry::new("fortitui", profile)?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(anyhow!(
            "failed to delete keychain entry for '{profile}': {e}"
        )),
    }
}

/// Redact a string for logs (never leak full secrets).
pub fn redact(s: &str) -> String {
    match s.len() {
        0 => "<empty>".to_string(),
        1..=8 => "***".to_string(),
        _ => format!("{}***", &s[..4]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_var_for_derives_uppercase_sanitized_names() {
        assert_eq!(env_var_for("leatherleaf"), "FORTITUI_LEATHERLEAF");
        assert_eq!(env_var_for("pve-dev"), "FORTITUI_PVE_DEV");
        assert_eq!(env_var_for("branch-01"), "FORTITUI_BRANCH_01");
        assert_eq!(env_var_for("my.profile"), "FORTITUI_MY_PROFILE");
    }

    #[test]
    fn get_resolves_per_profile_env_var() {
        unsafe { std::env::set_var("FORTITUI_MYTEST", "super-secret-token") };
        // The per-profile env var is preferred over the global fallback and any
        // keychain entry.
        assert_eq!(get("mytest").unwrap(), "super-secret-token");
        unsafe { std::env::remove_var("FORTITUI_MYTEST") };
    }
}
