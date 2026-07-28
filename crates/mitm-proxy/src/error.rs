//! Proxy error types.

use thiserror::Error;

/// Errors that can occur in the proxy.
#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("no SNI in ClientHello")]
    NoSni,

    #[error("invalid SNI: {0}")]
    InvalidSni(String),

    #[error("certificate generation failed: {0}")]
    CertGeneration(String),

    #[error("TLS config error: {0}")]
    TlsConfig(String),

    #[error("TLS handshake failed: {0}")]
    TlsHandshake(String),

    #[error("upstream TLS failed: {0}")]
    UpstreamTls(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Upstream connect failed: {0}")]
    UpstreamConnect(String),

    #[error("Upstream write failed: {0}")]
    UpstreamWrite(String),

    #[error("Upstream read failed: {0}")]
    UpstreamRead(String),
}
