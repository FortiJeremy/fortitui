# FortiTUI — Gap Analysis & Developer Environment Plan

> **Status:** PLANNING ONLY. No code, containers, VMs, repos, credentials, or infrastructure were created. This document merely evaluates the specification and identifies (a) additional information needed before implementation is unambiguous, and (b) the software required to stand up a developer environment.

**Source evaluated:** `drop/FortiTUI — Product & Technical Specification.md` (2,997 lines, fully read).

---

## 1. Summary of the Spec as Written

FortiTUI is a keyboard-first TUI (terminal) for operational monitoring/troubleshooting of FortiGate firewalls. It is deliberately **not** a config manager, GUI/CLI replacement, SIEM, or historical analytics platform; it answers *"what is happening right now and how do I investigate it?"*.

Three operating modes are envisioned:
- **Phase 1 (build now): Direct Mode** — single binary connecting HTTPS/API straight to one FortiGate. No server, DB, Docker, or cloud. Read-only by default.
- **Phase 2 (later): FortiTUI Server/Proxy** — centralized fleet polling/cache/credentials.
- **Phase 3 (later): FortiManager backend** — device inventory/ADOM/revision awareness.

**Chosen stack (per §50):** Rust + Ratatui (TUI) + Crossterm (terminal) + Tokio (async) + reqwest (HTTP) + serde/serde_json (serialization) + YAML/TOML (config) + OS keychain (credentials) + tracing (logging).

**Central architectural requirement:** a `FortiGateBackend` trait / normalized data-model abstraction so the TUI never consumes raw FortiGate API responses (§13, §52, §53). Data is normalized into application models (`InterfaceStatus`, `SdwanMember`, `IpsecTunnel`, `BGPNeighbor`, `Route`, `SystemStatus`, `FirewallSession`). Capability model (§14) and version-aware feature detection (§40) gate UI/backend features. Polling is async with per-datatype refresh rates (§37, §38).

Phase 1 milestones (§67) run: foundation → connectivity → interfaces → SD-WAN → VPN → routing/BGP → diagnostics → polish. Definition of Done is itemized in §68.

No later appendix overrides the main body; the document is internally consistent.

---

## 2. Gaps / Open Questions — Additional Information Needed

> These are genuinely unanswered by the spec. Grouped by theme. The user should answer/append under each before or during implementation planning.

### 2.1 FortiGate API specifics (biggest gap)
1. **Exact REST API path scheme & version to target.** FortiOS REST API (`/api/v2/...`) has different monitor vs config roots, and newer releases differ. Spec says "FortiGate REST API" but never pins: the scheme (v2 monitor endpoints), the API format version, or the **specific endpoint path for every feature** (system status, interfaces+counters, sdwan/health-check, ipsec/vpn, router routes + `/monitor/router/...`, BGP neighbor state, firewall policies, sessions, diagnostics).
A1. Fortigate 8.0.0+ is first priority, after 8.0.0, we may explore 7.6.x, or iterate to support newer versions if released. in drop/FortiOS_8/ you'll find reference docs for most API's

2. **Which data is impossible (or inconsistent) over the REST API**, requiring the CLI/`execute` path. E.g. `ping`, `traceroute`, `get router info bgp neighbors`, `diagnose sniffer packet` — several of these are only available via the FortiGate CLI/execute/ssh, not the REST monitor API. The spec (§30–§33) lists these diagnostics but never states *how FortiTUI triggers them* (execute API vs SSH vs a privileged API admin). This is the single most important open implementation question.
If it can't be implemented over API, we should add those features to a to-do list, as method is still being determined.

3. **Minimum supported FortiOS version** — spec says "define a minimum" (§40) but never states it. Needs: e.g. 7.2.x, 7.4.x, or 7.6.x, and the feature/API matrix across those.
8.0.0 is the only version we're working on at the moment.

4. **API administrator permission matrix** — §57 says "document minimum permissions" but the actual FortiGate read-only admin role/API-permission set per feature (system, interface, sdwan, vpn, routing, bgp, firewall, diagnostics, packet-capture) is not enumerated.
Read only permission as full admin is good enough

5. **How the user obtains an API token** — walkthrough needed (FortiGate admin GUI/CLI steps), and whether token-only auth is sufficient or username/password token-exchange also needs support (§11).
Token only auth will be sufficient, admin will need to know how to configure and provide that token.

6. **PQC IPsec fields** (§25): which FortiOS versions expose the PQC/PPK fields, and via which endpoint. Under-specified by design — needs a version matrix.
Refer to references in drop/FortiOS_8. Only version we're concerned with.

