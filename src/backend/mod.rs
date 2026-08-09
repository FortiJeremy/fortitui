//! Backend abstraction. The TUI depends only on these traits, never on raw
//! FortiGate API responses. Phase 1 implements `DirectBackend`.

pub mod capabilities;
pub mod direct;
pub mod traits;

pub use capabilities::Capabilities;
pub use direct::DirectBackend;
pub use traits::{AddressFamily, FortiGateBackend};
