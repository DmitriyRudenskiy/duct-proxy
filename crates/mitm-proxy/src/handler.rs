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

/// Detect protocol from first bytes (sync version for use with peeked data).
pub fn detect_protocol_from_bytes(bytes: &[u8]) -> Protocol {
    if bytes.is_empty() {
        return Protocol::Raw;
    }

    // Check for TLS (first byte 0x16 = Handshake record type)
    if bytes[0] == 0x16 {
        return Protocol::Tls;
    }

    // Check for HTTP CONNECT
    if bytes.len() >= 8 {
        let prefix = std::str::from_utf8(&bytes[..8]).unwrap_or("");
        if prefix.eq_ignore_ascii_case("CONNECT ") {
            return Protocol::HttpConnect;
        }
    }

    // Check for HTTP methods
    let http_methods = ["GET ", "POST ", "PUT ", "DELETE ", "PATCH ", "HEAD ", "OPTIONS ", "TRACE "];
    if bytes.len() >= 4 {
        let prefix = std::str::from_utf8(&bytes[..4]).unwrap_or("");
        for method in &http_methods {
            if prefix.eq_ignore_ascii_case(method) {
                return Protocol::Http;
            }
        }
    }

    // Default to Raw TCP
    Protocol::Raw
}

/// Detect protocol from first bytes of a TCP stream (async version).
///
/// # Arguments
/// * `stream` - Mutable reference to TCP stream
///
/// # Returns
/// Result with Protocol or error
pub async fn detect_protocol(stream: &mut TcpStream) -> Result<Protocol, Box<dyn std::error::Error + Send + Sync>> {
    let mut peek_buf = [0u8; 8];
    stream.peek(&mut peek_buf).await?;
    Ok(detect_protocol_from_bytes(&peek_buf))
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

        // Per-flow logging for CONNECT tunnel
        tracing::info!(
            host = host,
            port = port,
            "CONNECT {}:{} → TLS intercepted",
            host,
            port
        );

        info!("Tunnel to {}: {} closed", host, port);
        Ok(())
    }
}

/// HTTP request/response forwarder.
pub struct HttpForwarder;

