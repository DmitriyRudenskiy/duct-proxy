//! Proxy server implementation with TCP listener and accept loop.

use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::task::JoinSet;
use tracing::{info, error, warn};

use mitm_addons::AddonManager;
use mitm_certs::{CaRoot, CertStore};
use mitm_options::Options;

use crate::error::ProxyError;
use crate::handler::{detect_protocol_from_bytes, HttpForwarder, Protocol};
use crate::tls::{intercept_tls, forward_bidirectional};

/// Proxy server configuration.
#[derive(Clone, Debug)]
pub struct ProxyConfig {
    /// Listen address (e.g., "0.0.0.0:8080").
    pub listen_addr: String,
    /// Maximum connections to handle concurrently.
    pub max_connections: usize,
    /// Enable TLS interception.
    pub tls_interception: bool,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0:8080".to_string(),
            max_connections: 1000,
            tls_interception: true,
        }
    }
}

/// Main proxy server struct.
pub struct ProxyServer {
    /// Server configuration.
    config: ProxyConfig,
    /// JoinSet for managing connection handler tasks.
    join_set: JoinSet<()>,
    /// Shared state for connection counting.
    active_connections: Arc<Mutex<usize>>,
}

impl ProxyServer {
    /// Create a new ProxyServer with the given configuration.
    pub fn new(config: ProxyConfig) -> Self {
        Self {
            config,
            join_set: JoinSet::new(),
            active_connections: Arc::new(Mutex::new(0)),
        }
    }

    /// Create a ProxyServer from Options struct.
    pub fn from_options(opts: &Options) -> Result<Self, String> {
        let addr = format!("{}:{}", opts.listen_host, opts.listen_port);
        let config = ProxyConfig {
            listen_addr: addr,
            max_connections: 1000,
            tls_interception: !opts.ssl_insecure,
        };
        Ok(Self::new(config))
    }

    /// Bind to the configured address and return a TcpListener.
    pub async fn bind(&self) -> Result<TcpListener, Box<dyn std::error::Error + Send + Sync>> {
        let listener = TcpListener::bind(&self.config.listen_addr).await?;
        info!("Proxy server listening on {}", self.config.listen_addr);
        Ok(listener)
    }

    /// Run the accept loop with CA, cert store, and addon manager.
    pub async fn run(
        mut self,
        listener: TcpListener,
        ca: Arc<CaRoot>,
        cert_store: Arc<Mutex<CertStore>>,
        addon_mgr: Arc<Mutex<AddonManager>>,
    ) -> Result<(), ProxyError> {
        info!("Starting accept loop...");

        // Handle graceful shutdown
        let shutdown = tokio::signal::ctrl_c();

        tokio::select! {
            result = self.accept_loop(listener, ca, cert_store, addon_mgr) => {
                result?;
            }
            _ = shutdown => {
                info!("Shutdown signal received, draining connections...");
            }
        }

        // Wait for all connection handlers to complete
        info!("Waiting for {} active connections to complete...", self.join_set.len());
        while self.join_set.join_next().await.is_some() {}
        info!("All connections drained. Server stopped.");

        Ok(())
    }

    /// Accept loop: accept connections and spawn handlers.
    async fn accept_loop(
        &mut self,
        listener: TcpListener,
        ca: Arc<CaRoot>,
        cert_store: Arc<Mutex<CertStore>>,
        addon_mgr: Arc<Mutex<AddonManager>>,
    ) -> Result<(), ProxyError> {
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    info!("New connection from {}", addr);

                    let active = Arc::clone(&self.active_connections);
                    let mut connections = active.lock().await;
                    *connections += 1;
                    drop(connections);

                    // Clone Arc references for the async block
                    let ca_clone = Arc::clone(&ca);
                    let cert_store_clone = Arc::clone(&cert_store);
                    let addon_mgr_clone = Arc::clone(&addon_mgr);

                    // Spawn connection handler
                    self.join_set.spawn(async move {
                        let result = handle_connection(stream, addr, ca_clone, cert_store_clone, addon_mgr_clone).await;
                        if let Err(e) = handle_connection_result(result, addr) {
                            error!("Connection handler error from {}: {}", addr, e);
                        }
                        let mut connections = active.lock().await;
                        *connections -= 1;
                    });
                }
                Err(e) => {
                    error!("Accept error: {}", e);
                    // Continue accepting other connections
                }
            }
        }
    }

    /// Get the number of active connections.
    pub async fn active_connections(&self) -> usize {
        *self.active_connections.lock().await
    }
}

