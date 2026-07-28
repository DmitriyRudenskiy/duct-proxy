//! End-to-end TLS interception test.

use mitm_certs::ca::CaRoot;
use mitm_certs::store::CertStore;

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
