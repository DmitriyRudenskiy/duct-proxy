//! Proxy error types.

use thiserror::Error;

/// Errors that can occur in the proxy.
#[derive(Error, Debug)]
pub enum ProxyError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("No SNI in ClientHello")]
    NoSni,

    #[error("Invalid SNI: {0}")]
    InvalidSni(String),

    #[error("Certificate generation failed: {0}")]
    CertGeneration(String),

    #[error("TLS configuration error: {0}")]
    TlsConfig(String),

    #[error("TLS handshake failed: {0}")]
    TlsHandshake(String),

    #[error("Upstream TLS error: {0}")]
    UpstreamTls(String),

    #[error("Connection error: {0}")]
    Connection(String),
}
