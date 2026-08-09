# FortiTUI — Product & Technical Specification

**Project:** FortiTUI  
**Status:** Initial Specification  
**Version:** 0.1  
**Primary implementation target:** Phase 1 — Direct FortiGate connectivity  
**Future phases:** Phase 2 — Proxy/Fleet Mode; Phase 3 — FortiManager Integration

---

# 1. Executive Summary

FortiTUI is a terminal user interface (TUI) designed specifically for operational management, monitoring, troubleshooting, and diagnostics of FortiGate firewalls.

The project is intentionally designed around two distinct operating models:

1. **Direct Mode** — FortiTUI connects directly to a single FortiGate.
2. **Server/Proxy Mode** — FortiTUI connects to a FortiTUI server which manages connectivity to multiple FortiGates.
3. **FortiManager Mode** — FortiTUI uses FortiManager as an additional management/data source for environments where FortiManager already provides centralized Fortinet management.

The initial implementation should focus heavily on **Direct Mode**.

Phase 1 should be a complete, useful product on its own. A user should be able to install a single FortiTUI binary, create a profile for a FortiGate, and immediately use FortiTUI as a powerful terminal-based operational console.

Phase 2 and Phase 3 should not be over-designed before real users have exercised Phase 1. The architecture should provide clean extension points for those modes, but their exact functionality should be driven by real-world requirements discovered during Phase 1 deployments.

---

# 2. Product Goals

## 2.1 Primary Goal

Provide a fast, keyboard-driven operational interface for FortiGate administrators.

FortiTUI should make it significantly easier to answer questions such as:

- Is the FortiGate healthy?
- What is consuming resources?
- Are my WAN links healthy?
- Which SD-WAN member is currently being used?
- Why did SD-WAN select a particular link?
- Are IPsec tunnels up?
- Are BGP neighbors established?
- What routes are installed?
- What interfaces are experiencing errors?
- Are sessions increasing unexpectedly?
- What changed recently?
- Can I quickly run a diagnostic against a destination?
- Is the firewall itself experiencing a problem or is the problem upstream?

The product should favor **rapid operational understanding** over exposing every possible FortiOS configuration option.

---

# 3. Non-Goals

FortiTUI is not initially intended to be:

- A replacement for the FortiGate GUI
- A replacement for the FortiGate CLI
- A configuration management platform
- A FortiManager replacement
- A complete SIEM
- A long-term metrics database
- A historical observability platform
- A packet analysis replacement for Wireshark
- A general-purpose network management system supporting multiple vendors

The product should complement existing Fortinet tooling rather than attempt to replace it.

---

# 4. Relationship to Existing Tooling

FortiTUI should occupy a different operational role from historical observability systems.

For example:

```text
                         TIME
                          │
                          │
       Historical         │        Real-Time
       Analysis           │        Operations
                          │
                          ▼
                 ┌─────────────────┐
                 │      Argus      │
                 │                 │
                 │ DNS             │
                 │ DHCP            │
                 │ FortiGuard      │
                 │ Grafana         │
                 │                 │
                 │ Historical      │
                 │ analysis        │
                 └─────────────────┘

                                      ┌─────────────────┐
                                      │    FortiTUI     │
                                      │                 │
                                      │ FortiGate state │
                                      │ SD-WAN          │
                                      │ VPN             │
                                      │ Routing         │
                                      │ Interfaces      │
                                      │ Diagnostics     │
                                      │                 │
                                      │ Real-time       │
                                      │ operations      │
                                      └─────────────────┘
```

FortiTUI should primarily answer:

> **"What is happening right now, and how do I investigate it?"**

It should not attempt to become another historical analytics platform.

---

# 5. High-Level Architecture

The architecture should support three connection modes while keeping the TUI itself largely independent of how data is obtained.

```text
                              ┌───────────────────────┐
                              │       FortiTUI        │
                              │                       │
                              │  TUI / Presentation   │
                              └───────────┬───────────┘
                                          │
                                  Backend abstraction
                                          │
                       ┌──────────────────┼──────────────────┐
                       │                  │                  │
                       ▼                  ▼                  ▼
               Direct Backend      Server Backend     FMG Backend
                       │                  │                  │
                       ▼                  ▼                  ▼
                 FortiGate          FortiTUI Server    FortiManager
                                          │
                            ┌─────────────┼─────────────┐
                            ▼             ▼             ▼
                         FortiGate     FortiGate     FortiGate
```

The TUI must not be tightly coupled to the FortiGate REST API.

Instead, the application should have an internal abstraction layer.

For example:

```text
TUI
 │
 ▼
Backend Interface
 │
 ├── DirectBackend
 │
 ├── ServerBackend
 │
 └── FortiManagerBackend
```

Phase 1 implements only:

```text
DirectBackend
```

The other interfaces may initially be stubs, traits/interfaces, or conceptual architecture only.

---

# 6. Phase Roadmap

## Phase 1 — Direct FortiGate

**Priority: Highest**

FortiTUI connects directly to one FortiGate.

```text
fortitui
    │
    │ HTTPS / FortiGate API
    ▼
FortiGate
```

Characteristics:

- Single device
- Local configuration profile
- Direct authentication
- No server component required
- No database required
- No Docker required
- Fully functional standalone binary
- Focus on operational visibility and troubleshooting

This phase should receive the majority of implementation effort.

---

## Phase 2 — FortiTUI Server / Proxy

**Priority: Later**

A centralized service manages multiple FortiGate connections.

```text
FortiTUI
    │
    │ HTTPS
    ▼
FortiTUI Server
    │
    ├── FortiGate 1
    ├── FortiGate 2
    ├── FortiGate 3
    └── FortiGate N
```

Potential functionality:

- Fleet management
- Centralized credentials
- Concurrent polling
- State caching
- Cross-device views
- Fleet health
- Alerts/events
- Multi-user access
- Shared configuration
- Short-term state retention

The exact feature set should be determined from Phase 1 user feedback.

---

## Phase 3 — FortiManager

**Priority: Later**

FortiTUI gains FortiManager as another possible backend.

Potential architecture:

```text
FortiTUI
    │
    ▼
FortiManager
    │
    ├── ADOM
    │    ├── FortiGate
    │    ├── FortiGate
    │    └── FortiGate
    │
    └── ADOM
         ├── FortiGate
         └── FortiGate
```

Potential functionality may include:

