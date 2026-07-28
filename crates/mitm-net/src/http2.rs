//! HTTP/2 support module.

use std::io::Error as IoError;
use thiserror::Error;
use tokio::net::TcpStream;

/// HTTP/2 connection preface bytes for prior knowledge.
pub const HTTP2_CLIENT_PRIOR_KNOWLEDGE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

/// Errors that can occur during HTTP/2 operations.
#[derive(Error, Debug)]
pub enum Http2Error {
    #[error("I/O error: {0}")]
    Io(#[from] IoError),

    #[error("Invalid connection preface")]
    InvalidPreface,

    #[error("H2 error: {0}")]
    H2Error(String),
}

/// Check if data starts with HTTP/2 client prior knowledge preface.
pub fn has_http2_prior_knowledge(data: &[u8]) -> bool {
    data.starts_with(HTTP2_CLIENT_PRIOR_KNOWLEDGE)
}

/// Detect if a connection is HTTP/2 (simplified check).
pub async fn detect_http2(stream: &mut TcpStream) -> Result<bool, Http2Error> {
    let mut peek_buf = [0u8; HTTP2_CLIENT_PRIOR_KNOWLEDGE.len()];
    let n = stream.peek(&mut peek_buf).await?;
    Ok(has_http2_prior_knowledge(&peek_buf[..n]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http2_prior_knowledge_detection() {
        assert!(has_http2_prior_knowledge(HTTP2_CLIENT_PRIOR_KNOWLEDGE));
        assert!(!has_http2_prior_knowledge(b"GET / HTTP/1.1\r\n"));
    }

    #[test]
    fn test_http2_prior_knowledge_partial() {
        assert!(!has_http2_prior_knowledge(b"PRI * HTTP/2.0"));
        assert!(!has_http2_prior_knowledge(b""));
    }
}
