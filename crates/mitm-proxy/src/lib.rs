//! mitm-proxy: concrete flow types for the mitmproxy-rs data model.
//!
//! This crate provides:
//! - **Flow enum**: `Flow::Http`, `Flow::Tcp`, `Flow::Udp`, `Flow::Dns`
//! - **Flow variants**: `HTTPFlow`, `TCPFlow`, `UDPFlow`, `DNSFlow`
//! - **Flow lifecycle**: intercept, resume, kill, copy
//! - **TCP/UDP messages**: `TCPMessage`, `UDPMessage`
//! - **DNS types**: `DNSMessage`, `Question`, `ResourceRecord`, `DNSFlow`
//! - **WebSocket types**: `WebSocketMessage`, `WebSocketData`

pub mod dns;
pub mod flows;
pub mod stream;
pub mod websocket;

// Re-exports.
pub use dns::{DNSFlow, DNSMessage, DnsClass, DnsError, DnsType, Question, Rcode, ResourceRecord};
pub use flows::{Flow, HTTPFlow, TCPFlow, UDPFlow};
pub use stream::{TCPMessage, UDPMessage};
pub use websocket::{WebSocketData, WebSocketMessage, WebSocketOpcode};

// Re-export core and net for convenience.
pub use mitm_core as core;
pub use mitm_net as net;
