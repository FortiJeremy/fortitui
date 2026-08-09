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
            // Allow HTTP/2 via ALPN so concurrent polling (spec §38) can use
            // multiplexing when the FortiGate supports it. Do NOT force http1.
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        let base = format!("https://{host}:{port}/api/v2/monitor");
        Ok(Self {
            base,
            token,
            client,
        })
    }

    /// Perform a GET to a monitor endpoint and return the full response body.
    /// Normalizers expect the HTTP envelope (with `results`, top-level
    /// `serial`/`version`/`vdom`) — the raw FortiGate response is never passed
    /// to the TUI directly, only after normalization.
    pub async fn get(&self, endpoint: &str) -> Result<serde_json::Value> {
        let url = format!("{}{}", self.base, endpoint);
        self.send(&url).await
    }

    /// Perform a GET with URL-encoded query parameters (e.g. route lookup's
    /// `?destination=`). `params` is an ordered list of `(key, value)` pairs.
    pub async fn get_query(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> Result<serde_json::Value> {
        let base = format!("{}{}", self.base, endpoint);
        let url = if params.is_empty() {
            base
        } else {
            reqwest::Url::parse_with_params(&base, params.iter().map(|(k, v)| (*k, *v)))
                .map_err(|e| anyhow!("invalid query URL for {endpoint}: {e}"))?
                .to_string()
        };
        self.send(&url).await
    }

    /// Shared request/parse path for `get` and `get_query`.
    async fn send(&self, url: &str) -> Result<serde_json::Value> {
        debug!("GET {url}");
        let resp = self
            .client
            .get(url)
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|e| anyhow!("HTTP error for {url}: {e}"))?;

        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| anyhow!("read error for {url}: {e}"))?;

        if status.is_success() {
            if text.trim().is_empty() {
                // FortiGate may return 200 with an empty body (e.g. nothing to
                // report for some endpoints). Return a null value so callers
                // treat it as "no results" rather than failing.
                return Ok(serde_json::Value::Null);
            }
            let body: serde_json::Value =
                serde_json::from_str(&text).map_err(|e| anyhow!("invalid JSON from {url}: {e}"))?;
            return Ok(body);
        }

        // Error path: try to surface the FortiGate error message even when the
        // body isn't clean JSON (e.g. empty body, HTML, or plain text).
        let parsed = serde_json::from_str::<serde_json::Value>(&text).ok();
        let msg = parsed
            .as_ref()
            .and_then(|b| b.get("status").and_then(|s| s.as_str()))
            .unwrap_or("request failed");
        let code = parsed
            .as_ref()
            .and_then(|b| b.get("http_status").map(|v| v.to_string()))
            .unwrap_or_else(|| {
                if text.trim().is_empty() {
                    String::new()
                } else {
                    text.chars().take(120).collect()
                }
            });
        Err(anyhow!(
            "FortiGate returned HTTP {} ({msg}) for {url}{}",
            status.as_u16(),
            if code.is_empty() {
                String::new()
            } else {
                format!(" [{code}]")
            }
        ))
    }
}
