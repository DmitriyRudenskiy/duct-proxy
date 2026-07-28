//! mitm-addons: Addon system for mitmproxy-rs.
//!
//! This crate provides:
//! - **Addon trait**: Lifecycle hooks for HTTP, TCP, UDP, DNS flows
//! - **AddonManager**: Registration, sequential dispatch, error isolation
//! - **Built-in addons**: ModifyHeaders, ModifyBody, Block, Filter

pub mod addon;
pub mod manager;
pub mod modify_headers;
pub mod modify_body;
pub mod block;
pub mod filter;

// Re-exports.
pub use addon::{Addon, AddonError};
pub use manager::AddonManager;
pub use modify_headers::ModifyHeaders;
pub use modify_body::ModifyBody;
pub use block::Block;
pub use filter::Filter;
