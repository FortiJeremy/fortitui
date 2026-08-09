# FortiTUI

A fast, keyboard-driven **terminal console for FortiGate** — operational
monitoring, troubleshooting, and diagnostics right from your terminal.

> FortiTUI answers the question *"What is happening on my FortiGate right now,
> and how do I investigate it?"* It is deliberately **not** a configuration
> manager, GUI/CLI replacement, historical analytics platform, or SIEM. It
> complements existing FortiOS tooling rather than replacing it.

## Features

Read-only, keyboard-first visibility into one or more FortiGates from a single
binary — **no server, no database, no Docker**.

**Interactive terminal UI** (run `fortitui`):
- **Dashboard** — overall health with a situational-awareness strip
- **System** — model, serial, version, uptime, CPU/memory/disk, sessions
- **Interfaces** — link state, addresses, counters, errors; select one for a live throughput graph
- **SD-WAN** — members (state / latency / jitter / loss / SLA) and health checks, active member highlighted
- **IPsec** — tunnel list with Phase 1/2 state, IKE version, traffic, uptime
- **Routing / BGP** — IPv4 + IPv6 routes, BGP neighbors, and interactive **route lookup**
- **Sessions** — active firewall sessions (source/dest, protocol, policy, traffic)
- **Firewall Policies** — policy counters (hits / bytes / sessions)
- **Events** — in-memory state-transition log (interface up/down, SD-WAN changes, CPU/memory thresholds)

**Non-TUI CLI** (for scripts and automation; append `--json` for machine-readable
output):
- `fortitui status`, `interfaces`, `sdwan`, `vpn`, `routes`, `sessions`, `policies`
- `fortitui lookup <destination>` — best-route lookup
- `fortitui profile add/list/remove/test`

Use the same binary against **multiple FortiGate profiles**.

## Requirements

- FortiOS **8.0.x**
- An API token from a **read-only full-admin** account
- Linux (primary Phase 1 target; macOS/Windows planned for Phase 1.5+)
- Rust toolchain (>= 1.75) only if building from source

## Install / Build

```bash
cargo build --release
./target/release/fortitui --version
```

## Quick start

1. **Create a profile** — either interactively:
   ```bash
   fortitui profile add
   ```
   or add `profiles.yaml` under `~/.config/fortitui/`:
   ```yaml
   profiles:
     branch-01:
       type: direct
       host: 10.0.0.1
       port: 443
       verify_tls: true
       credential: branch-01
   ```

2. **Provide your API token** (via per-profile env, global env, or keychain — see
   [Configuration & Secrets](#configuration--secrets)):
   ```bash
   export FORTITUI_BRANCH_01=<token>   # per-profile: FORTITUI_<PROFILE>
   # or the legacy single-token fallback:
   # export FORTITUI_TOKEN=<token>
   ```

3. **Launch the terminal UI** (offers your single profile automatically, or pass one):
   ```bash
   fortitui --profile branch-01
   ```

   Or use the CLI non-interactively:
   ```bash
   fortitui status --profile branch-01
   fortitui status --profile branch-01 --json
   fortitui interfaces --profile branch-01
   fortitui lookup 8.8.8.8 --profile branch-01
   ```

## Configuration & Secrets

Profiles live in `~/.config/fortitui/profiles.yaml` (platform-conventional), and
contain **no secrets**. API tokens are resolved **per profile** in this order:

1. `token:` set explicitly in the profile YAML (discouraged — plaintext)
2. `FORTITUI_<PROFILE>` environment variable — derived from the profile name,
   e.g. profile `branch-01` → `FORTITUI_BRANCH_01`, profile `leatherleaf` →
   `FORTITUI_LEATHERLEAF`. This is the preferred way to supply a token per
   profile, so `fortitui --profile X` just works without re-exporting.
3. OS keychain entry `fortitui/<profile>` (when built with the `keyring`
   feature). Store one with:
   ```bash
   fortitui credential set branch-01   # prompts for the token
   fortitui credential unset branch-01 # remove it
   ```
4. legacy global `FORTITUI_TOKEN` environment variable (single shared token)

The keychain feature is opt-in: `cargo build --features keyring` (requires
system keyring libraries on Linux). When built without it, use the environment
variables above.

- FortiTUI is **read-only by default** and does not modify FortiOS configuration.
- `--insecure` disables TLS certificate verification — **lab/testing only, never
  in production.**

## Compatibility

- FortiOS **8.0.x** (monitor API). See `drop/FortiOS_8/` for the API reference.
- Linux (primary). macOS/Windows planned for Phase 1.5+.

## Documentation

- `docs/spec.md` — full product & technical specification
- `docs/gap-analysis.md` — implementation decision checklist
- `dev-log.md` — development status, architecture, and milestone log

## License

MIT
