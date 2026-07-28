//! TLS interception: dual handshake MITM using rustls.

use std::sync::Arc;

use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName};
use rustls::ServerConfig;
use tokio::net::TcpStream;
use tokio_rustls::TlsAcceptor;
use tokio_rustls::TlsConnector;

use mitm_certs::ca::CaRoot;
use mitm_certs::store::CertStore;

use crate::error::ProxyError;

/// Decode PEM-encoded PKCS8 private key to DER bytes.
fn decode_pem_key(pem_bytes: &[u8]) -> Result<Vec<u8>, ProxyError> {
    let pem = pem::parse(pem_bytes)
        .map_err(|e| ProxyError::TlsConfig(format!("Failed to parse PEM: {}", e)))?;
    
    Ok(pem.contents().to_vec())
}

/// Performs full MITM TLS interception.
///
/// 1. Peek ClientHello → extract SNI
/// 2. Generate leaf cert for SNI (signed by our CA)
/// 3. TLS-accept from client using leaf cert
/// 4. TCP-connect to upstream
/// 5. TLS-connect to upstream
/// 6. Return both decrypted streams
///
/// # Arguments
/// * `client_tcp` - TCP stream from client
/// * `ca` - CA root for signing leaf certificates
/// * `cert_store` - Certificate store for caching
/// * `upstream_override` - Optional override for upstream address (format: "host:port")
///  If None, uses "{sni}:443"
pub async fn intercept_tls(
    client_tcp: TcpStream,
    ca: &CaRoot,
    cert_store: &mut CertStore,
    upstream_override: Option<&str>,
) -> Result<
    (
        tokio_rustls::server::TlsStream<TcpStream>,
        tokio_rustls::client::TlsStream<TcpStream>,
        String, // SNI hostname
    ),
    ProxyError,
> {
    // ── 1. Peek ClientHello to extract SNI ──────────────────────────
    let mut peek_buf = [0u8; 4096];
    let n = client_tcp.peek(&mut peek_buf).await?;
    let sni = mitm_certs::sni::extract_sni(&peek_buf[..n])
        .ok_or(ProxyError::NoSni)?;

    tracing::info!(sni = %sni, "TLS interception starting");

    // ── 2. Get or generate leaf certificate ─────────────────────────
    let leaf = cert_store
        .get_or_generate(&sni, ca)
        .map_err(|e| ProxyError::CertGeneration(e.to_string()))?;

    // ── 3. Build rustls ServerConfig with leaf cert ─────────────────
    let cert_chain: Vec<CertificateDer<'static>> = vec![
        CertificateDer::from(leaf.cert_der().to_vec()),
        CertificateDer::from(ca.cert_der().to_vec()),
    ];
    let key_der_bytes = decode_pem_key(leaf.key_pkcs8())?;
    let key_der = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key_der_bytes));

    let server_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key_der)
        .map_err(|e| ProxyError::TlsConfig(e.to_string()))?;

    let acceptor = TlsAcceptor::from(Arc::new(server_config));

    // ── 4. Accept TLS from client ───────────────────────────────────
    let client_tls = acceptor
        .accept(client_tcp)
        .await
        .map_err(|e| ProxyError::TlsHandshake(e.to_string()))?;

    // ── 5. Connect to upstream (TCP + TLS) ──────────────────────────
    let upstream_addr = upstream_override
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{}:443", sni));
    let upstream_tcp = TcpStream::connect(&upstream_addr).await?;

    let mut root_store = rustls::RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    let client_config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let connector = TlsConnector::from(Arc::new(client_config));
    let server_name: ServerName<'static> = sni
        .clone()
        .try_into()
        .map_err(|_| ProxyError::InvalidSni(sni.clone()))?;

    let upstream_tls = connector
        .connect(server_name, upstream_tcp)
        .await
        .map_err(|e| ProxyError::UpstreamTls(e.to_string()))?;

    tracing::info!(sni = %sni, "TLS interception complete, both handshakes done");

    Ok((client_tls, upstream_tls, sni))
}

/// Bidirectional copy between two decrypted TLS streams.
/// Returns bytes transferred (client→upstream, upstream→client).
pub async fn forward_bidirectional(
    client: tokio_rustls::server::TlsStream<TcpStream>,
    upstream: tokio_rustls::client::TlsStream<TcpStream>,
) -> Result<(u64, u64), ProxyError> {
    let (mut client_read, mut client_write) = tokio::io::split(client);
    let (mut upstream_read, mut upstream_write) = tokio::io::split(upstream);

    let c2u = tokio::io::copy(&mut client_read, &mut upstream_write);
    let u2c = tokio::io::copy(&mut upstream_read, &mut client_write);

    tokio::select! {
        result = c2u => {
            let bytes = result?;
            Ok((bytes, 0))
        }
        result = u2c => {
            let bytes = result?;
            Ok((0, bytes))
        }
    }
}