/// Handle a connection based on detected protocol.
async fn handle_connection(
    mut stream: TcpStream,
    peer_addr: std::net::SocketAddr,
    ca: Arc<CaRoot>,
    cert_store: Arc<Mutex<CertStore>>,
    _addon_mgr: Arc<Mutex<AddonManager>>,
) -> Result<(), ProxyError> {
    // 1. Peek first bytes to detect protocol
    let mut peek_buf = [0u8; 8192];
    let n = stream.peek(&mut peek_buf).await
        .map_err(|e| ProxyError::Io(e.into()))?;
    if n == 0 { return Ok(()); }
    
    info!("Read {} bytes from {}", n, peer_addr);
    
    let protocol = detect_protocol_from_bytes(&peek_buf[..n]);
    info!("Protocol: {:?} from {}", protocol, peer_addr);
    
    // Handle connection based on protocol
    match protocol {
        Protocol::HttpConnect => {
            // CONNECT example.com:443 HTTP/1.1
            // 1. Read the full CONNECT request
            let mut buf = [0u8; 8192];
            let n = stream.read(&mut buf).await
                .map_err(|e| ProxyError::Io(e.into()))?;
            let request = String::from_utf8_lossy(&buf[..n]);
            
            // 2. Parse target host:port
            let target = request.lines().next()
                .and_then(|l| l.split_whitespace().nth(1))
                .unwrap_or("");
            info!("CONNECT tunnel to: {}", target);
            
            // 3. Respond 200 Connection Established
            stream.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n").await
                .map_err(|e| ProxyError::Io(e.into()))?;
            stream.flush().await
                .map_err(|e| ProxyError::Io(e.into()))?;
            
            // 4. TLS interception (MITM)
            let mut store = cert_store.lock().await;
            match intercept_tls(stream, &ca, &mut store, None).await {
                Ok((client_tls, upstream_tls, sni)) => {
                    info!("TLS intercepted: {}", sni);
                    // 5. Forward bidirectionally
                    forward_bidirectional(client_tls, upstream_tls).await?;
                }
                Err(e) => {
                    error!("TLS interception failed: {}", e);
                }
            }
        }
        
        Protocol::Http => {
            // Plain HTTP: GET http://example.com/ HTTP/1.1
            let mut buf = [0u8; 65536];
            let n = stream.read(&mut buf).await
                .map_err(|e| ProxyError::Io(e.into()))?;
            
            let forwarder = HttpForwarder;
            forwarder.forward(&mut stream, &buf[..n]).await?;
        }
        
        Protocol::Tls => {
            // Direct TLS (transparent mode) — same as CONNECT but no 200 response
            let mut store = cert_store.lock().await;
            match intercept_tls(stream, &ca, &mut store, None).await {
                Ok((client_tls, upstream_tls, sni)) => {
                    info!("TLS intercepted (transparent): {}", sni);
                    forward_bidirectional(client_tls, upstream_tls).await?;
                }
                Err(e) => {
                    error!("TLS interception failed: {}", e);
                }
            }
        }
        
        Protocol::Raw => {
            warn!("Unknown protocol from {}, closing", peer_addr);
        }
    }
    
    Ok(())
}

/// Handle result from connection handler, filtering out client disconnects.
fn handle_connection_result(
    result: Result<(), ProxyError>,
    peer_addr: std::net::SocketAddr,
) -> Result<(), ProxyError> {
    match result {
        Err(ProxyError::Io(e)) if e.kind() == std::io::ErrorKind::ConnectionReset => {
            tracing::debug!("Client disconnected: {}", peer_addr);
            Ok(())
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_proxy_server_creation() {
        let config = ProxyConfig::default();
        let server = ProxyServer::new(config);
        assert_eq!(server.active_connections().await, 0);
    }

    #[tokio::test]
    async fn test_proxy_server_bind() {
        let config = ProxyConfig {
            listen_addr: "127.0.0.1:0".to_string(), // Port 0 = random port
            ..Default::default()
        };
        let server = ProxyServer::new(config);
        let listener = server.bind().await.unwrap();
        let addr = listener.local_addr().unwrap();
        assert!(addr.port() > 0);
    }

    #[test]
    fn test_proxy_server_from_options() {
        let opts = Options::default();
        let server = ProxyServer::from_options(&opts);
        assert!(server.is_ok());
    }
}
