//! End-to-end TLS interception test.

use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName};
use rustls::ClientConfig;
use rustls::RootCertStore;
use rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;
use tokio_rustls::TlsConnector;
use mitm_certs::ca::CaRoot;
use mitm_certs::leaf::LeafCert;
use mitm_certs::store::CertStore;
use pem::Pem;

/// Test real TLS interception handshake through intercept_tls().
///
/// This test verifies that:
/// 1. intercept_tls() successfully intercepts a TLS connection
/// 2. Generates a valid leaf certificate signed by our CA
/// 3. Completes the TLS handshake with the client
/// 4. The client receives a valid server certificate
#[tokio::test]
async fn test_intercept_tls_real_handshake() {
    // 1. Create CA and CertStore
    let ca = CaRoot::generate("Test MITM CA").expect("Failed to generate CA");
    let mut store = CertStore::new(64);

    // 2. Start proxy listener
    let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_port = proxy_listener.local_addr().unwrap().port();

    // Spawn client task that will connect to proxy
    let ca_for_client = ca.clone();
    let client_task = tokio::spawn(async move {
        let client_stream = TcpStream::connect(format!("127.0.0.1:{}", proxy_port)).await.unwrap();

        let mut root_store = RootCertStore::empty();
        root_store.add_parsable_certificates(
            vec![CertificateDer::from(ca_for_client.cert_der().to_vec())],
        );

        let client_config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        let connector = TlsConnector::from(Arc::new(client_config));

        let server_name: ServerName<'static> = "localhost"
            .try_into()
            .unwrap();

        connector
            .connect(server_name, client_stream)
            .await
            .unwrap()
    });

    // Accept client connection
    let (client_stream, _) = proxy_listener.accept().await.unwrap();

    // Perform TLS interception (without upstream for this test)
    // We'll use a non-existent upstream to test just the client-side interception
    let intercept_result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        mitm_proxy::intercept_tls(
            client_stream,
            &ca,
            &mut store,
            Some("127.0.0.1:1"), // Non-existent upstream
        ),
    )
    .await;

    // The interception should succeed up to the point where it tries to connect to upstream
    // We expect it to fail at the upstream connection, but the client-side should work
    match intercept_result {
        Ok(Ok((_client_tls, _upstream_tls, sni))) => {
            // If we got here, the full interception worked
            assert_eq!(sni, "localhost");
            let leaf = store.get("localhost").expect("Leaf cert should exist");
            assert_eq!(leaf.domain(), "localhost");
        }
        Ok(Err(e)) => {
            // Expected: upstream connection fails
            // But the client-side TLS should have worked
            let err_msg = format!("{:?}", e);
            assert!(
                err_msg.contains("UpstreamTls") || err_msg.contains("Connection") || err_msg.contains("UpstreamConnect"),
                "Expected upstream error, got: {}",
                err_msg
            );
            // Verify the leaf cert was generated before the upstream failure
            let leaf = store.get("localhost").expect("Leaf cert should exist even if upstream fails");
            assert_eq!(leaf.domain(), "localhost");
        }
        Err(_) => panic!("intercept_tls timed out"),
    }

    // Wait for client task to complete
    let _client_tls = client_task.await.unwrap();
}

/// Test TLS interception certificate generation.
///
/// This test verifies that:
/// 1. The CA can be generated
/// 2. The CertStore can generate leaf certificates
/// 3. The leaf certificate is properly cached
#[test]
fn test_tls_interception_cert_generation() {
    // 1. Create CA
    let ca = CaRoot::generate("Test MITM CA").expect("Failed to generate CA");

    // 2. Create CertStore
    let mut cert_store = CertStore::new(10);

    // 3. Generate leaf certificate for "localhost"
    let sni = "localhost";
    let leaf = cert_store.get_or_generate(sni, &ca).expect("Failed to generate leaf cert");

    // 4. Verify the certificate was generated
    assert_eq!(leaf.domain(), sni);

    // 5. Verify the certificate is cached
    let cached = cert_store.get(sni).expect("Leaf cert should be cached");
    assert_eq!(cached.domain(), sni);

    // 6. Verify the certificate has DER format
    assert!(!leaf.cert_der().is_empty(), "Certificate DER should not be empty");
    assert!(!leaf.key_pkcs8().is_empty(), "Key PKCS8 should not be empty");

    // 7. Generate another certificate to verify LRU behavior
    let leaf2 = cert_store.get_or_generate("example.com", &ca).expect("Failed to generate leaf cert");
    assert_eq!(leaf2.domain(), "example.com");

    // 8. Verify both certificates exist
    assert!(cert_store.get("localhost").is_some());
    assert!(cert_store.get("example.com").is_some());
}

/// Test that the CertStore respects max_entries
#[test]
fn test_cert_store_lru() {
    let ca = CaRoot::generate("Test CA").unwrap();
    let mut store = CertStore::new(2);

    // Insert two certificates
    store.insert(&ca, "a.com").unwrap();
    store.insert(&ca, "b.com").unwrap();

    // Access a.com to make it recently used
    store.get("a.com");

    // Insert a new certificate, which should evict b.com (LRU)
    store.insert(&ca, "c.com").unwrap();

    assert!(store.get("a.com").is_some(), "a.com should still exist");
    assert!(store.get("b.com").is_none(), "b.com should be evicted");
    assert!(store.get("c.com").is_some(), "c.com should exist");
}
