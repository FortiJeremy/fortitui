//! In-memory event / state-transition detection (spec §36).
//!
//! Pure functions that diff a previous snapshot against a new one and emit
//! `Event`s for transitions the operator cares about: interface up/down,
//! SD-WAN member state changes, and CPU/memory threshold crossings. Events are
//! held in memory only (no persistence, spec §36/§61).

use crate::models::{Event, InterfaceStatus, LinkState, SdwanState, SystemStatus};
use std::time::{SystemTime, UNIX_EPOCH};

/// Unix seconds now.
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Interface link-state transitions: up↔down.
pub fn detect_interfaces(prev: &[InterfaceStatus], new: &[InterfaceStatus]) -> Vec<Event> {
    let mut out = Vec::new();
    for ni in new {
        let before = prev.iter().find(|p| p.name == ni.name);
        let before_up = before.map(|p| p.link_state == LinkState::Up);
        let new_up = ni.link_state == LinkState::Up;
        if let Some(up) = before_up {
            if up && !new_up {
                out.push(Event {
                    timestamp: now(),
                    description: format!("Interface {} went DOWN", ni.name),
                    severity: "WARNING".to_string(),
                });
            } else if !up && new_up {
                out.push(Event {
                    timestamp: now(),
                    description: format!("Interface {} came UP", ni.name),
                    severity: "INFO".to_string(),
                });
            }
        }
    }
    out
}

/// SD-WAN member state transitions (ACTIVE / STANDBY / DOWN).
pub fn detect_sdwan(prev: &SdwanState, new: &SdwanState) -> Vec<Event> {
    let mut out = Vec::new();
    for n in &new.members {
        let before = prev.members.iter().find(|m| m.name == n.name);
        if let Some(b) = before {
            if b.state != n.state {
                let sev = if n.state == "DOWN" || n.state == "STANDBY" {
                    "WARNING"
                } else {
                    "INFO"
                };
                out.push(Event {
                    timestamp: now(),
                    description: format!("SD-WAN member {} {} -> {}", n.name, b.state, n.state),
                    severity: sev.to_string(),
                });
            }
        }
    }
    out
}

/// Detect an SD-WAN member entering the ACTIVE (selected) state.
pub fn detect_sdwan_active(prev: &SdwanState, new: &SdwanState) -> Vec<Event> {
    let mut out = Vec::new();
    for n in &new.members {
        if n.state == "ACTIVE" {
            let was_active = prev
                .members
                .iter()
                .any(|m| m.name == n.name && m.state == "ACTIVE");
            if !was_active {
                out.push(Event {
                    timestamp: now(),
                    description: format!("SD-WAN member {} became ACTIVE", n.name),
                    severity: "INFO".to_string(),
                });
            }
        }
    }
    out
}

/// CPU / memory threshold crossings against spec §36 defaults (CPU>90, mem>85).
pub fn detect_system(prev: &SystemStatus, new: &SystemStatus) -> Vec<Event> {
    let mut out = Vec::new();
    if prev.cpu_percent <= 90.0 && new.cpu_percent > 90.0 {
        out.push(Event {
            timestamp: now(),
            description: format!("CPU exceeded 90% ({:.1}%)", new.cpu_percent),
            severity: "CRITICAL".to_string(),
        });
    }
    if prev.memory_percent <= 85.0 && new.memory_percent > 85.0 {
        out.push(Event {
            timestamp: now(),
            description: format!("Memory exceeded 85% ({:.1}%)", new.memory_percent),
            severity: "CRITICAL".to_string(),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{InterfaceStatus, SdwanMember, SdwanState, SystemStatus};

    fn iface(name: &str, up: bool) -> InterfaceStatus {
        InterfaceStatus {
            name: name.to_string(),
            link_state: if up { LinkState::Up } else { LinkState::Down },
            ..Default::default()
        }
    }

    #[test]
    fn interface_detect_updown() {
        let prev = vec![iface("wan1", true), iface("wan2", false)];
        let new = vec![iface("wan1", false), iface("wan2", true)];
        let evs = detect_interfaces(&prev, &new);
        assert!(evs
            .iter()
            .any(|e| e.description.contains("wan1") && e.description.contains("DOWN")));
        assert!(evs
            .iter()
            .any(|e| e.description.contains("wan2") && e.description.contains("UP")));
        assert!(evs.iter().any(|e| e.severity == "WARNING"));
        assert!(evs.iter().any(|e| e.severity == "INFO"));
        // No change -> no events.
        assert!(detect_interfaces(&prev, &prev).is_empty());
    }

    #[test]
    fn sdwan_detect_state_change() {
        let mk = |state: &str| SdwanState {
            members: vec![SdwanMember {
                name: "port4".into(),
                state: state.to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let evs = detect_sdwan(&mk("ACTIVE"), &mk("DOWN"));
        assert!(!evs.is_empty());
        assert!(evs[0].description.contains("port4"));
        assert!(detect_sdwan(&mk("ACTIVE"), &mk("ACTIVE")).is_empty());
    }

    #[test]
    fn system_threshold_crossing() {
        let low = SystemStatus {
            cpu_percent: 30.0,
            memory_percent: 40.0,
            ..Default::default()
        };
        let high = SystemStatus {
            cpu_percent: 95.0,
            memory_percent: 90.0,
            ..Default::default()
        };
        let evs = detect_system(&low, &high);
        assert_eq!(evs.len(), 2);
        assert!(evs.iter().all(|e| e.severity == "CRITICAL"));
        // Staying high does NOT re-fire.
        assert!(detect_system(&high, &high).is_empty());
    }
}
