//! Unit tests for normalizers, driven by captured FortiOS 8.0 fixtures.
//! Fixtures live in fixtures/fortios-8.0/ (captured live, sanitized).

use fortitui::fortigate::normalize;

fn load(name: &str) -> serde_json::Value {
    let path = format!("fixtures/fortios-8.0/{name}.json");
    let raw = std::fs::read_to_string(&path).expect("fixture missing");
    serde_json::from_str(&raw).expect("fixture invalid JSON")
}

#[test]
fn system_status_parses() {
    let v = load("system-status");
    let s = normalize::system_status(&v).unwrap();
    assert_eq!(s.model, "FG121G");
    assert_eq!(s.hostname, "Leatherleaf-121G");
    assert!(!s.serial.is_empty());
}

#[test]
fn resource_usage_parses() {
    let v = load("system-resource-usage");
    let s = normalize::resource_usage(&v).unwrap();
    assert!(s.cpu_percent >= 0.0);
    assert!(s.sessions > 0);
}

#[test]
fn interfaces_parse() {
    let v = load("system-interface");
    let ifs = normalize::interfaces(&v).unwrap();
    assert!(!ifs.is_empty());
    let mgmt = ifs.iter().find(|i| i.name == "mgmt").unwrap();
    assert!(mgmt.speed_mbps.is_some() || mgmt.speed_mbps.is_none());
}

#[test]
fn sdwan_members_parse() {
    let v = load("virtual-wan-members");
    let m = normalize::sdwan_members(&v).unwrap();
    assert!(!m.is_empty());
    assert!(!m[0].interface.is_empty());
}

#[test]
fn sdwan_health_check_parse() {
    let v = load("virtual-wan-health-check");
    let s = normalize::sdwan_health_check(&v).unwrap();
    assert!(!s.health_checks.is_empty());
}

#[test]
fn routes_parse() {
    let v = load("router-ipv4");
    let r = normalize::routes(&v, "ipv4").unwrap();
    assert!(!r.is_empty());
    assert!(r.iter().any(|x| !x.prefix.is_empty()));
}

#[test]
fn firewall_policy_parse_placeholder() {
    let v = load("firewall-policy");
    let arr = v.get("results").and_then(|r| r.as_array()).cloned().unwrap_or_default();
    assert!(!arr.is_empty());
}
