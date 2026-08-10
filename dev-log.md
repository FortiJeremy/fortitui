# FortiTUI — Development Log

Status, architecture, and implementation notes for people working on or
following the project. **User-facing** docs (what it is, what it can do, how to
get started) live in the [README](./README.md).

## Status

Phase 1 — **Direct FortiGate mode** (**complete** as of 2026-08-09). FortiOS 8.0.x target.
Single binary, no server, no database, no Docker; read-only by default.

### Implemented (committed to `main`)

- Project skeleton: Rust, clap CLI, YAML profile system, `FORTITUI_TOKEN` auth
  (OS keychain via the `keyring` feature, off by default), `--insecure` flag.
- `FortiGateBackend` trait + `DirectBackend`, with a **normalized data model**
  (the UI never consumes raw FortiGate API responses).
- Non-TUI CLI commands: `status`, `interfaces`, `sdwan`, `vpn`, `routes`,
  `sessions`, `policies`, `lookup <destination>`, plus `profile add/list/remove/test`
  and `--json` on all data views.
- Interactive **Ratatui TUI** with screens: Dashboard, System, Interfaces
  (selectable list + live throughput graph), SD-WAN (members + health checks),
  IPsec, Routing/BGP (v4/v6 routes + BGP + route lookup), Sessions, Firewall
  Policies, and Events. Screen-aware refresh (only the active screen's data is
  polled each tick).
- In-memory **event detection**: interface up/down, SD-WAN member/active changes,
  CPU (>90%) and memory (>85%) threshold crossings.
- Sanitized FortiOS 8 monitor-API fixtures under `fixtures/fortios-8.0/` and
  fixture-driven unit tests.
- **Per-profile credentials** (2026-08-09): tokens resolve per profile —
  `profile.token` → `FORTITUI_<PROFILE>` env (derived from profile name, e.g.
  `pve-dev` → `FORTITUI_PVE_DEV`) → OS keychain `fortitui/<profile>` (`keyring`
  feature) → global `FORTITUI_TOKEN` fallback. New `credential set|unset <name>`
  manage keychain entries. Previously a single global `FORTITUI_TOKEN` required
  re-exporting when switching tokens between profiles.

### Next milestones

- **Phase 1 complete (2026-08-09, commit `1f7c9b4`).** Shipped in the final batch:
  SD-WAN rolling trend (C7), IPsec cryptography detail incl. PQC (C9), Diagnostics
  screen (C13), global `/` search (D1), `:` command palette (D2), contextual help (D3),
  `--json` for all views (D4), error UX (D5), client error-branch tests (A5).
- Known limitation: ping/traceroute/DNS/sniffer are CLI-only on FortiOS 8 (gap-analysis
  Q2) and are surfaced as NOT AVAILABLE in the Diagnostics screen.
- Phase 2: FortiTUI Server/Proxy fleet mode. Phase 3: FortiManager backend.

## Architecture

```
TUI (Ratatui)
    │
    ▼
Backend Interface (FortiGateBackend trait)
    │
    ├── DirectBackend       (Phase 1 — implemented)
    ├── ServerBackend       (Phase 2, planned)
    └── FortiManagerBackend (Phase 3, planned)
```

Data is normalized into application models (never raw FortiGate API responses),
so the UI is independent of how devices are accessed. Raw responses are captured
as sanitized fixtures in `fixtures/fortios-8.0/` for offline development and tests.

See `docs/spec.md` (product & technical specification) and `docs/gap-analysis.md`
(answered decision checklist) for the authoritative design.

## Testing

```bash
cargo test                                   # unit tests driven by fixtures/fortios-8.0/
cargo clippy --all-targets -- -D warnings    # lints — must be zero warnings (CI enforce)
cargo fmt --check                            # formatting — must be clean (CI enforce)
```

CI (GitHub Actions) runs `fmt --check`, `clippy -D warnings`, `build --release`,
and `test` on every push to `main`.

## Dev environment notes

- The project builds/lints/tests inside the `hermes-fortitui-dev` container; the
  repo checkout is at `/workspace` there. See the session handoff docs for the
  exact toolchain and device details.
- Known lab quirks are tracked in `/opt/data/FORTITUI_DEV_HANDOFF.md` (e.g. the
  `/firewall/sessions` endpoint requires a `count` query param; the backend trait
  is RPITIT because async-fn-in-trait isn't object-safe; the TUI needs a real TTY
  to render).
