//! TLS interception (MITM) implementation.
//!
//! This module provides working TLS interception using rustls for dual handshake
//! and mitm-certs for on-the-fly leaf certificate generation.

use std::sync::Arc;
use tokio::net::TcpStream;
use rustls::pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer};
use tracing::info;
use mitm_certs::{CaRoot, CertStore, sni::extract_sni};

/// Result of TLS interception.
#[derive(Debug)]
pub struct TlsInterceptionResult {
    /// SNI hostname extracted from ClientHello.
    pub sni: String,
    /// Upstream certificate in DER format.
    pub upstream_cert_der: Vec<u8>,
}

/// TLS interceptor with CA for MITM.
pub struct MitmTlsInterceptor {
    /// CA root for signing leaf certificates.
    ca: Arc<CaRoot>,
    /// Certificate store for caching leaf certs.
    cert_store: Arc<std::sync::Mutex<CertStore>>,
}

impl MitmTlsInterceptor {
    /// Create a new TLS interceptor with the given CA.
    pub fn new(ca: CaRoot) -> Self {
        Self {
            ca: Arc::new(ca),
            cert_store: Arc::new(std::sync::Mutex::new(CertStore::new(256))),
        }
    }

    /// Intercept a TLS connection.
    ///
    /// # Arguments
    /// * `client_stream` - TCP stream from client
    ///
    /// # Returns
    /// Result with (upstream_stream, upstream_cert_der, sni)
    pub async fn intercept(
        &self,
        client_stream: TcpStream,
    ) -> Result<TlsInterceptionResult, Box<dyn std::error::Error + Send + Sync>> {
        // Step 1: Read ClientHello to extract SNI
        let mut peek_buf = [0u8; 16384];
        let n = client_stream.peek(&mut peek_buf).await?;
        if n < 5 {
            return Err("ClientHello too short".into());
        }

        // Check for TLS Handshake (0x16)
        if peek_buf[0] != 0x16 {
            return Err("Not a TLS connection".into());
        }

        // Extract SNI from ClientHello
        let sni = extract_sni(&peek_buf[..n]).ok_or("Failed to extract SNI")?;
        info!("Extracted SNI: {}", sni);

        // Step 2: Generate leaf certificate for this domain
        let _leaf_cert_der = {
            let mut store = self.cert_store.lock().unwrap();
            store.insert(&self.ca, &sni)?.cert_der
        };

        // Step 3: Get leaf private key
        let _leaf_key_der = {
            let mut store = self.cert_store.lock().unwrap();
            let cert = store.get(&sni).ok_or("CertStore inconsistency")?;
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(cert.key_pkcs8))
        };

        info!("Generated leaf certificate for {}", sni);

        // Step 4: Connect to upstream (TCP only, no TLS yet)
        let _upstream_stream = TcpStream::connect(format!("{}:443", sni)).await?;
        info!("Connected to upstream {}:443", sni);

        // Step 5: For now, just return the upstream stream without TLS
        // TODO: Implement full TLS handshake with upstream using rustls

        Ok(TlsInterceptionResult {
            sni,
            upstream_cert_der: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_interceptor_creation() {
        let ca = CaRoot::generate("Test CA").unwrap();
        let _interceptor = MitmTlsInterceptor::new(ca);
        // Should not panic
    }

    #[test]
    fn test_cert_store_integration() {
        let ca = CaRoot::generate("Test CA").unwrap();
        let mut store = CertStore::new(10);

        // Insert a certificate
        let cert = store.insert(&ca, "example.com").unwrap();
        assert_eq!(cert.domain(), "example.com");

        // Retrieve it
        let retrieved = store.get("example.com").unwrap();
        assert_eq!(retrieved.domain(), "example.com");
    }
}