- Device inventory
- ADOM awareness
- Device status
- Configuration status
- Revision information
- Install status
- Workspace status
- Policy/configuration context
- Centralized device discovery

However, **Phase 3 should not be specified beyond architectural compatibility at this stage.**

Real FortiManager users should determine what functionality provides the most value.

---

# 7. Phase 1 — Direct FortiGate

## 7.1 Design Philosophy

The first version should feel like a native Unix operational tool.

A user should be able to install:

```bash
fortitui
```

and begin using it.

No:

- Docker
- Database
- Web server
- External API service
- Cloud service
- Account registration

should be required.

The application should be usable from:

- Linux
- macOS
- Windows

with Linux being the primary development and operational target.

---

# 8. Invocation

Basic invocation:

```bash
fortitui
```

If exactly one configured profile exists, FortiTUI may automatically offer to connect to it.

Explicit profile:

```bash
fortitui --profile fortigate-1
```

Interactive profile selection:

```bash
fortitui --profile-select
```

Configuration management:

```bash
fortitui profile list
fortitui profile add
fortitui profile remove fortigate-1
fortitui profile test fortigate-1
```

Diagnostics:

```bash
fortitui --version
fortitui --debug
fortitui --config
```

The CLI should remain useful even though the primary interface is a TUI.

---

# 9. Profile System

Profiles are a fundamental part of FortiTUI.

A profile represents a connection to one FortiGate in Phase 1.

Example:

```yaml
profiles:

  fortigate-1:
    type: direct
    host: 10.10.10.1
    port: 443
    verify_tls: true
    credential: fortigate-1

  branch-01:
    type: direct
    host: 192.168.20.1
    port: 443
    verify_tls: true
    credential: branch-01
```

The exact configuration format may be YAML, TOML, or another appropriate format.

The format should be:

- Human-readable
- Versionable
- Easy to edit
- Explicit
- Extensible
- Capable of supporting future profile types

The configuration schema should therefore include a `type` field.

Example:

```yaml
type: direct
```

Future:

```yaml
type: server
```

or:

```yaml
type: fortimanager
```

---

# 10. Credential Handling

Credentials should not be casually stored in plaintext configuration files.

The preferred architecture is:

```text
Profile
   │
   └── credential reference
             │
             ▼
       Credential store
```

The profile should ideally contain:

```yaml
credential: fortigate-1
```

rather than:

```yaml
username: admin
password: password123
```

Possible credential mechanisms:

1. OS-native credential/keychain storage
2. Environment variables
3. External secret providers
4. Explicit token input
5. Configuration file as a fallback

The initial implementation should prioritize:

- API token authentication
- Secure local storage
- Clear documentation of credential security

---

# 11. FortiGate Authentication

Phase 1 should prioritize the FortiGate REST API.

The implementation should support API tokens where available.

The application should not require an administrative username/password unless a specific API workflow requires it.

Recommended authentication model:

```text
FortiTUI
   │
   │ HTTPS
   │ Authorization: Bearer <token>
   ▼
FortiGate API
```

The application should clearly distinguish:

- Authentication failure
- Authorization failure
- Network failure
- TLS failure
- API endpoint failure
- FortiGate-side error

Example:

```text
Unable to connect to Branch-01

Network:       OK
TLS:           OK
Authentication: FAILED
Authorization:  Unknown

FortiGate returned HTTP 401 Unauthorized.
```

This is much more useful than:

```text
Connection failed.
```

---

# 12. TLS

HTTPS should be the default.

The application should support:

- Valid certificate verification
- Custom CA certificates
- Certificate verification disablement for lab/testing environments

Disabling verification should require an explicit configuration setting.

For example:

```yaml
verify_tls: false
```

The UI should make the security implication obvious.

Example warning:

```text
WARNING

TLS certificate verification is disabled for this profile.

Connection may be vulnerable to man-in-the-middle attacks.

Continue? [y/N]
```

---

# 13. Internal Backend Abstraction

The most important architectural requirement is that the TUI should not directly consume raw FortiGate API responses.

Conceptually:

```text
FortiGate API
      │
      ▼
Direct FortiGate Client
      │
      ▼
Normalized Data Model
      │
      ▼
Backend Interface
      │
      ▼
TUI
```

For example:

```text
FortiGate API:

{
    "results": {
        ...
    }
}
```

should never be passed directly into a TUI widget.

Instead:

```text
InterfaceStatus
SDWANMember
IPsecTunnel
BGPNeighbor
Route
SystemStatus
FirewallSession
```

should represent the application-level data.

This will dramatically simplify Phase 2 and Phase 3.

---

# 14. Capability Model

Each backend should expose capabilities.

Example:

```text
Capabilities:

system
interfaces
sdwan
ipsec
routing
bgp
ospf
firewall
sessions
diagnostics
logs
```

The TUI can use these capabilities to determine which functionality should be exposed.

For Phase 1, most capabilities will be available directly from the FortiGate.

Future backends may have incomplete capabilities.

For example:

```text
Direct FortiGate:

✓ Interfaces
✓ SD-WAN
✓ IPsec
✓ Routing
✓ Diagnostics

Future FortiManager:

✓ Device inventory
✓ Device status
? SD-WAN
? Diagnostics
```

This prevents the UI from assuming that every backend can perform every operation.

---

# 15. Main UI

The default screen should provide an operational summary.

Example:

```text
┌──────────────────────────────────────────────────────────────────────┐
│ FortiTUI                         Branch-01          v7.6.5           │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│ SYSTEM                                                               │
│ CPU       18%          Memory       61%          Uptime    43d 12h   │
│ Sessions  18,421       Disk         42%          HA        Primary   │
│                                                                      │
│ WAN                                                                  │
│ WAN1      UP           942 Mbps ↓    184 Mbps ↑                     │
│ WAN2      UP           187 Mbps ↓     42 Mbps ↑                     │
│                                                                      │
│ SD-WAN                                                               │
│ WAN1      ACTIVE       18ms latency    0.0% loss    SLA PASS         │
│ WAN2      STANDBY      42ms latency    0.3% loss    SLA PASS         │
│                                                                      │
│ VPN                                                                  │
│ IPsec     14 / 14 UP                                                 │
│                                                                      │
│ ROUTING                                                              │
│ BGP       2 / 2 ESTABLISHED                                          │
│ OSPF      4 neighbors                                                 │
│                                                                      │
├──────────────────────────────────────────────────────────────────────┤
│ [i] Interfaces  [s] SD-WAN  [v] VPN  [r] Routing  [d] Diagnostics   │
└──────────────────────────────────────────────────────────────────────┘
```

