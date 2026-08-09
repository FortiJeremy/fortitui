//! Profile definitions (YAML).
//!
//! A profile represents one connection target. `type` distinguishes direct vs
//! future server/fortimanager modes. Credentials are referenced by name.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    #[serde(default = "default_type")]
    pub r#type: String,

    /// Hostname or IP address.
    #[serde(default)]
    pub host: String,

    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default = "default_tls")]
    pub verify_tls: bool,

    /// Reference to a credential entry (keychain) or env var name.
    #[serde(default)]
    pub credential: String,

    /// Token override (usually empty; use env FORTITUI_TOKEN).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

fn default_type() -> String {
    "direct".to_string()
}
fn default_port() -> u16 {
    443
}
fn default_tls() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfilesConfig {
    #[serde(default)]
    pub profiles: BTreeMap<String, Profile>,
}

impl ProfilesConfig {
    /// Create a config even if the file doesn't exist yet.
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw =
            std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        let cfg: Self =
            serde_yaml::from_str(&raw).with_context(|| format!("parsing {}", path.display()))?;
        Ok(cfg)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        // Set restrictive perms on profiles file (may reference credential names).
        let raw = serde_yaml::to_string(self)?;
        std::fs::write(path, raw)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
        }
        Ok(())
    }
}
