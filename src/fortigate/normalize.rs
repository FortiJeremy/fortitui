//! Normalization from raw FortiGate JSON into application models.
//!
//! These functions parse the observed FortiOS 8.0 API response shapes
//! (see fixtures/fortios-8.0/) into the stable `crate::models` types. The TUI
//! never sees the raw `serde_json::Value` returned by the client.

use crate::models::*;
use anyhow::Result;
use serde_json::Value;

pub fn system_status(v: &Value) -> Result<SystemStatus> {
    let serial = v
        .get("serial")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let version = v
        .get("version")
        .and_then(|s| s.as_str())
        .unwrap_or("")
        .to_string();
    let build = v.get("build").map(|b| b.to_string()).unwrap_or_default();
    let hostname = v
        .get("results")
        .and_then(|r| r.get("hostname"))
        .and_then(|h| h.as_str())
        .unwrap_or("")
        .to_string();
    let model = v
        .get("results")
        .and_then(|r| r.get("model"))
        .and_then(|m| m.as_str())
        .unwrap_or("")
        .to_string();
    Ok(SystemStatus {
        hostname,
        serial,
        model,
        fortios: version,
        build,
        ..Default::default()
    })
}

pub fn resource_usage(v: &Value) -> Result<SystemStatus> {
    // Each of cpu/mem/disk/session is a LIST of one-or-more sensor entries,
    // each with { current, historical }. Take the max `current` across entries.
    let elem = |name: &str| -> Option<f64> {
        let arr = v
            .get("results")
            .and_then(|r| r.get(name))
            .and_then(|x| x.as_array())
            .cloned()
            .unwrap_or_default();
        arr.iter()
            .filter_map(|e| e.get("current").and_then(|c| c.as_f64()))
            .fold(None::<f64>, |acc, c| match acc {
                Some(m) if m >= c => Some(m),
                _ => Some(c),
            })
    };
    let cpu = elem("cpu").unwrap_or(0.0) as f32;
    let mem = elem("mem").unwrap_or(0.0) as f32;
    let disk = elem("disk").unwrap_or(0.0) as f32;
    let sessions = elem("session").unwrap_or(0.0) as u64;
    Ok(SystemStatus {
        cpu_percent: cpu,
        memory_percent: mem,
        disk_percent: disk,
        sessions,
        ..Default::default()
    })
}

pub fn interfaces(v: &Value) -> Result<Vec<InterfaceStatus>> {
    let mut out = Vec::new();
    // results is a keyed map: { "port1": {...}, ... }
    if let Some(map) = v.get("results").and_then(|r| r.as_object()) {
        for (_key, val) in map {
            let name = val
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("")
                .to_string();
            let ip = val.get("ip").and_then(|i| i.as_str()).unwrap_or("");
            let mask = val.get("mask").and_then(|m| m.as_u64()).unwrap_or(0);
            let link = val.get("link").and_then(|l| l.as_bool()).unwrap_or(false);
            let ipv4 = if !ip.is_empty() && ip != "0.0.0.0" {
                Some(if mask > 0 {
                    format!("{ip}/{mask}")
                } else {
                    ip.to_string()
                })
            } else {
                None
            };
            out.push(InterfaceStatus {
                name,
                ipv4,
                link_state: if link { LinkState::Up } else { LinkState::Down },
                rx_bytes: val.get("rx_bytes").and_then(|b| b.as_u64()).unwrap_or(0),
                tx_bytes: val.get("tx_bytes").and_then(|b| b.as_u64()).unwrap_or(0),
                rx_packets: val.get("rx_packets").and_then(|b| b.as_u64()).unwrap_or(0),
                tx_packets: val.get("tx_packets").and_then(|b| b.as_u64()).unwrap_or(0),
                errors: val.get("rx_errors").and_then(|b| b.as_u64()).unwrap_or(0),
                speed_mbps: val.get("speed").and_then(|s| s.as_f64()).map(|f| f as u64),
                ..Default::default()
            });
        }
    }
    Ok(out)
}

pub fn sdwan_members(v: &Value) -> Result<Vec<SdwanMember>> {
    let mut out = Vec::new();
    let arr = v
        .get("results")
        .and_then(|r| r.as_array())
        .cloned()
        .unwrap_or_default();
    for m in arr {
        let state = m
            .get("link")
            .and_then(|l| l.as_str())
            .unwrap_or("down")
            .to_string();
        let interface = m
            .get("interface")
            .and_then(|i| i.as_str())
            .unwrap_or("")
            .to_string();
        out.push(SdwanMember {
            name: interface.clone(),
            interface,
            state: if state == "up" {
                "ACTIVE".to_string()
            } else {
                "DOWN".to_string()
            },
            tx_rate_bps: m.get("tx_bandwidth").and_then(|b| b.as_u64()),
            rx_rate_bps: m.get("rx_bandwidth").and_then(|b| b.as_u64()),
            sessions: m.get("session").and_then(|s| s.as_u64()).unwrap_or(0),
            ..Default::default()
        });
    }
    Ok(out)
}