### 2.2 Credential storage / security
7. **Concrete keychain backend per OS** — macOS Keychain, Windows Credential Manager, libsecret/`secret-service` on Linux (requires a running keyring daemon). Decision needed on the crate (`keyring`/`keyring-rs`) and the Linux fallback when no keyring daemon is present.
Provide recommendation for this following best practices.
8. **`verify_tls` custom CA** mechanism — where CA certs live (file path in profile? system store?), and how a self-signed lab cert is supplied. Spec says "custom CA certificates" supported but not *how they're provided*.
at this time no custom ca certs will be supported, provide a flag to ignore ssl checking.

### 2.3 Config & model decisions
9. **Config format final choice** — YAML vs TOML (§9 says "YAML, TOML, or another"). YAML examples are given but not binding.
YAML is fine
10. **Field-level normalized data model** — exact fields, units, and **how utilization/rates are computed from counter deltas** between polls (byte counters alone don't give Mbps). Needs an agreed math/state model (delta over sample interval).
we want to align with the data provided by the FortiGate.
11. **Rolling in-memory history window** — how many samples / how long (e.g. last 10 min at 2–5s?) before trimming. Not specified (§23, §60).
need to explore performance, but should generally target 10-60 minutes, at 1-30sec intervals.

12. **Event detection defaults & configurability** — §36 gives example thresholds (CPU>90%, mem>85%) but no formal default set or config surface.
Will need to determine after we begin processing data

### 2.4 Diagnostics & operational coupling
13. **Packet capture** — how output streams back to the TUI, size/rotation limits, and permission model. Under-specified.
TBD, lets explore the data provided via API and make recommendation.

14. **Route lookup / SD-WAN service lookup** (§27, §30) — only "where FortiGate APIs permit"; unclear which endpoint provides best-route + SD-WAN-rule resolution.
validate under drop/FortiOS_8 reference materials.

15. **SD-WAN "selected/active member" truth** — which endpoint gives current member selection across FortiOS versions.
validate under drop/FortiOS_8 refernce materials

### 2.5 Cross-platform & packaging
16. **License choice** (open-source? which?) — not stated anywhere; a required decision for a distributable product.
MIT
17. **Release/versioning & signing** — semver? code-signing on macOS/Windows? Not specified.
not at this time.
18. **Windows terminal target** — Ratatui/Crossterm work on Windows but alternate-screen/color behavior varies; confirm Windows is genuinely Phase-1-supported vs best-effort.
Linux in phase-1, mac/windows will come either in phase 1.5, or else after phase 2

### 2.6 Testing / lab
19. **Live test FortiGate availability** — is there a physical/lab FortiGate available (model(s), FortiOS version(s), credentials, network reachability from dev)? Needed for Milestone 2+ and §54 live tests.
Yes, 2 devices available - Preferred test platform at 10.0.0.2, but if live data is needed there is a production fortigate at 10.0.0.1

20. **Fixture capture workflow** — how sanitized real API responses are captured & stored (§55) — script, which model states, update cadence per FortiOS release.
provide recommendations as needed.

21. **FortiManager lab** (only if Phase 3 is exercised) — note the drop folder already contains `FortiManager-8.0.0-JSON-API-Reference.zip`, `FortiAnalyzer-8.0.0-JSON-API-Reference.zip`, and a `dvmdb__meta_fields.json` — these are exactly the Phase 3 API references. **Implicit-but-unstated**: is the intent to build Phase 3 against FortiManager 8.0 REST API from these docs? Confirm before Phase 3 work (Phase 1 does not need them).
Will be determined after phase1 completion.

### 2.7 CI / release ops
22. **CI platform & cross-compile matrix** — which CI (GitHub Actions?), and the release artifact target list from §49 (Linux x86_64/arm64, macOS arm64/x86_64, Windows x86_64) — how cross-compilation is achieved (targets + `cross`/toolchains).
23. **Dependency security** — whether `cargo-audit`/`cargo-deny` gating is expected in CI.

---

## 3. Required Software — Developer Environment

To build FortiTUI (Phase 1) as specified, the following toolchain and software are required on the primary development host (Linux, per §7). Most map 1:1 to §50's chosen stack.

### 3.1 Core language toolchain
| Tool | Purpose | Install |
|---|---|---|
| **Rust toolchain** (`rustup`, `cargo`, `rustc`) | Build/package. Pin a recent stable; decide edition (2021 or 2024) | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **rustfmt** (component) | Formatting | `rustup component add rustfmt` |
| **clippy** (component) | Linting | `rustup component add clippy` |

### 3.2 Cross-compilation targets (for §49 release matrix)
Add target toolchains and the `cross` helper (or native toolchains) to build for:
- `x86_64-unknown-linux-gnu` (native)
- `aarch64-unknown-linux-gnu` (Linux ARM64 — needs a cross C toolchain, or use `cargo cross`/Docker)
- `aarch64-apple-darwin`, `x86_64-apple-darwin` (macOS — normally built on macOS or CI runners, not cross-compiled from Linux)
- `x86_64-pc-windows-msvc` (Windows — built on Windows/CI)

```bash
rustup target add aarch64-unknown-linux-gnu x86_64-unknown-linux-gnu
cargo install cross   # containerized cross-compilation
```
Note: macOS/Windows artifacts are most reliably produced on macOS/Windows runners in CI (§3.6), not cross-compiled from Linux.

### 3.3 Rust crates (project dependencies — per spec §50)
- **TUI:** `ratatui`
- **Terminal:** `crossterm`
- **Async runtime:** `tokio` (full features)
- **HTTP:** `reqwest` (async, `rustls` or `native-tls` features for TLS; decide one)
- **Serialization:** `serde`, `serde_json`
- **Config:** `toml` or `serde_yaml` (whichever format chosen, §2.3 #9); `config`/`figment` optional
- **CLI parsing:** `clap` (≥4, derive) — for `profile add/list/remove/test`, `--profile`, `--debug`, `--version`, `--config`, `--json`
- **Credentials:** `keyring` (keyring-rs) for OS-native secure storage on all three platforms
- **Errors:** `thiserror` + `anyhow` (for the normalized error taxonomy in §39)
- **Logging:** `tracing` + `tracing-subscriber` (structured logs, redaction §46)
- **Async TLS/CA:** as required by reqwest TLS feature + custom-CA handling
- **Testing:** `insta` (snapshot tests for API fixtures §55), optionally `proptest`, `wiremock` (mock the FortiGate API for integration tests without a physical device)
- **Optional dev helpers:** `cargo-edit`, `cargo-audit`, `cargo-deny`

### 3.4 Editor / tooling
| Tool | Use |
|---|---|
| **VS Code** (or equivalent: Helix/Neovim) | Editor for Rust |
| **rust-analyzer** | Language server (IDE extension or standalone `rust-analyzer`) |
| **Git** | Version control (user already uses GitHub via CLI/`gh`) |
| **`gh` (GitHub CLI)** | Repo creation, CI/push workflows (user has this toolchain) |
| `just` or `Makefile` | Local task runner (build/test/tidy) — optional |

### 3.5 Runtime / external (for development & testing)
- **A terminal emulator** capable of full TUI rendering (alacritty/kitty/gnome-terminal; export `TERM`) for testing Ratatui rendering.
- **A development FortiGate** (lab) to exercise Milestone 2+ — API token, an API administrator with read-only access, reachable from the dev host over HTTPS.
- **TLS: self-signed/custom-CA cert** for the lab FortiGate (to exercise `verify_tls` and custom-CA paths §12).
- **For Phase 2 only (not Phase 1):** Docker + docker compose (the server ships as a Docker image §69). Not required for the Phase 1 dev environment.

### 3.6 CI / release (recommended, not strictly local)
- **GitHub Actions** — matrix build for the §49 targets; `docker/buildx` only if/when Phase 2 server image is added.
- **Release workflow** for single-binary artifacts.

### 3.7 Explicitly NOT required for Phase 1
Per §61, §69, §78: no database, no web server, no Docker (except Phase 2 server), no FortiManager, no cloud account, no external API service.

---

## 4. Confirmed Non-Goals / Already-Answered Items

To avoid re-answering what the spec already settles:
- Single binary, no server/DB/Docker/cloud for Phase 1 (§7, §61, §78).
- Language = Rust; TUI stack = Ratatui/Crossterm; async = Tokio; HTTP = reqwest (§50).
- **Read-only by default**; write ops (disable policy, clear session, restart tunnel) deferred with explicit-confirmation model (§43, §44).
- Backend abstraction (`FortiGateBackend` trait + normalized models) is mandatory, with capability + version-awareness (§13, §14, §40, §52, §53).
- Profile-based connection model with `type` field; credentials referenced, not embedded (§9, §10).
- API-token-first auth over HTTPS, Bearer header, TLS verification on by default with explicit opt-out warning (§11, §12).
- Per-datatype async refresh rates, high-level defaults given (§37, §38).
- Error taxonomy + actionable error UX, no secrets in logs (§39, §46, §47).
- IPv6 is first-class across screens (§29).
- Search, command palette, contextual help are expected features (§16, §17, §63, §64).
- JSON CLI output uses the normalized model, not raw API responses (§65, §66).
- Phase 2/3 deliberately not over-specified; driven by Phase 1 users (§69, §72).

---

## 5. Recommended Next Steps

1. **Nothing has been started** — no repo, code, containers, or provisioning.
2. Collect answers to the Section 2 questions, prioritizing **2.1 API specifics** (esp. #1, #2, #3) and **2.7 #19 (lab device)** — these gate Milestones 2–7.
3. Decide the small configuration decisions (§2.3 #9 format, §2.5 #16 license) before Milestone 1.
4. Capture a set of sanitized FortiOS 7.x API fixtures (§55) early — they unblock integration tests and normalize the data model without a live box.
5. Confirm the FortiManager 8.0 API reference zips in `drop/` are the intended Phase 3 source, but defer all Phase 3 work until Phase 1 ships (§69, §72).
6. Stand up the Phase 1 dev environment from Section 3 (Rust toolchain + crates + editor + a lab FortiGate), then begin Milestone 1.

---

## 6. Infra / Access Details Placeholder

Fill in when a live test target is provisioned. **Do not paste secrets/private keys into this document — reference "provided out-of-band" or a secrets manager.**

| Item | Value |
|---|---|
| Provider / platform | *(TBD — lab FortiGate; possibly a VM or physical device)* |
| FortiGate IP / hostname | |
| SSH/TLS port | 443 (HTTPS API) |
| FortiOS version(s) to test | (see §2.1 #3) |
| API administrator role & perms | |
| Network path from dev host | |
| TLS/custom-CA plan | |
| Phase 2/3 host (if needed later) | |

---

## 8. Open Questions → Decision Checklist

> Fill in each answer in place (append under the line, don't rewrite the question). Checked = decided. Use for Milestone 0 (pre-implementation) review.

### A. FortiGate API target
- [x] **Q1. REST API path scheme + format version** — FortiOS **8.0.0+** (v8.0.1 target), `/api/v2/monitor/...` Swagger specs in `drop/FortiOS_8/`.
- [x] **Q2. Diagnostic execution path** — anything not implementable over REST → TODO list; API-first.
- [x] **Q3. Minimum supported FortiOS version** — 8.0.0 only for now.
- [x] **Q4. API admin permission matrix** — read-only full admin is sufficient.
- [x] **Q5. API token acquisition** — token-only auth; admin provisions token.
- [x] **Q6. PQC IPsec fields** — from drop/FortiOS_8 refs (vpn.json).

### B. Credential storage / TLS
- [x] **Q7. Keychain backend per OS** — use `keyring` crate (OS-native), best-practice recommendation.
- [x] **Q8. Custom CA** — no custom CA support now; provide a flag to skip TLS verification.

### C. Config & model
- [x] **Q9. Config format** — YAML.
- [x] **Q10. Normalized field schema** — align with data FortiGate returns (see fixtures).
- [x] **Q11. Rolling history window** — target 10–60 min, 1–30s intervals (explore perf).
- [x] **Q12. Event detection defaults** — to be determined after data processing.

### D. Diagnostics & SD-WAN
- [x] **Q13. Packet capture** — TBD after exploring API data.
- [x] **Q14. Route/SD-WAN lookup endpoints** — `/router/lookup`, `/router/sdwan/routes`, `/router/policy` (validate vs refs).
- [x] **Q15. SD-WAN active-member** — validate vs refs (`/virtual-wan/*`, `/router/sdwan/routes`).

### E. Cross-platform & product
- [x] **Q16. License** — MIT.
- [x] **Q17. Release/versioning/signing** — none at this time.
- [x] **Q18. Platforms** — Linux only for Phase 1; macOS/Windows Phase 1.5+.

### F. Testing / lab
- [x] **Q19. Live test FortiGate** — test 10.0.0.2 (DEV), prod 10.0.0.1 (PROD, for data-gen).
- [x] **Q20. Fixture capture workflow** — captured live from PROD v8.0.1 into `fixtures/fortios-8.0/`.
- [x] **Q21. FortiManager Phase 3** — deferred until after Phase 1.

### G. CI / release ops
- [ ] **Q22. CI platform + cross-compile matrix** — TBD.
- [ ] **Q23. Dependency security gating** — TBD.


---

## 7. Appendix / Source Reference

Sources read in full for this analysis:
- `drop/FortiTUI — Product & Technical Specification.md` — **2,997 lines, fully read** (all seven read passes).
- `drop/FortiManager-8.0.0-JSON-API-Reference.zip` (9.1 MB) — present, **not unzipped** (relevant only to Phase 3; flagged in §2.1 #21).
- `drop/FortiAnalyzer-8.0.0-JSON-API-Reference.zip` — present, not unzipped; outside FortiTUI Phase 1 scope.
- `drop/FortiManager 8.0.0 Database Modules Device Manager Database dvmdb__meta_fields.json` — present; Phase 3 reference only.
- `drop/DNS_Observatory-PM_Eval(1).md` — a separate prior project's PM-eval (used only as a format reference, not content).
