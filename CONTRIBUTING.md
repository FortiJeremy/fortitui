# Contributing to FortiTUI

Thanks for your interest in contributing. This project is a keyboard-driven terminal
console for FortiGate, currently in **Phase 1** (direct FortiGate connectivity).

## Project docs (read first)

- `README.md` — quick start, usage, architecture overview
- `docs/spec.md` — the full product & technical specification (single source of truth)
- `docs/gap-analysis.md` — the answered decision checklist that guides implementation

## Dev environment

FortiTUI is written in **Rust** using Ratatui (TUI), Crossterm, Tokio, and reqwest.
Linux is the primary Phase‑1 target.

**Required:** rustc 1.75+ (tested on 1.97), cargo, rustfmt, clippy.

## Getting started

```bash
cargo build              # compile (debug)
cargo test               # fixture-driven normalizer tests
cargo fmt --check        # must be clean (CI enforces)
cargo clippy --all-targets -- -D warnings   # must have zero warnings (CI enforces)
```

## Running against a FortiGate (Phase 1)

```bash
fortitui profile add          # interactive: host, port, API token
FORTITUI_TOKEN=<token> fortitui status --profile <name> --insecure
FORTITUI_TOKEN=<token> fortitui interfaces --profile <name> --insecure
```

- Provide your API token via the `FORTITUI_TOKEN` environment variable, or use the
  `keyring` feature (`cargo build --features keyring`) for the OS keychain.
- `--insecure` disables TLS verification — **lab/testing only, never production**.
- Don't commit any real tokens, credentials, or private keys.

## Code wants (per the spec)

- **Normalized data model** — the TUI must never consume raw FortiGate API responses.
  Add/adjust fields in `src/models/` and parsers in `src/fortigate/normalize.rs`.
- **Backend abstraction** — feature logic goes behind the `FortiGateBackend` trait
  (`src/backend/`), never hard-coded to one connection type.
- **Read-only by default** — no configuration modification without a compelling,
  safe use case.
- **No secrets in logs** — API tokens and headers must be redacted.
- **Tests with fixtures** — capture sanitized real API responses under
  `fixtures/fortios-8.0/` and drive normalizer tests from them (see `tests/normalize.rs`).

## Fixtures & sanitization

Captured API responses are good for development, but this repo is **public**. When you
add a fixture or example, strip all personally identifiable / environment-specific data
(device hostnames, serial numbers, real IPs, MACs, UUIDs) and replace them with generic
placeholders (e.g. hostname `FortiGate-A`, TEST‑NET IP ranges). Review your PR diff for
real identifiers before pushing.

## CI

GitHub Actions runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
`cargo build --release`, and `cargo test` on every push to `main`. Make sure all four
pass locally before pushing.

## License

MIT (see `LICENSE`).