The exact visual design is subject to implementation experimentation.

The important requirement is **information density without becoming visually overwhelming**.

---

# 16. Navigation Philosophy

Navigation should be keyboard-first.

Mouse support may be implemented but should not be required.

Recommended global keys:

```text
q       Quit
?       Help
Esc     Back
Enter   Select
Tab     Next panel
Shift+Tab Previous panel
↑/↓     Navigate
←/→     Navigate
r       Refresh
/       Search
```

Additional screen-specific shortcuts may be assigned.

The application should always provide a discoverable help mechanism.

---

# 17. Command Palette

A command palette would be highly valuable.

Example:

```text
┌─ Command ─────────────────────────────────┐
│ > sdwan                                   │
│                                           │
│   Open SD-WAN                             │
│   Show SD-WAN health checks               │
│   Show SD-WAN members                     │
│   Run SD-WAN route lookup                 │
└───────────────────────────────────────────┘
```

This allows experienced operators to move quickly without memorizing every navigation key.

---

# 18. System Overview

The system page should expose:

- Hostname
- Serial number
- Model
- FortiOS version
- Build
- System uptime
- CPU utilization
- Memory utilization
- Disk utilization
- Session count
- VDOM information
- HA state
- HA peer state
- License/support state where available

Example:

```text
SYSTEM

Hostname       Branch-01
Model          FortiGate-121G
Serial         FGT...
FortiOS        7.6.5
Build          xxxx
Uptime         43d 12h
CPU            18%
Memory         61%
Sessions       18,421
VDOMs          3
HA             Primary
```

---

# 19. Interface View

The interface screen should be one of the primary operational views.

Required information:

- Interface name
- Alias
- Type
- Administrative state
- Link state
- IP address
- IPv6 address
- Link speed
- Duplex
- MTU
- RX bytes
- TX bytes
- RX packets
- TX packets
- Errors
- Drops
- Utilization

Example:

```text
INTERFACES

NAME       STATE   ADDRESS          SPEED       RX        TX      ERR
────────────────────────────────────────────────────────────────────
wan1       UP      203.0.113.10     1 Gbps      942M      184M      0
wan2       UP      198.51.100.10    500 Mbps    187M       42M      3
port1      UP      10.10.1.1/24     1 Gbps      220M      180M      0
port2      DOWN    --               --          --        --        --
```

Selecting an interface should open a detailed view.

---

# 20. Interface Detail

The detailed interface view should provide:

- Configuration
- Addresses
- Counters
- Errors
- Traffic rate
- Link information
- Related routes
- Related SD-WAN membership
- Related firewall policy/session information where feasible

A small real-time throughput graph would be highly desirable.

Example:

```text
WAN1 — 1 Gbps

RX     942 Mbps
TX     184 Mbps

RX ┤       ╭──────────╮
   │   ╭───╯          ╰──
   │───╯
   └────────────────────────

TX ┤      ╭────╮
   │──────╯    ╰──────────
   └────────────────────────

Errors: 0
Drops:  12
MTU:    1500
```

The graph should be based on recently sampled values rather than historical database data.

---

# 21. SD-WAN

SD-WAN should be considered a **first-class feature**.

This is one of the primary reasons to build FortiTUI specifically for FortiGate rather than as a generic network TUI.

The SD-WAN screen should expose:

- SD-WAN members
- Member state
- Zone
- Interface
- Gateway
- Health-check state
- Latency
- Jitter
- Packet loss
- SLA state
- Current traffic
- Current preferred/selected member
- Failover state
- Performance metrics
- Related rules

Example:

```text
SD-WAN MEMBERS

MEMBER     STATE     LATENCY   JITTER   LOSS     SLA       TRAFFIC
────────────────────────────────────────────────────────────────────
wan1       ACTIVE       18ms     1ms     0.0%     PASS       942M
wan2       STANDBY      42ms     8ms     0.3%     PASS       187M
lte1       DOWN          --      --       --      FAIL         --
```

---

# 22. SD-WAN Health Checks

Health checks should have their own view.

Example:

```text
HEALTH CHECKS

CHECK             MEMBER    LATENCY   JITTER   LOSS   STATUS
──────────────────────────────────────────────────────────────
internet-google   wan1       18ms      1ms    0.0%   PASS
internet-google   wan2       42ms      8ms    0.3%   PASS
cloudflare         wan1       16ms      2ms    0.0%   PASS
cloudflare         wan2       41ms      7ms    0.2%   PASS
```

Selecting a health check should expose additional information.

---

# 23. SD-WAN Performance History

Although FortiTUI is not intended to be a historical analytics platform, the application should retain a **small rolling in-memory history**.

This enables:

- Recent latency trend
- Recent packet-loss trend
- Recent jitter trend
- Recent throughput
- Recent state transitions

The purpose is operational diagnosis.

Example:

> WAN2 was healthy 10 minutes ago but has degraded steadily.

It is not intended to answer:

> What was WAN2's average latency over the last six months?

That belongs in a historical observability system.

---

# 24. VPN / IPsec

The VPN view should show:

- Tunnel name
- Phase 1 state
- Phase 2 state
- Remote gateway
- Local gateway
- IKE version
- Encryption
- Authentication
- Rekey timers
- Traffic counters
- Last state change
- Tunnel uptime where available

Example:

```text
IPSEC TUNNELS

NAME              STATE      IKE      TRAFFIC       UPTIME
────────────────────────────────────────────────────────────
Branch-01         UP         IKEv2    18.4 GB       12d
Branch-02         UP         IKEv2    9.2 GB         4d
Branch-03         DOWN       IKEv2    --              --
```

Selecting a tunnel should provide detailed Phase 1/Phase 2 information.

---

# 25. PQC Visibility

Because modern FortiGate deployments may use post-quantum cryptographic features, FortiTUI should eventually expose relevant IPsec cryptographic configuration.

The UI should distinguish:

- Classical key exchange
- PQC key exchange
- PQC preshared key mechanisms
- PQC signature/authentication mechanisms
- Encryption algorithm
- Authentication/integrity algorithm

The exact fields should be based on the FortiGate API capabilities available for the supported FortiOS versions.

The UI should avoid implying that a VPN is "PQC secure" based on one enabled feature.

Instead, expose the actual mechanisms individually.

