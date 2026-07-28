## 1. Project Setup

- [ ] 1.1 Add dependencies to workspace Cargo.toml: x509-parser, rcgen, ring, pkcs8, pem, indexmap
- [ ] 1.2 Add dependencies to mitm-certs Cargo.toml
- [ ] 1.3 Verify workspace builds with `cargo build` before any code changes
- [ ] 1.4 Set up `mitm-certs` crate structure: lib.rs, certs/, store/, config/

## 2. Certificate Types (mitm-certs)

- [ ] 2.1 Define `Cert` struct wrapping `x509_parser::certificate::X509Certificate<'static>`
- [ ] 2.2 Implement `Cert::from_der(bytes) -> Result<Self>` parser
- [ ] 2.3 Implement `Cert::to_pem() -> Result<String>` serializer
- [ ] 2.4 Implement `Cert::to_der() -> Result<Vec<u8>>` serializer
- [ ] 2.5 Implement `Cert::fingerprint_sha256() -> String` (colon-separated hex)
- [ ] 2.6 Implement `Cert::subject() -> &str` accessor
- [ ] 2.7 Implement `Cert::issuer() -> &str` accessor
- [ ] 2.8 Implement `Cert::not_before() -> DateTime` accessor
- [ ] 2.9 Implement `Cert::not_after() -> DateTime` accessor
- [ ] 2.10 Implement `Cert::is_ca() -> bool` accessor
- [ ] 2.11 Implement `Cert::san_dns_names() -> Vec<String>` accessor
- [ ] 2.12 Implement `Cert::san_ip_addresses() -> Vec<String>` accessor
- [ ] 2.13 Add serde Serialize/Deserialize for Cert
- [ ] 2.14 Write unit tests: parse well-known certs, fingerprint determinism, SAN extraction

## 3. Key Pair Types (mitm-certs)

- [ ] 3.1 Define `KeyPair` struct wrapping ring key material
- [ ] 3.2 Implement `KeyPair::generate_rsa(bits: u16) -> Result<Self>` (2048 or 4096)
- [ ] 3.3 Implement `KeyPair::generate_ec(curve: EcCurve) -> Result<Self>` (P256 or P384)
- [ ] 3.4 Implement `KeyPair::to_pkcs8_pem() -> Result<String>` serializer
- [ ] 3.5 Implement `KeyPair::from_pkcs8_pem(pem: &str) -> Result<Self>` parser
- [ ] 3.6 Implement `KeyPair::public_key_der() -> Result<Vec<u8>>` accessor
- [ ] 3.7 Add serde Serialize/Deserialize for KeyPair
- [ ] 3.8 Write unit tests: RSA key generation, EC key generation, PEM round-trip

## 4. Certificate Authority (mitm-certs)

- [ ] 4.1 Define `CaRoot` struct with certificate, key, and metadata
- [ ] 4.2 Implement `CaRoot::generate(cn: &str, validity_days: u32) -> Result<Self>`
- [ ] 4.3 Implement CA root key generation (RSA 2048 or EC P-256)
- [ ] 4.4 Implement CA root certificate generation with CA=TRUE, pathlen=0
- [ ] 4.5 Implement `CaRoot::to_pem_files(dir: &Path) -> Result<()>` persistence
- [ ] 4.6 Implement `CaRoot::load_from_dir(dir: &Path) -> Result<Self>` persistence
- [ ] 4.7 Define `IntermediateCa` struct with certificate, key, and root reference
- [ ] 4.8 Implement `IntermediateCa::generate(root: &CaRoot, cn: &str, validity_days: u32) -> Result<Self>`
- [ ] 4.9 Implement intermediate CA signing by root CA
- [ ] 4.10 Implement `IntermediateCa::to_pem_files(dir: &Path) -> Result<()>` persistence
- [ ] 4.11 Write unit tests: CA generation, intermediate signing, PEM persistence round-trip

## 5. End-Entity Certificates (mitm-certs)

- [ ] 5.1 Define `EndEntityCert` struct with certificate, key, and signing CA reference
- [ ] 5.2 Implement `EndEntityCert::generate(ca: &IntermediateCa, cn: &str, validity_days: u32, sans: &[SanType]) -> Result<Self>`
- [ ] 5.3 Support DNS SANs (e.g., "example.com")
- [ ] 5.4 Support IP SANs (e.g., "127.0.0.1", "::1")
- [ ] 5.5 Sign end-entity cert with intermediate CA key
- [ ] 5.6 Implement `EndEntityCert::to_pem_files(dir: &Path) -> Result<()>` persistence
- [ ] 5.7 Implement `EndEntityCert::build_chain(&self, intermediate: &IntermediateCa, root: &CaRoot) -> Vec<Cert>` chain builder
- [ ] 5.8 Write unit tests: EE cert with DNS SANs, EE cert with IP SANs, chain building, chain verification

