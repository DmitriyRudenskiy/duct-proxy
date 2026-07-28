//! mitm-proxy: concrete flow types and proxy engine for mitmproxy-rs.
//!
//! This crate provides:
//! - **Flow enum**: `Flow::Http`, `Flow::Tcp`, `Flow::Udp`, `Flow::Dns`
//! - **Flow variants**: `HTTPFlow`, `TCPFlow`, `UDPFlow`, `DNSFlow`
//! - **Flow lifecycle**: intercept, resume, kill, copy
//! - **TCP/UDP messages**: `TCPMessage`, `UDPMessage`
//! - **DNS types**: `DNSMessage`, `Question`, `ResourceRecord`, `DNSFlow`
//! - **WebSocket types**: `WebSocketMessage`, `WebSocketData`
//! - **Proxy server**: TCP listener, accept loop, connection handling
//! - **Protocol detection**: HTTP vs TLS vs raw TCP classification
//! - **Hook system**: Extension points for addons

pub mod dns;
pub mod error;
pub mod flows;
pub mod handler;
pub mod hooks;
pub mod server;
pub mod stream;
pub mod tls;
pub mod websocket;

// Re-exports.
pub use dns::{DNSFlow, DNSMessage, DnsClass, DnsError, DnsType, Question, Rcode, ResourceRecord};
pub use flows::{Flow, HTTPFlow, TCPFlow, UDPFlow};
pub use handler::{detect_protocol, Protocol, TunnelHandler, HttpForwarder};
pub use hooks::{HookDispatcher, HttpRequestHook, HttpResponseHook, ErrorHook};
pub use server::ProxyServer;
pub use stream::{TCPMessage, UDPMessage};
pub use tls::{intercept_tls, forward_bidirectional};
pub use websocket::{WebSocketData, WebSocketMessage, WebSocketOpcode};

// Re-export core and net for convenience.
pub use mitm_core as core;
pub use mitm_net as net;
