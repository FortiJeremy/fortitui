//! Normalized application data models.
//!
//! The TUI only ever sees these types — never raw FortiGate API responses.

pub mod bgp;
pub mod events;
pub mod firewall;
pub mod interface;
pub mod routing;
pub mod sdwan;
pub mod system;
pub mod vpn;

pub use bgp::{BgpNeighbor, BgpState};
pub use events::Event;
pub use firewall::FirewallPolicy;
pub use interface::{InterfaceStatus, InterfaceType, LinkState};
pub use routing::Route;
pub use sdwan::{SdwanHealthCheck, SdwanMember, SdwanState};
pub use system::SystemStatus;
pub use vpn::IpsecTunnel;
