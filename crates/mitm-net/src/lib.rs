//! mitm-net: network protocol types for the mitmproxy-rs data model.
//!
//! This crate provides HTTP message types used by the proxy layer.

pub mod http;

// Re-exports.
pub use http::{Message, MessageData, Request, Response, StreamMode};
