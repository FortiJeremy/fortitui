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

        let status = resp.status().as_u16();
        let text = resp
            .text()
            .await
            .map_err(|e| anyhow!("read error for {url}: {e}"))?;

        // Pure parsing path is factored out so error/empty branches are unit
        // testable without a live FortiGate (A5).
        parse_response(status, &text, url)
    }
}

/// Parse a FortiGate monitor response body given the HTTP status.
///
/// For 2xx responses:
/// - an empty 200 body yields `Value::Null` (FortiGate returns empty for some
///   endpoints; callers treat it as "no results" rather than failing)
/// - a non-empty 200 body is parsed as JSON (the HTTP envelope is preserved).
///
/// For non-2xx responses the FortiGate error message (`status`) and a snippet
/// of the body (`http_status` or raw text) are surfaced for actionable
/// diagnostics.
fn parse_response(status: u16, text: &str, url: &str) -> Result<serde_json::Value> {
    if (200..300).contains(&status) {
        if text.trim().is_empty() {
            return Ok(serde_json::Value::Null);
        }
        return serde_json::from_str(text).map_err(|e| anyhow!("invalid JSON from {url}: {e}"));
    }

    let parsed: Option<serde_json::Value> = serde_json::from_str(text).ok();
    // Prefer the actionable `error`/`message` field (monitor API) over the
    // generic `status` tag ("error" / "success").
    let msg = parsed
        .as_ref()
        .and_then(|b| {
            b.get("error")
                .and_then(|s| s.as_str())
                .or_else(|| b.get("message").and_then(|s| s.as_str()))
                .or_else(|| b.get("status").and_then(|s| s.as_str()))
        })
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
        "FortiGate returned HTTP {status} ({msg}) for {url}{}",
        if code.is_empty() {
            String::new()
        } else {
            format!(" [{code}]")
        }
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_200_body_is_null() {
        let v = parse_response(200, "  ", "https://fg/api/v2/monitor/x").unwrap();
        assert_eq!(v, serde_json::Value::Null);
    }

    #[test]
    fn valid_200_json_is_parsed() {
        let v = parse_response(200, r#"{"results":[1,2],"status":"success"}"#, "/x").unwrap();
        assert_eq!(v["results"][1], 2);
    }

    #[test]
    fn invalid_200_json_is_an_error() {
        let r = parse_response(200, "not json {{{", "/x");
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("invalid JSON"));
    }

    #[test]
    fn non_json_error_body_surfaces_status_and_snippet() {
        let err = parse_response(500, "<html>oops</html>", "/bgp").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("HTTP 500"), "msg: {msg}");
        assert!(msg.contains("<html>oops</html>"), "msg: {msg}");
    }

    #[test]
    fn json_error_surfaces_status_and_http_status() {
        let err = parse_response(
            424,
            r#"{"status":"error","http_status":424,"error":"need count"}"#,
            "/sessions",
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("HTTP 424"), "msg: {msg}");
        assert!(msg.contains("need count"), "msg: {msg}");
        assert!(msg.contains("[424]"), "msg: {msg}");
    }

    #[test]
    fn empty_error_body_has_no_snippet() {
        let err = parse_response(502, "", "/x").unwrap_err();
        assert_eq!(
            err.to_string(),
            "FortiGate returned HTTP 502 (request failed) for /x"
        );
    }
}
