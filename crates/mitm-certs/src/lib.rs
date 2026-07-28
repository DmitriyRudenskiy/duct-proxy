//! mitm-certs: Certificate management for mitmproxy-rs.
//!
//! This crate provides:
//! - CA certificate generation (ECDSA P-256)
//! - Certificate store with LRU eviction
//! - SNI extraction from TLS ClientHello
//! - On-the-fly leaf certificate generation

pub mod ca;
pub mod leaf;
pub mod sni;
pub mod store;

pub use ca::{CaError, CaRoot};
pub use leaf::{LeafCert, LeafError};
pub use store::{CertEntry, CertStore, StoreError};
