//! FortiGate REST API client.
//!
//! Handles HTTPS, Bearer-token auth, TLS verification toggling, and raw
//! `/api/v2/monitor/...` requests. Returns raw JSON `serde_json::Value`;
//! normalization happens elsewhere (never in the TUI).

use anyhow::{anyhow, Result};
use tracing::debug;

/// The FortiGate API client.
#[derive(Debug, Clone)]
pub struct FortiGateClient {
    base: String,
    token: String,
    client: reqwest::Client,
}

impl FortiGateClient {
    pub fn new(host: &str, port: u16, token: String, verify_tls: bool) -> Result<Self> {
        // reqwest tls config. Danger flag: verify=false only for lab/testing.
        let tls = if verify_tls {
            reqwest::Client::builder()
        } else {
            reqwest::Client::builder().danger_accept_invalid_certs(true)
        };
        let client = tls
            .http1_only() // FortiGate API is most reliable over HTTP/1.1
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        let base = format!("https://{host}:{port}/api/v2/monitor");
        Ok(Self { base, token, client })
    }

    /// Perform a GET to a monitor endpoint and return the full response body.
    /// Normalizers expect the HTTP envelope (with `results`, top-level
    /// `serial`/`version`/`vdom`) — the raw FortiGate response is never passed
    /// to the TUI directly, only after normalization.
    pub async fn get(&self, endpoint: &str) -> Result<serde_json::Value> {
        let url = format!("{}{}", self.base, endpoint);
        debug!("GET {}", endpoint);
        let resp = self
            .client
            .get(&url)
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|e| anyhow!("HTTP error for {endpoint}: {e}"))?;

        let status = resp.status();
        let body: serde_json::Value = resp.json().await?;

        // Normalize error surfaces per spec §11/§39.
        if status.is_success() {
            Ok(body)
        } else {
            let msg = body
                .get("status")
                .and_then(|s| s.as_str())
                .unwrap_or("error");
            let code = body
                .get("http_status")
                .map(|v| v.to_string())
                .unwrap_or_default();
            Err(anyhow!(
                "FortiGate returned HTTP {} ({msg}) for {endpoint} [{code}]",
                status.as_u16()
            ))
        }
    }
}
