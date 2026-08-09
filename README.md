# FortiTUI

A fast, keyboard-driven **terminal console for FortiGate** — operational monitoring, troubleshooting, and diagnostics.

> **FortiTUI is a terminal-based operational console for FortiGate.** It answers
> *"What is happening right now, and how do I investigate it?"* — not another
> historical analytics platform, GUI/CLI replacement, or config manager.

## Status

**Phase 1 — Direct FortiGate mode (in development).** FortiOS 8.0.x target.
Single binary, no server, no database, no Docker required. Read-only by default.

Working so far (non-TUI CLI backed by a real client):
- `fortitui status` — system status (model, serial, FortiOS, CPU/mem/disk, sessions)
- `fortitui interfaces` — interface list with link state, IP, RX/TX counters
- `fortitui sdwan` — SD-WAN members
- `fortitui routes` — IPv4 routing table
- `fortitui profile add/list/remove/test` — profile management
- `--json` on any data command for automation

The interactive TUI (dashboard, per-view screens) and remaining views (VPN detail,
BGP, sessions, diagnostics, events) are the next milestones.

## Architecture

```
TUI (Ratatui)               <-not yet wired
    │
    ▼
Backend Interface (FortiGateBackend trait)
    │
    ├── DirectBackend       (Phase 1 — implemented)
    ├── ServerBackend       (Phase 2, planned)
    └── FortiManagerBackend (Phase 3, planned)
```

Data is normalized into application models (never raw FortiGate API responses),
so the TUI is independent of how devices are accessed. Raw responses are captured
as sanitized fixtures in `fixtures/fortios-8.0/` for offline development/tests.

See `docs/spec.md` (product & technical specification) and `docs/gap-analysis.md`
(decision checklist) for detail.

## Building

Requires the Rust toolchain (rustc 1.75+; tested on 1.97).

```bash
cargo build --release
./target/release/fortitui --version
```

## Quick start (Phase 1)

```bash
# create a profile (prompts for host, port, API token)
fortitui profile add

# or drop a profiles.yaml in ~/.config/fortitui/:
#   profiles:
#     leatherleaf:
#       type: direct
#       host: 10.0.0.1
#       port: 443
#       verify_tls: false   # lab only; never disable in production
#       credential: leatherleaf

# run a non-TUI command against it (token via env unless keyring enabled)
FORTITUI_TOKEN=<token> fortitui status --profile leatherleaf --insecure
FORTITUI_TOKEN=<token> fortitui status --profile leatherleaf --insecure --json
```

## Configuration & Secrets

Profiles live in `~/.config/fortitui/profiles.yaml` (platform-conventional).
**Secrets are never stored in the repo.** The API token is supplied via the
`FORTITUI_TOKEN` environment variable, or stored in the OS keychain when built
with the `keyring` feature (`cargo build --features keyring` — requires system
keyring libraries on Linux).

`--insecure` disables TLS certificate verification — use only for lab/testing,
never in production.

## Testing

```bash
cargo test          # unit tests driven by fixtures/fortios-8.0/
cargo clippy        # lints
```

## Compatibility

- FortiOS **8.0.x** (monitor API). See `drop/FortiOS_8/` for the API reference.
- Linux (primary target for Phase 1). macOS/Windows planned for Phase 1.5+.

## License

MIT