pub fn sdwan_health_check(v: &Value) -> Result<SdwanState> {
    // { results: { "Default_DNS": { "port4": {...}, ... }, ... } }
    let mut checks = Vec::new();
    if let Some(map) = v.get("results").and_then(|r| r.as_object()) {
        for (check_name, members) in map {
            if let Some(mm) = members.as_object() {
                for (member, stats) in mm {
                    let status = stats
                        .get("status")
                        .and_then(|s| s.as_str())
                        .unwrap_or("")
                        .to_string();
                    let latency = stats.get("latency").and_then(|l| l.as_f64());
                    let jitter = stats.get("jitter").and_then(|j| j.as_f64());
                    let loss = stats.get("packet_loss").and_then(|l| l.as_f64());
                    checks.push(SdwanHealthCheck {
                        name: check_name.clone(),
                        member: member.clone(),
                        status: if status == "up" {
                            Some("PASS".to_string())
                        } else if status == "down" {
                            Some("FAIL".to_string())
                        } else {
                            Some(status)
                        },
                        latency_ms: latency.map(|v| v as f32),
                        jitter_ms: jitter.map(|v| v as f32),
                        packet_loss_pct: loss.map(|v| v as f32),
                    });
                }
            }
        }
    }
    Ok(SdwanState {
        health_checks: checks,
        ..Default::default()
    })
}

pub fn ipsec_tunnels(v: &Value) -> Result<Vec<IpsecTunnel>> {
    let mut out = Vec::new();
    let arr = v
        .get("results")
        .and_then(|r| r.as_array())
        .cloned()
        .unwrap_or_default();
    for t in arr {
        out.push(IpsecTunnel {
            name: t
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("")
                .to_string(),
            phase1_state: t
                .get("status")
                .and_then(|s| s.as_str())
                .map(|s| s.to_string()),
            ike_version: t
                .get("ike_version")
                .and_then(|s| s.as_str())
                .map(|s| s.to_string()),
            remote_gateway: t
                .get("remote_gw")
                .and_then(|s| s.as_str())
                .map(|s| s.to_string()),
            rx_bytes: t.get("rx_bytes").and_then(|b| b.as_u64()).unwrap_or(0),
            tx_bytes: t.get("tx_bytes").and_then(|b| b.as_u64()).unwrap_or(0),
            ..Default::default()
        });
    }
    Ok(out)
}

pub fn bgp_neighbors(v: &Value) -> Result<BgpState> {
    let router_id = v
        .get("results")
        .and_then(|r| r.get("router_id"))
        .and_then(|s| s.as_str())
        .map(|s| s.to_string());
    let arr = v
        .get("results")
        .and_then(|r| r.get("neighbors"))
        .and_then(|n| n.as_array())
        .cloned()
        .unwrap_or_default();
    let mut neighbors = Vec::new();
    for n in arr {
        let remote_as = n.get("remote_as").and_then(|v| v.as_u64());
        neighbors.push(crate::models::BgpNeighbor {
            address: n
                .get("remote_host")
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string(),
            remote_as: remote_as.map(|a| a as u32),
            state: n
                .get("state")
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string(),
            rx_prefixes: n.get("rx_pfx").and_then(|v| v.as_u64()).unwrap_or(0),
            tx_prefixes: n.get("tx_pfx").and_then(|v| v.as_u64()).unwrap_or(0),
            ..Default::default()
        });
    }
    Ok(BgpState {
        router_id,
        local_as: None,
        neighbors,
    })
}

pub fn routes(v: &Value, family: &str) -> Result<Vec<Route>> {
    let mut out = Vec::new();
    let arr = v
        .get("results")
        .and_then(|r| r.as_array())
        .cloned()
        .unwrap_or_default();
    for r in arr {
        let protocol = r
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();
        let origin = r
            .get("origin")
            .and_then(|o| o.as_str())
            .unwrap_or("")
            .to_string();
        let proto = if origin == "sd-wan" || origin == "sdwan" {
            "sd-wan".to_string()
        } else {
            protocol
        };
        out.push(Route {
            prefix: r
                .get("ip_mask")
                .and_then(|i| i.as_str())
                .unwrap_or("")
                .to_string(),
            family: family.to_string(),
            protocol: proto,
            next_hop: r
                .get("gateway")
                .and_then(|g| g.as_str())
                .map(|s| s.to_string()),
            interface: r
                .get("interface")
                .and_then(|i| i.as_str())
                .map(|s| s.to_string()),
            distance: r.get("distance").and_then(|d| d.as_u64()).map(|d| d as u32),
            metric: r.get("metric").and_then(|m| m.as_u64()).map(|m| m as u32),
            ..Default::default()
        });
    }
    Ok(out)
}