Example:

```text
IPSEC CRYPTOGRAPHY

IKE Version              IKEv2
Key Exchange             ECDH + PQC
Authentication           Certificate
Signature                ECDSA
PQC Signature            Disabled
PQC PPK                  Enabled
Encryption               AES-256-GCM
```

---

# 26. Routing

The routing section should support:

- IPv4 routing table
- IPv6 routing table
- Route lookup
- Route details
- Administrative distance
- Metric
- Protocol
- Next hop
- Interface
- Active/inactive state

Example:

```text
ROUTES

PREFIX             PROTOCOL    NEXT-HOP          INTERFACE   DIST
──────────────────────────────────────────────────────────────────
0.0.0.0/0          static      203.0.113.1       wan1          10
10.0.0.0/8         BGP         10.255.0.1        vpn1          20
192.168.20.0/24    connected   --                lan           0
```

---

# 27. Route Lookup

Route lookup should be a first-class diagnostic operation.

User enters:

```text
Destination: 10.20.30.40
```

FortiTUI should show:

```text
ROUTE LOOKUP

Destination: 10.20.30.40

Selected route:
  Prefix:      10.20.0.0/16
  Protocol:    BGP
  Next-hop:    10.255.0.1
  Interface:   vpn-branch
  Distance:    20
  Metric:      100

SD-WAN:
  Rule:        Branch traffic
  Member:      wan1
```

Where FortiGate APIs permit this level of information.

---

# 28. BGP

BGP should expose:

- Neighbor state
- Local AS
- Remote AS
- Router ID
- Uptime
- Received prefixes
- Advertised prefixes
- State
- Last state change
- Address family
- IPv4/IPv6 status

Example:

```text
BGP NEIGHBORS

NEIGHBOR       AS       STATE          RX        TX
──────────────────────────────────────────────────────
10.255.0.1     65001    ESTABLISHED    142       138
10.255.0.2     65002    ESTABLISHED    87        92
```

---

# 29. IPv6

IPv6 should not be treated as an afterthought.

All applicable screens should support IPv6.

This includes:

- Interface addresses
- Routing
- BGP
- SD-WAN where applicable
- Diagnostics
- DNS
- IPsec
- Health checks

The application should not assume IPv4-only environments.

---

# 30. Diagnostics

Diagnostics are one of the highest-value features for a TUI.

The goal is to make common troubleshooting actions immediately available.

Potential diagnostics:

- Ping
- IPv6 ping
- Traceroute
- IPv6 traceroute
- DNS lookup
- Route lookup
- SD-WAN service lookup
- IPsec status
- Session lookup
- Packet capture
- Interface diagnostics

Example:

```text
DIAGNOSTICS

1. Ping
2. Traceroute
3. DNS Lookup
4. Route Lookup
5. SD-WAN Lookup
6. Session Lookup
7. IPsec Tunnel
8. Packet Capture
```

---

# 31. Ping

Ping should allow:

- IPv4 destination
- IPv6 destination
- Source interface/address
- Count
- Packet size
- DF bit where supported
- Timeout

Example:

```text
PING

Destination: 8.8.8.8
Source:      auto
Count:       10
Packet size: 1500

[Run]
```

Output should be parsed and summarized.

```text
RESULT

Packets: 10
Received: 10
Loss: 0.0%

Min: 18ms
Avg: 20ms
Max: 24ms
Jitter: 2ms
```

---

# 32. Traceroute

Traceroute should expose:

- Destination
- IPv4/IPv6
- Source
- Protocol/options where supported

Output should be presented in a structured table rather than simply dumping CLI text.

---

# 33. DNS Lookup

DNS diagnostics should support:

- A
- AAAA
- CNAME
- MX
- TXT
- NS
- PTR
- Custom record types where feasible

The user should be able to specify:

```text
Hostname:
Record type:
Server:
Source:
```

---

# 34. Session Visibility

The session view should provide:

- Source
- Destination
- Source port
- Destination port
- Protocol
- Policy
- NAT information
- Interface
- Bytes
- Packets
- Session age

The UI should allow filtering.

Example:

```text
/ 10.20.30.40
```

or:

```text
/ tcp
```

or:

```text
/ policy=123
```

---

# 35. Firewall Policy Visibility

Phase 1 should focus on **operational information**, not full configuration editing.

Useful information:

- Policy ID
- Name
- Source
- Destination
- Service
- Action
- Hit count
- Bytes
- Sessions
- Last used

Example:

```text
POLICIES

ID   NAME                 ACTION    SESSIONS     BYTES
─────────────────────────────────────────────────────────
10   LAN → Internet       ACCEPT    1,284        4.2 GB
20   VPN → Server         ACCEPT      421        812 MB
30   Guest → LAN          DENY         87         24 KB
```

---

# 36. Event Detection

Even without a persistent database, FortiTUI should identify state transitions.

Examples:

```text
WAN1 changed UP → DOWN
WAN2 changed STANDBY → ACTIVE
IPsec Branch-03 changed UP → DOWN
BGP neighbor changed ESTABLISHED → IDLE
CPU exceeded 90%
Memory exceeded 85%
```

The application should present recent events.

Example:

```text
RECENT EVENTS

17:42  WAN2 packet loss exceeded SLA threshold
17:40  IPsec Branch-03 went DOWN
17:38  BGP neighbor 10.255.0.2 established
17:31  WAN1 became active SD-WAN member
```

Events should initially be held in memory.

Persistence should be deferred until requirements justify it.

---

# 37. Refresh Architecture

FortiTUI should not refresh everything every second.

Different data types should have different refresh rates.

Example:

```text
System status:        5 sec
Interface counters:   2 sec
SD-WAN state:         2 sec
SD-WAN health:        5 sec
VPN state:             5 sec
BGP state:            10 sec
Routing table:        30 sec
Policy information:   60 sec
Static configuration: 300 sec
```

These are initial defaults and should be configurable.

The user should never experience the UI freezing because an API request is slow.

---

# 38. Async Architecture

All network operations should be asynchronous.

The TUI event loop must never block on:

```text
HTTP request
DNS lookup
FortiGate timeout
Ping
Traceroute
Packet capture
```

A slow FortiGate should produce:

```text
Branch-03
Status: Refreshing...
```

rather than freezing the entire interface.

Timeouts should be explicit and bounded.

---

# 39. Error Handling

Errors should be normalized.

Possible categories:

