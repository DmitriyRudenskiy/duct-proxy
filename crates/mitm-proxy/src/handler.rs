//! Protocol detection and connection handling.

use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{info, debug, error};

/// Detected protocol type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    /// TLS ClientHello (first byte 0x16).
    Tls,
    /// HTTP CONNECT request.
    HttpConnect,
    /// HTTP request (GET, POST, etc.).
    Http,
    /// Raw TCP (unrecognized data).
    Raw,
}

/// Detect protocol from first bytes of a TCP stream.
///
/// # Arguments
/// * `stream` - Mutable reference to TCP stream
///
/// # Returns
/// Result with Protocol or error
pub async fn detect_protocol(stream: &mut TcpStream) -> Result<Protocol, Box<dyn std::error::Error + Send + Sync>> {
    let mut peek_buf = [0u8; 8];
    stream.peek(&mut peek_buf).await?;

    // Check for TLS (first byte 0x16 = Handshake record type)
    if peek_buf[0] == 0x16 {
        return Ok(Protocol::Tls);
    }

    // Check for HTTP CONNECT
    if peek_buf.len() >= 8 {
        let prefix = std::str::from_utf8(&peek_buf[..8]).unwrap_or("");
        if prefix.eq_ignore_ascii_case("CONNECT ") {
            return Ok(Protocol::HttpConnect);
        }
    }

    // Check for HTTP methods
    let http_methods = ["GET ", "POST ", "PUT ", "DELETE ", "PATCH ", "HEAD ", "OPTIONS ", "TRACE "];
    if peek_buf.len() >= 4 {
        let prefix = std::str::from_utf8(&peek_buf[..4]).unwrap_or("");
        for method in &http_methods {
            if prefix.eq_ignore_ascii_case(method) {
                return Ok(Protocol::Http);
            }
        }
    }

    // Default to Raw TCP
    Ok(Protocol::Raw)
}

/// Handler for CONNECT tunnels.
pub struct TunnelHandler;

impl TunnelHandler {
    /// Handle a CONNECT request: parse URI, establish tunnel, forward bytes.
    pub async fn handle_connect(
        &self,
        mut client_stream: TcpStream,
        _addr: SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Read CONNECT request line
        let mut buf = vec![0u8; 1024];
        let n = client_stream.read(&mut buf).await?;
        let request_line = std::str::from_utf8(&buf[..n])?;

        // Parse "CONNECT host:port HTTP/1.1\r\n"
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 2 {
            return Err("Invalid CONNECT request".into());
        }

        let host_port = parts[1];
        let (host, port) = if let Some(colon_pos) = host_port.rfind(':') {
            (&host_port[..colon_pos], host_port[colon_pos + 1..].parse::<u16>()?)
        } else {
            (host_port, 443u16) // Default to 443
        };

        info!("CONNECT tunnel to {}:{}", host, port);

        // Send 200 OK
        client_stream.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n").await?;

        // Connect to upstream
        let mut upstream = TcpStream::connect(format!("{}:{}", host, port)).await?;
        info!("Connected to upstream {}:{}", host, port);

        // Bidirectional forwarding
        let (mut client_read, mut client_write) = client_stream.split();
        let (mut upstream_read, mut upstream_write) = upstream.split();

        let client_to_upstream = tokio::io::copy(&mut client_read, &mut upstream_write);
        let upstream_to_client = tokio::io::copy(&mut upstream_read, &mut client_write);

        tokio::select! {
            result = client_to_upstream => {
                result?;
            }
            result = upstream_to_client => {
                result?;
            }
        }

        info!("Tunnel to {}: {} closed", host, port);
        Ok(())
    }
}

/// HTTP request/response forwarder.
pub struct HttpForwarder;

impl HttpForwarder {
    /// Forward an HTTP request to upstream and return the response.
    pub async fn forward(
        &self,
        mut client_stream: TcpStream,
        addr: SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Read HTTP request
        let mut buf = vec![0u8; 65536];
        let n = client_stream.read(&mut buf).await?;
        let request = std::str::from_utf8(&buf[..n])?;

        debug!("Received HTTP request:\n{}", request);

        // For now, just echo back a 200 OK (full implementation would parse and forward)
        let response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 12\r\n\r\nHello, world!";
        client_stream.write_all(response.as_bytes()).await?;

        info!("Sent HTTP response to {}", addr);
        Ok(())
    }
}

/// Handle a connection based on detected protocol.
pub async fn handle_connection(
    mut stream: TcpStream,
    addr: SocketAddr,
    protocol: Protocol,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match protocol {
        Protocol::Tls => {
            info!("TLS connection from {}", addr);
            // Perform TLS interception (MITM)
            // For now, we just accept the connection and wait for data
            let mut buf = vec![0u8; 4096];
            match stream.read(&mut buf).await {
                Ok(n) => info!("Read {} bytes from TLS connection", n),
                Err(e) => error!("TLS read error: {}", e),
            }
            Ok(())
        }
        Protocol::HttpConnect => {
            info!("CONNECT request from {}", addr);
            TunnelHandler.handle_connect(stream, addr).await
        }
        Protocol::Http => {
            info!("HTTP request from {}", addr);
            HttpForwarder.forward(stream, addr).await
        }
        Protocol::Raw => {
            info!("Raw TCP connection from {}", addr);
            // For raw TCP, just forward bytes (simplified)
            let mut buf = vec![0u8; 4096];
            match stream.read(&mut buf).await {
                Ok(n) => info!("Read {} bytes from {}", n, addr),
                Err(e) => error!("Read error from {}: {}", addr, e),
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;

    async fn setup_test_stream(data: &[u8]) -> TcpStream {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let data_copy = data.to_vec();

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            stream.write_all(&data_copy).await.unwrap();
            stream.flush().await.unwrap();
        });

        let client = TcpStream::connect(addr).await.unwrap();
        server.await.unwrap();
        client
    }

    #[tokio::test]
    async fn test_detect_protocol_tls() {
        let mut stream = setup_test_stream(&[0x16, 0x03, 0x01]).await;
        let protocol = detect_protocol(&mut stream).await.unwrap();
        assert_eq!(protocol, Protocol::Tls);
    }

    #[tokio::test]
    async fn test_detect_protocol_http_connect() {
        let mut stream = setup_test_stream(b"CONNECT example.com:443 HTTP/1.1\r\n").await;
        let protocol = detect_protocol(&mut stream).await.unwrap();
        assert_eq!(protocol, Protocol::HttpConnect);
    }

    #[tokio::test]
    async fn test_detect_protocol_http_get() {
        let mut stream = setup_test_stream(b"GET / HTTP/1.1\r\n").await;
        let protocol = detect_protocol(&mut stream).await.unwrap();
        assert_eq!(protocol, Protocol::Http);
    }

    #[tokio::test]
    async fn test_detect_protocol_raw() {
        let mut stream = setup_test_stream(b"random data here").await;
        let protocol = detect_protocol(&mut stream).await.unwrap();
        assert_eq!(protocol, Protocol::Raw);
    }
}
