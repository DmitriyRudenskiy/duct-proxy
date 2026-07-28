//! mitm-core: foundational types for the mitmproxy-rs data model.
//!
//! This crate provides:
//! - **Connection types**: `Client`, `Server`, `ConnectionState`
//! - **Headers**: case-insensitive, order-preserving HTTP header collection
//! - **Flow base**: `FlowError`, `FlowBase`, lifecycle helpers
//!
//! Concrete flow variants (HTTPFlow, TCPFlow, etc.) and the `Flow` enum
//! live in the `mitm-proxy` crate.

pub mod connection;
pub mod flow;
pub mod headers;

// Re-exports for convenience.
pub use connection::{
    Address, Cert, Client, Connection, ConnectionError, ConnectionState, Server, ServerSpec,
};
pub use flow::{current_timestamp, default_client, default_server, FlowBase, FlowError};
pub use headers::{headers_to_bytes, HeaderError, HeaderField, Headers};