```text
ConnectionError
AuthenticationError
AuthorizationError
TlsError
ApiError
TimeoutError
UnsupportedError
InvalidResponse
FortiGateError
```

The UI should provide actionable errors.

Bad:

```text
Request failed.
```

Better:

```text
Unable to retrieve SD-WAN state.

FortiGate:
  Branch-02

Error:
  HTTP 403 Forbidden

The API token authenticated successfully but does not have
permission to access this resource.
```

---

# 40. FortiOS Compatibility

Phase 1 should support a defined minimum FortiOS version.

The implementation should avoid assuming that all API endpoints exist on all FortiOS versions.

The backend should expose feature availability.

Example:

```text
FortiOS 7.4:
✓ Interfaces
✓ SD-WAN
✓ IPsec
? Feature X unavailable
```

The application should gracefully handle unsupported endpoints.

It should not crash when a newer or older FortiGate lacks a particular API endpoint.

---

# 41. VDOM Support

VDOMs should be explicitly considered during Phase 1.

The application should detect whether VDOMs are enabled.

Possible models:

```text
FortiGate
 ├── root
 ├── customer-a
 └── customer-b
```

The UI should provide a VDOM selector where appropriate.

Example:

```text
FortiGate: Branch-01
VDOM:      root ▼
```

The application should distinguish:

- Global/system-level information
- VDOM-specific information

This is particularly important because many FortiGate API endpoints behave differently depending on VDOM context.

---

# 42. Configuration vs Operational Data

Phase 1 should intentionally distinguish:

### Operational data

Read frequently:

- CPU
- Memory
- Interfaces
- Sessions
- SD-WAN
- VPN
- Routing
- BGP
- Health checks

### Configuration data

Read infrequently:

- Interface configuration
- SD-WAN rules
- Firewall policies
- VPN configuration
- Routing configuration

The TUI should not repeatedly download large configuration structures simply to render a dashboard.

---

# 43. Read-Only by Default

Phase 1 should be **read-only by default**.

Diagnostics that execute commands should be explicitly classified as operational actions.

No configuration modification should be required for the initial product.

Potential future functionality:

```text
set interface
disable policy
clear session
restart tunnel
```

should not be implemented until there is a compelling use case and an appropriate safety model.

The first version should make it difficult to accidentally change production state.

---

# 44. Explicit Dangerous Operations

If write functionality is eventually introduced, destructive operations must require explicit confirmation.

For example:

```text
WARNING

You are about to disable:

Policy 103
"Internet → Server"

This may interrupt production traffic.

Type the policy ID to confirm:
>
```

Never rely solely on:

```text
Are you sure? [Y/n]
```

for high-impact operations.

---

# 45. Configuration File Locations

The application should follow platform conventions.

Conceptually:

Linux:

```text
~/.config/fortitui/
```

or:

```text
~/.config/fortitui/config.yaml
```

Credentials should preferably be separate from the general configuration.

Example:

```text
~/.config/fortitui/config.yaml
~/.config/fortitui/profiles.yaml
```

with secrets stored in the OS credential store where possible.

---

# 46. Logging

The application should provide structured debug logging.

Normal mode:

```text
No debug logging.
```

Debug mode:

```bash
fortitui --debug
```

Logs should help diagnose:

- API failures
- Authentication problems
- Timeout issues
- Serialization errors
- Version compatibility issues
- TUI failures

Secrets must never appear in logs.

Specifically:

- API tokens
- Passwords
- Authorization headers
- Private keys

must be redacted.

---

# 47. Debug Mode

A debug mode should be available for users reporting problems.

Example:

```bash
fortitui --debug
```

The application may show:

```text
Backend: DirectFortiGate
Host:    10.10.10.1
FortiOS: 7.6.5
API:     reachable
Auth:    successful
```

But must never display the actual token.

---

# 48. Performance Requirements

The TUI should feel instantaneous even when the FortiGate is slow.

Target:

- Keyboard input response: effectively immediate
- Screen navigation: <100 ms under normal conditions
- Cached data rendering: <100 ms
- API calls: asynchronous
- One slow endpoint must not block unrelated screens

The application should support at least one FortiGate comfortably without excessive CPU/memory usage.

---

# 49. Packaging

Phase 1 should produce a simple distributable binary.

Preferred model:

```text
fortitui
```

rather than:

```text
python fortitui.py
```

or requiring a large runtime environment.

Release artifacts should eventually include:

```text
Linux x86_64
Linux ARM64
macOS ARM64
macOS x86_64
Windows x86_64
```

Additional platforms can be added later.

---

# 50. Dependencies

A likely implementation stack is:

## Language

**Rust**

Reasons:

- Excellent TUI ecosystem
- Strong async support
- Single-binary deployment
- Good cross-platform support
- Memory safety
- Good networking libraries
- Excellent suitability for long-running terminal applications

## TUI

**Ratatui**

Responsibilities:

- Layout
- Tables
- Lists
- Charts
- Panels
- Rendering

## Terminal

**Crossterm**

Responsibilities:

- Keyboard input
- Terminal control
- Mouse support if implemented
- Alternate screen
- Raw mode

## Async Runtime

**Tokio**

Responsibilities:

- Concurrent API requests
- Timers
- Background polling
- Diagnostics
- Event processing

## HTTP

An async Rust HTTP client such as `reqwest`.

Responsibilities:

- HTTPS
- API calls
- Timeouts
- Connection pooling

## Serialization

`serde` / `serde_json`

Responsibilities:

- FortiGate API responses
- Configuration
- Internal serialization

## Configuration

A human-readable configuration format such as YAML or TOML.

## Credential Storage

Use platform-appropriate secure storage where feasible.

## Logging

A structured Rust logging/tracing framework.

---

# 51. Recommended Internal Modules

A possible Phase 1 project structure:

```text
src/
│
├── main.rs
│
├── cli/
│   ├── args.rs
│   └── commands.rs
│
├── config/
│   ├── mod.rs
│   ├── profiles.rs
│   └── credentials.rs
│
├── backend/
│   ├── mod.rs
│   ├── traits.rs
│   ├── capabilities.rs
│   └── direct.rs
│
├── fortigate/
│   ├── client.rs
│   ├── auth.rs
│   ├── endpoints.rs
│   ├── models.rs
│   └── normalize.rs
│
├── models/
│   ├── system.rs
│   ├── interface.rs
│   ├── sdwan.rs
│   ├── vpn.rs
│   ├── routing.rs
│   ├── bgp.rs
│   ├── firewall.rs
│   └── events.rs
│
├── poller/
│   ├── scheduler.rs
│   ├── tasks.rs
│   └── cache.rs
│
├── diagnostics/
│   ├── ping.rs
│   ├── traceroute.rs
│   ├── dns.rs
│   └── route.rs
│
└── tui/
    ├── app.rs
    ├── event.rs
    ├── state.rs
    ├── layout.rs
    ├── widgets/
    └── screens/
```