impl HttpForwarder {
    /// Forward an HTTP request to upstream and write response back to client.
    pub async fn forward(
        &self,
        client_stream: &mut TcpStream,
        request_bytes: &[u8],
    ) -> Result<(), crate::error::ProxyError> {
        // 1. Parse request line to get host
        let request_str = String::from_utf8_lossy(request_bytes);
        let first_line = request_str.lines().next().unwrap_or("");
        // "GET http://example.com/path HTTP/1.1"
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() < 2 {
            return Err(crate::error::ProxyError::InvalidRequest("malformed request line".into()));
        }
        let url = parts[1];
        
        // 2. Extract host:port from URL
        let host_port = url
            .trim_start_matches("http://")
            .split('/')
            .next()
            .unwrap_or("example.com:80");
        let addr = if host_port.contains(':') {
            host_port.to_string()
        } else {
            format!("{}:80", host_port)
        };
        
        tracing::info!("Forwarding to upstream: {}", addr);
        
        // 3. Connect to upstream with Happy Eyeballs (try all DNS addresses)
        let mut upstream = None;
        for addr in tokio::net::lookup_host(&addr).await
            .map_err(|e| crate::error::ProxyError::UpstreamConnect(format!("DNS: {}", e)))? {
            tracing::debug!("Trying {}", addr);
            match tokio::time::timeout(
                tokio::time::Duration::from_secs(2),
                TcpStream::connect(addr),
            ).await {
                Ok(Ok(s)) => { upstream = Some(s); break; }
                _ => continue,
            }
        }
        let mut upstream = upstream
            .ok_or_else(|| crate::error::ProxyError::UpstreamConnect("all addresses failed".into()))?;
        
        // 4. Rewrite request: absolute URL → relative path
        let path = url
            .trim_start_matches("http://")
            .find('/')
            .and_then(|i| {
                url.find("://").map(|pos| &url[pos + 3 + i..])
            })
            .unwrap_or("/");
        let rewritten = request_str.replacen(url, path, 1);
        
        // 5. Send request to upstream
        upstream.write_all(rewritten.as_bytes()).await
            .map_err(|e| crate::error::ProxyError::UpstreamWrite(e.to_string()))?;
        upstream.flush().await
            .map_err(|e| crate::error::ProxyError::UpstreamWrite(e.to_string()))?;
        
        tracing::debug!("Request sent to upstream ({} bytes)", rewritten.len());
        
        // 6. Read response — read until connection close or timeout
        let mut total = Vec::new();
        let mut buf = [0u8; 65536];
        
        loop {
            let read_result = tokio::time::timeout(
                tokio::time::Duration::from_secs(10),
                upstream.read(&mut buf),
            ).await;
            
            match read_result {
                Ok(Ok(0)) => break,      // connection closed
                Ok(Ok(n)) => {
                    total.extend_from_slice(&buf[..n]);
                    // Если получили полные headers + chunked body с terminal chunk
                    if total.windows(5).any(|w| w == b"0\r\n\r\n") {
                        break;  // chunked encoding complete
                    }
                    // Если получили Content-Length body полностью
                    if let Some(header_end) = total.windows(4).position(|w| w == b"\r\n\r\n") {
                        let headers = String::from_utf8_lossy(&total[..header_end]);
                        if let Some(cl_line) = headers.lines().find(|l| l.to_lowercase().starts_with("content-length:")) {
                            let expected: usize = cl_line.split(':').nth(1).unwrap_or("0").trim().parse().unwrap_or(0);
                            if total.len() >= header_end + 4 + expected {
                                break;
                            }
                        }
                    }
                }
                Ok(Err(e)) => {
                    tracing::warn!("Upstream read error: {}", e);
                    break;
                }
                Err(_) => {
                    tracing::debug!("Read timeout, sending what we have ({} bytes)", total.len());
                    break;
                }
            }
        }
        
        tracing::debug!("Response received from upstream ({} bytes)", total.len());
        
        // If no response received, return 502 Bad Gateway
        if total.is_empty() {
            let resp = b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 15\r\nConnection: close\r\n\r\n502 Bad Gateway";
            client_stream.write_all(resp).await
                .map_err(|e| crate::error::ProxyError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
            client_stream.flush().await
                .map_err(|e| crate::error::ProxyError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
            
            tracing::warn!("No response received from upstream {}", addr);
            return Ok(());
        }
        
        // 7. Write response to client
        client_stream.write_all(&total).await
            .map_err(|e| crate::error::ProxyError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
        client_stream.flush().await
            .map_err(|e| crate::error::ProxyError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
        
        // 8. Handle keep-alive: close connection if "Connection: close" header present
        let has_connection_close = total.windows(17)
            .any(|w| w.eq_ignore_ascii_case(b"connection: close\r\n"));
        let has_keep_alive = total.windows(20)
            .any(|w| w.eq_ignore_ascii_case(b"connection: keep-alive\r\n"));
        
        if has_connection_close || (!has_keep_alive && !has_connection_close) {
            // HTTP/1.1 default is keep-alive, but if server said close, close it
            if has_connection_close {
                tracing::debug!("Connection: close from upstream, closing client connection");
                // Gracefully shutdown the write half to signal close
                use tokio::io::AsyncWriteExt;
                let _ = client_stream.shutdown().await;
            }
        }
        
        // Per-flow logging - parse status code from response
        let status = total
            .windows(12)
            .find_map(|w| {
                let line = String::from_utf8_lossy(w);
                if line.starts_with("HTTP/1.") {
                    line.split_whitespace().nth(1).and_then(|s| s.parse::<u16>().ok())
                } else { None }
            })
            .unwrap_or(0);
        
        tracing::info!(
            method = %parts.get(0).unwrap_or(&"UNKNOWN"),
            url = url,
            status = status,
            "{} {} → {}",
            parts.get(0).unwrap_or(&"UNKNOWN"),
            url,
            status
        );
        
        info!("Sent HTTP response ({} bytes) to {}", total.len(), addr);
        Ok(())
    }
}

/// Handle a connection based on detected protocol.
pub async fn handle_connection(
    mut stream: TcpStream,
    addr: SocketAddr,
    protocol: Protocol,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("Protocol detected: {:?} from {}", protocol, addr);
    
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
            // Read the HTTP request first
            let mut buf = vec![0u8; 65536];
            let n = stream.read(&mut buf).await
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;
            
            // Forward the request to upstream
            HttpForwarder.forward(&mut stream, &buf[..n]).await
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;
            Ok(())
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
