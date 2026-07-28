//! Proxy server implementation with TCP listener and accept loop.

use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::task::JoinSet;
use tracing::{info, error};

use mitm_options::Options;

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

    /// Run the accept loop, handling connections until shutdown.
    pub async fn run(mut self, listener: TcpListener) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting accept loop...");

        // Handle graceful shutdown
        let shutdown = tokio::signal::ctrl_c();

        tokio::select! {
            result = self.accept_loop(listener) => {
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
    async fn accept_loop(&mut self, listener: TcpListener) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        loop {
            match listener.accept().await {
                Ok((mut stream, addr)) => {
                    info!("New connection from {}", addr);

                    let active = Arc::clone(&self.active_connections);
                    let mut connections = active.lock().await;
                    *connections += 1;
                    drop(connections);

                    let mut join_set = tokio::task::JoinSet::new();
                    join_set.spawn(async move {
                        // Connection handler would go here
                        info!("Handling connection from {}", addr);
                        // For now, just read and discard
                        let mut buf = vec![0u8; 4096];
                        match stream.read(&mut buf).await {
                            Ok(n) => info!("Read {} bytes from {}", n, addr),
                            Err(e) => error!("Read error from {}: {}", addr, e),
                        }
                        let mut connections = active.lock().await;
                        *connections -= 1;
                    });

                    // Store the join handle for later
                    self.join_set.spawn(async move {
                        join_set.join_next().await;
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