This is illustrative rather than prescriptive.

---

# 52. Backend Trait

The architecture should revolve around an abstract backend.

Conceptually:

```rust
trait FortiGateBackend {
    async fn system_status(&self) -> Result<SystemStatus>;
    async fn interfaces(&self) -> Result<Vec<Interface>>;
    async fn sdwan(&self) -> Result<SdwanState>;
    async fn vpn(&self) -> Result<Vec<IpsecTunnel>>;
    async fn routes(&self, family: AddressFamily) -> Result<Vec<Route>>;
    async fn bgp(&self) -> Result<BgpState>;
    async fn sessions(&self, filter: SessionFilter) -> Result<Vec<Session>>;
    async fn capabilities(&self) -> Result<Capabilities>;
}
```

Phase 1:

```text
FortiGateBackend
       │
       └── DirectFortiGateBackend
```

Phase 2:

```text
FortiGateBackend
       ├── DirectFortiGateBackend
       └── FortiTuiServerBackend
```

Phase 3:

```text
FortiGateBackend
       ├── DirectFortiGateBackend
       ├── FortiTuiServerBackend
       └── FortiManagerBackend
```

The exact Rust API should be designed during implementation.

---

# 53. Data Normalization

FortiGate API responses should be converted into stable internal structures.

Example:

```text
FortiGate API
      │
      ▼
API-specific response
      │
      ▼
Normalizer
      │
      ▼
Application model
```

For example:

```text
FortiGate API:
member/interface/status/health-check/etc.

            ↓

SdwanMember {
    name
    interface
    state
    latency
    jitter
    packet_loss
    sla
}
```

The TUI should only know about `SdwanMember`.

---

# 54. Testing

Testing should exist at several layers.

## Unit tests

Test:

- API parsing
- Normalization
- Configuration
- Profile loading
- Capability detection
- Event detection

## Integration tests

Use captured FortiGate API responses.

Example:

```text
fixtures/
├── fortios-7.4/
├── fortios-7.6/
└── fortios-8.0/
```

This allows development without requiring a physical FortiGate for every test.

## Live tests

A test FortiGate should eventually be used to validate:

- Authentication
- API behavior
- Endpoint compatibility
- Diagnostics
- Version differences

---

# 55. API Response Fixtures

Real API responses should be captured and sanitized.

Sensitive data must be removed.

Fixtures should represent:

- Healthy system
- Degraded system
- Interface down
- SD-WAN failure
- IPsec down
- BGP down
- IPv6
- Multiple VDOMs
- HA
- Unsupported endpoint

This will be particularly important for maintaining compatibility across FortiOS releases.

---

# 56. Security Requirements

FortiTUI will handle highly privileged network management credentials.

Security is therefore a first-class concern.

Requirements:

- Never log secrets
- Never display secrets
- Never transmit credentials unnecessarily
- Prefer API tokens
- Support TLS verification
- Warn when TLS verification is disabled
- Minimize required API privileges
- Avoid arbitrary remote command execution
- Avoid unnecessary write capabilities
- Secure local configuration permissions

The documentation should explicitly recommend using a dedicated FortiGate API administrator with the minimum permissions required.

---

# 57. API Permission Model

The exact FortiGate permissions required should be documented.

The project should aim to determine the minimum access required for:

- System information
- Interfaces
- SD-WAN
- VPN
- Routing
- BGP
- Firewall
- Diagnostics

If some functionality requires elevated privileges, the application should identify it.

Example:

```text
Available:
✓ System
✓ Interfaces
✓ SD-WAN

Unavailable:
✕ Packet Capture

Reason:
API administrator lacks required permission.
```

---

# 58. Operational Philosophy

FortiTUI should prioritize **situational awareness**.

The user should not have to inspect ten screens to discover that something is wrong.

Bad:

```text
System
Interfaces
SD-WAN
VPN
Routing
...
```

with no indication of abnormal state.

Better:

```text
FORTIGATE

● System healthy
▲ WAN2 degraded
● VPN healthy
▲ BGP neighbor unstable
● Routing healthy
```

The dashboard should surface abnormalities.

---

# 59. State Classification

A common state model should be used throughout the application.

For example:

```text
UNKNOWN
OK
INFO
DEGRADED
WARNING
CRITICAL
DOWN
```

Avoid relying purely on colors.

A terminal may have:

- No color
- Limited color
- Accessibility requirements
- Different terminal themes

State should therefore be represented by:

- Color
- Symbol
- Text

where appropriate.

Example:

```text
● UP
▲ DEGRADED
✕ DOWN
? UNKNOWN
```

---

# 60. Real-Time vs Historical Data

FortiTUI should clearly distinguish current state from sampled state.

Example:

```text
Current:
WAN1 latency: 18ms

Recent:
10-minute average: 19ms
10-minute maximum: 31ms
```

The short rolling history is operational context, not a replacement for Argus or another historical data platform.

---

# 61. No Mandatory Database in Phase 1

Phase 1 should not require a database.

State can be held in memory.

Optional local persistence may be considered later.

This keeps installation extremely simple:

```text
Download binary
      ↓
Create profile
      ↓
Connect
```

rather than:

```text
Install application
Install database
Configure database
Configure service
Configure credentials
Configure API
Configure client
```

---

# 62. User Experience Target

A new user should be able to go from zero to useful data in approximately:

```text
< 5 minutes
```

Target workflow:

```bash
$ fortitui profile add

FortiGate hostname:
10.10.10.1

API token:
********

Test connection...
SUCCESS

Save profile as:
branch-01

$ fortitui --profile branch-01
```

The user should immediately see the system dashboard.

---

# 63. Help System

Every screen should have contextual help.

For example:

```text
Press ? for help
```

Then:

```text
SD-WAN HELP

↑/↓      Select member
Enter    View details
r        Refresh
h        Health checks
l        View recent latency
Esc      Back
```

A global help screen should document all common shortcuts.