## 6. Certificate Store (mitm-certs)

- [ ] 6.1 Define `CertStore` struct with in-memory storage and LRU policy
- [ ] 6.2 Implement `CertStore::new(max_entries: usize) -> Self`
- [ ] 6.3 Implement `store.store(domain: &str, cert: Cert, key: KeyPair) -> Result<()>`
- [ ] 6.4 Implement `store.get(domain: &str) -> Option<(&Cert, &KeyPair)>` lookup
- [ ] 6.5 Implement `store.get_by_fingerprint(fingerprint: &str) -> Option<&Cert>` lookup
- [ ] 6.6 Implement LRU eviction when store exceeds max_entries
- [ ] 6.7 Implement `store.remove(domain: &str) -> Result<()>`
- [ ] 6.8 Implement `store.len() -> usize` accessor
- [ ] 6.9 Implement `store.clear()`
- [ ] 6.10 Write unit tests: store/retrieve, eviction, fingerprint lookup, remove

## 7. Persistence (mitm-certs)

- [ ] 7.1 Define `CertStore::persist(dir: &Path) -> Result<()>` for directory-based persistence
- [ ] 7.2 Write CA root cert + key as PEM files
- [ ] 7.3 Write intermediate CA cert + key as PEM files
- [ ] 7.4 Write each EE cert + key as PEM files in certs/ subdirectory
- [ ] 7.5 Implement `CertStore::load_from_dir(dir: &Path) -> Result<Self>` for directory-based loading
- [ ] 7.6 Atomic save: write to temp file, then rename
- [ ] 7.7 Set file permissions to 0600 on Unix
- [ ] 7.8 Write unit tests: persist + reload round-trip, atomic save

## 8. TLS Configuration (mitm-certs)

- [ ] 8.1 Define `TlsServerConfig` struct wrapping rustls::ServerConfig
- [ ] 8.2 Implement `TlsServerConfig::new(store: Arc<CertStore>) -> Self`
- [ ] 8.3 Implement SNI-based certificate selection from store
- [ ] 8.4 Configure TLS 1.2 + 1.3 support (disable 1.0/1.1)
- [ ] 8.5 Configure cipher suites: prefer AEAD (AES-GCM, ChaCha20-Poly1305)
- [ ] 8.6 Configure ALPN: support "h2" and "http/1.1"
- [ ] 8.7 Set session ticket support for TLS 1.2 resumption
- [ ] 8.8 Provide `into_rustls_server_config(self) -> Result<Arc<rustls::ServerConfig>>`
- [ ] 8.9 Define `TlsClientConfig` struct wrapping rustls::ClientConfig
- [ ] 8.10 Implement `TlsClientConfig::new() -> Self` with default verification
- [ ] 8.11 Implement `TlsClientConfig::with_custom_ca(ca_pem: &str) -> Result<Self>` for self-signed servers
- [ ] 8.12 Configure SNI: enabled by default
- [ ] 8.13 Configure ALPN: configurable list
- [ ] 8.14 Provide `into_rustls_client_config(self) -> Result<Arc<rustls::ClientConfig>>`
- [ ] 8.15 Write unit tests: server config creation, client config with custom CA

## 9. Connection Integration (mitm-core)

- [ ] 9.1 Update `mitm-core` Cert type to use new `mitm_certs::Cert`
- [ ] 9.2 Update `Connection.certificate_list` type to `Vec<mitm_certs::Cert>`
- [ ] 9.3 Update serialization for new Cert type
- [ ] 9.4 Update existing tests for new Cert type
- [ ] 9.5 Run `cargo test` — verify all tests pass
- [ ] 9.6 Run `cargo clippy -- -D warnings` — fix any warnings

## 10. Integration & Validation

- [ ] 10.1 Run `cargo build` — verify all crates compile
- [ ] 10.2 Run `cargo test` — verify all unit tests pass (target: 40+ tests)
- [ ] 10.3 Test full CA hierarchy: generate root → intermediate → EE cert
- [ ] 10.4 Test chain building and verification
- [ ] 10.5 Test store LRU eviction with many entries
- [ ] 10.6 Test persistence round-trip (save → load)
- [ ] 10.7 Run clippy: `cargo clippy -- -D warnings`
- [ ] 10.8 Document public API with doc comments on all public types and methods
