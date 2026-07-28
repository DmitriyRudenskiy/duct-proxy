//! mitm-options: Configuration system for mitmproxy-rs.
//!
//! This crate provides:
//! - **Options**: CLI-configurable struct with clap derive and serde serialization
//! - **ProxyMode**: Enum for proxy operating modes (Explicit, Transparent, Upstream, Local)
//! - **OptManager**: Thread-safe runtime options management
//! - **Config file**: YAML config file loading/saving with CLI override merge

pub mod options;
pub mod proxy_mode;
pub mod manager;
pub mod config;
pub mod error;

// Re-exports.
pub use options::Options;
pub use proxy_mode::ProxyMode;
pub use manager::OptManager;
pub use config::{load_config, save_config, dump_config};
pub use error::{OptionsError, ValidationError};