---

# 64. Search

Search should be implemented early because FortiGate environments can contain large numbers of objects.

Search should eventually work against:

- Interfaces
- VPN tunnels
- Policies
- Routes
- BGP neighbors
- SD-WAN members
- Health checks

Example:

```text
/branch-03
```

could locate:

```text
IPsec: Branch-03
SD-WAN: Branch-03
Interface: vpn-branch-03
Policy: Branch-03 → Server
```

---

# 65. Command-Line Non-TUI Mode

Although the primary interface is the TUI, basic non-interactive commands are useful.

Examples:

```bash
fortitui profile test fortigate-1
```

```bash
fortitui status --profile fortigate-1
```

```bash
fortitui vpn --profile fortigate-1
```

```bash
fortitui sdwan --profile fortigate-1
```

This enables:

- Scripts
- Automation
- Troubleshooting
- CI checks
- Shell workflows

Output should support human-readable and potentially JSON modes.

Example:

```bash
fortitui status --profile fortigate-1 --json
```

---

# 66. JSON Output

JSON output should use the normalized application model rather than raw FortiGate API responses.

This provides a stable automation interface.

Example:

```json
{
  "hostname": "branch-01",
  "status": "healthy",
  "cpu_percent": 18,
  "memory_percent": 61,
  "sessions": 18421
}
```

This becomes particularly valuable if users eventually want to integrate FortiTUI data with:

- Scripts
- Monitoring
- Alerting
- Automation
- Other observability systems

---

# 67. Phase 1 Milestones

## Milestone 1 — Project Foundation

Deliver:

- Rust project
- CLI argument handling
- Configuration system
- Profile system
- Logging
- Basic TUI
- Backend abstraction

Success criteria:

```text
fortitui --version
fortitui profile list
fortitui --profile <name>
```

all work.

---

## Milestone 2 — FortiGate Connectivity

Deliver:

- HTTPS
- TLS handling
- API authentication
- Connection testing
- Error handling
- System information

Success:

```text
FortiTUI successfully connects to a real FortiGate
and displays model, hostname, serial, version,
uptime, CPU, memory, and sessions.
```

---

## Milestone 3 — Interfaces

Deliver:

- Interface list
- Interface details
- Counters
- Rates
- Errors
- State
- IPv4
- IPv6
- Basic real-time graphs

---

## Milestone 4 — SD-WAN

Deliver:

- Members
- Health checks
- SLA state
- Latency
- Jitter
- Loss
- Traffic
- Selection state
- Short-term trends

This milestone should be considered one of the most important Phase 1 deliverables.

---

## Milestone 5 — VPN

Deliver:

- IPsec overview
- Tunnel state
- Phase 1
- Phase 2
- Traffic
- Crypto details
- Rekey information

---

## Milestone 6 — Routing

Deliver:

- IPv4 routes
- IPv6 routes
- Route lookup
- BGP
- OSPF where practical

---

## Milestone 7 — Diagnostics

Deliver:

- Ping
- IPv6 ping
- Traceroute
- DNS lookup
- Route lookup
- SD-WAN lookup

---

## Milestone 8 — Operational Polish

Deliver:

- Events
- State transitions
- Search
- Command palette
- Help
- Better error messages
- JSON CLI output
- Performance improvements

---

# 68. Phase 1 Definition of Done

Phase 1 should be considered complete when a network engineer can:

1. Install FortiTUI without deploying any server-side infrastructure.
2. Create a FortiGate profile.
3. Authenticate securely using an API token.
4. View overall FortiGate health.
5. Inspect interfaces.
6. Inspect IPv4 and IPv6 addresses.
7. View real-time interface traffic.
8. Inspect SD-WAN members.
9. Inspect SD-WAN health checks.
10. Identify degraded WAN members.
11. Inspect IPsec tunnels.
12. Inspect routing.
13. Inspect BGP.
14. Run basic network diagnostics.
15. Search operational objects.
16. Understand recent state changes.
17. Operate entirely from the keyboard.
18. Recover gracefully from FortiGate/API failures.
19. Use the application without a database or server.
20. Use the same application against multiple individual FortiGate profiles.

---

# 69. Phase 2 — FortiTUI Server / Proxy

Phase 2 should introduce a server component only when real Phase 1 users demonstrate a need for it.

Conceptually:

```text
                 ┌──────────────┐
                 │   FortiTUI   │
                 └──────┬───────┘
                        │
                     HTTPS
                        │
                        ▼
              ┌───────────────────┐
              │ FortiTUI Server   │
              │                   │
              │ Inventory         │
              │ Credentials       │
              │ Polling           │
              │ Cache             │
              │ Events            │
              └─────────┬─────────┘
                        │
          ┌─────────────┼─────────────┐
          ▼             ▼             ▼
        FG-01         FG-02         FG-03
```

The server should likely be distributed as:

```text
Docker image
```

and eventually:

```text
docker compose
```

for simple deployment.

Potential server responsibilities:

- Device inventory
- Credential management
- Polling
- State cache
- Concurrent connections
- Event generation
- Fleet aggregation
- Authentication
- Authorization
- Multi-user support

However, **these should not be treated as fixed Phase 2 requirements.**

User feedback from Phase 1 should determine:

- What data users actually need centrally
- How often they need it
- How many FortiGates they manage
- Whether polling should be centralized
- Whether events need persistence
- Whether users need multiple roles
- Whether credentials need centralized storage
- Whether users want browser access
- Whether the TUI should remain the only client

---

# 70. Phase 2 Multi-Device Profiles

The same TUI binary should be able to consume a server profile.

Conceptually:

```yaml
profiles:

  branch-01:
    type: direct
    host: 10.10.10.1
    credential: branch-01

  noc:
    type: server
    endpoint: https://fortitui.example.com
```

Then:

```bash
fortitui --profile branch-01
```

uses:

```text
FortiTUI → FortiGate
```

while:

```bash
fortitui --profile noc
```

uses:

```text
FortiTUI → FortiTUI Server → FortiGates
```

The TUI should remain fundamentally the same.

---

# 71. Phase 2 Fleet UI

Potential future layout:

```text
FORTIGATE FLEET

DEVICE          STATUS       CPU     WAN       VPN       ALERTS
──────────────────────────────────────────────────────────────────
HQ              ● HEALTHY    18%     2/2       14/14       0
Branch-01       ● HEALTHY    12%     2/2        4/4        0
Branch-02       ▲ DEGRADED   21%     1/2        4/4        2
Branch-03       ● HEALTHY     9%     2/2        2/2        0
Branch-04       ✕ DOWN        --      --         --         1
```

Selecting a device transitions into the Phase 1 experience.

This is an important design principle:

> **Fleet mode should add a layer above the single-device experience rather than creating a completely separate interface.**

---

# 72. Phase 3 — FortiManager

Phase 3 should add FortiManager as a backend.

The architecture should permit:

```text
TUI
 │
 ├── Direct FortiGate
 │
 ├── FortiTUI Server
 │
 └── FortiManager
```

Potential FortiManager capabilities:

- Discover managed FortiGates
- Display ADOMs
- Display device status
- Display configuration status
- Display revisions
- Display installation state
- Display management connectivity
- Provide centralized device metadata

The exact functionality should be determined after Phase 1 and Phase 2 users provide feedback.

---

# 73. FortiManager Is Not Merely Another Proxy

FortiManager should not automatically be treated as equivalent to the FortiTUI proxy.

FortiManager has its own concepts:

- ADOMs
- Managed devices
- Device databases
- Revisions
- Workspace
- Installation
- Policy packages
- Configuration status

FortiTUI should preserve those semantics where they are useful.

A future FortiManager screen may therefore look more like:

```text
FORTIMANAGER

ADOM: root

DEVICES

Branch-01     Managed      Synced
Branch-02     Managed      Modified
Branch-03     Offline     Unknown
```

rather than simply pretending FortiManager is another HTTP proxy.

---

# 74. Future Architecture

The eventual architecture should look approximately like:

```text
                         ┌──────────────────────┐
                         │       FortiTUI       │
                         │                      │
                         │      Ratatui         │
                         │      Core UI          │
                         └──────────┬───────────┘
                                    │
                              Backend API
                                    │
              ┌─────────────────────┼─────────────────────┐
              │                     │                     │
              ▼                     ▼                     ▼
       Direct Backend        Server Backend       FortiManager
              │                     │                     │
              ▼                     ▼                     ▼
         FortiGate             FortiTUI Server       FortiManager
                                    │
                         ┌──────────┼──────────┐
                         ▼          ▼          ▼
                        FG1        FG2        FG3
```

The TUI should remain largely ignorant of where the data originated.

---

# 75. Design Principles

## Principle 1 — Operational first

Every screen should help answer an operational question.

## Principle 2 — Don't recreate the GUI

The TUI should expose information that is valuable in a terminal and difficult to consume quickly from the standard GUI.

## Principle 3 — Read-only first

Avoid configuration modification until the product has established a strong operational foundation.

## Principle 4 — Fast

The TUI must remain responsive even when FortiGate APIs are slow.

## Principle 5 — Keyboard-first

An experienced operator should be able to navigate almost entirely without a mouse.

## Principle 6 — Normalize data

Never make the TUI dependent on raw FortiGate API response formats.

## Principle 7 — Backend independence

The UI should not care whether the data came from:

- Direct FortiGate
- FortiTUI Server
- FortiManager

## Principle 8 — Don't overbuild Phase 2

Phase 2 should be driven by actual users.

## Principle 9 — Don't overbuild Phase 3

FortiManager integration should be driven by actual FortiManager workflows.

## Principle 10 — Complement, don't replace

FortiTUI should work alongside:

- FortiGate GUI
- FortiGate CLI
- FortiManager
- FortiAnalyzer
- Grafana
- Historical observability systems

---

# 76. Initial Product Positioning

The simplest description of the product should be:

> **FortiTUI is a terminal-based operational console for FortiGate.**

The initial differentiator is:

> **Fast, keyboard-driven visibility into the things a network engineer actually needs during troubleshooting.**

Phase 1 is deliberately a **single-device tool**.

Phase 2 evolves it into a **FortiGate fleet/NOC tool**.

Phase 3 integrates it into **Fortinet-managed environments using FortiManager**.

The core product remains the same throughout.

---

# 77. Recommended Development Order

The implementation should proceed approximately in this order:

```text
1. Project skeleton
       ↓
2. CLI / configuration
       ↓
3. Profile system
       ↓
4. FortiGate API client
       ↓
5. Backend abstraction
       ↓
6. System dashboard
       ↓
7. Interface monitoring
       ↓
8. SD-WAN
       ↓
9. IPsec
       ↓
10. Routing / BGP
       ↓
11. Diagnostics
       ↓
12. Events / short-term state
       ↓
13. Search / command palette
       ↓
14. JSON / automation interface
       ↓
15. Compatibility / packaging
       ↓
16. Real-user feedback
       ↓
17. Phase 2 design
       ↓
18. Phase 3 design
```

The critical architectural milestone is **#5**.

Once the backend abstraction exists, the project can evolve without forcing the TUI itself to understand how devices are accessed.

---

# 78. Final Architectural Recommendation

The recommended initial architecture is:

```text
                 ┌───────────────────────────┐
                 │         FortiTUI          │
                 │                           │
                 │       Ratatui TUI         │
                 │                           │
                 │  Dashboard                │
                 │  Interfaces               │
                 │  SD-WAN                   │
                 │  VPN                      │
                 │  Routing                  │
                 │  BGP                      │
                 │  Diagnostics              │
                 │  Events                   │
                 └─────────────┬─────────────┘
                               │
                        Backend Interface
                               │
                               ▼
                  ┌─────────────────────────┐
                  │ Direct FortiGate        │
                  │ Backend                 │
                  └────────────┬────────────┘
                               │
                           HTTPS/API
                               │
                               ▼
                  ┌─────────────────────────┐
                  │       FortiGate         │
                  └─────────────────────────┘
```

Phase 1 should **not require**:

- Docker
- A server
- A database
- FortiManager
- A cloud service
- A web UI

The ideal Phase 1 experience is simply:

```bash
fortitui --profile branch-01
```

followed by an immediate operational view of the FortiGate.

Later:

```bash
fortitui --profile noc
```

can transparently switch to:

```text
FortiTUI
   ↓
FortiTUI Server
   ↓
FortiGate fleet
```

And eventually:

```bash
fortitui --profile fortimanager
```

can provide:

```text
FortiTUI
   ↓
FortiManager
   ↓
Managed FortiGate estate
```

The **TUI should remain the product**. The connection backend should be an implementation detail.

That separation is the central architectural decision that should be preserved throughout the project.