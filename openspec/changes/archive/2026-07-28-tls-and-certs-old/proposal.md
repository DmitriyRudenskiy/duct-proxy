## Why

The `mitmproxy-rs` data model references `Cert` in `Connection.certificate_list` but the type is a placeholder (`Vec<u8>` with optional CN). The `mitm-certs` crate is empty. Without a real certificate system, the proxy cannot perform TLS interception (the core MITM functionality), validate peer certificates, or present its own CA-signed certificates to clients. This change implements the certificate management and TLS handshake infrastructure that enables HTTPS interception.

## What Changes

- Replace placeholder `Cert` type with a real certificate representation wrapping `x509-parser`
- Implement CA certificate generation (root CA + per-domain intermediate CAs)
- Implement per-domain certificate generation (end-entity certs signed by intermediate CA)
- Implement certificate store: in-memory cache with LRU eviction and file-based persistence
- Define TLS configuration types: `TlsConfig`, `TlsServerConfig`, `TlsClientConfig`
- Add ALPN negotiation support and SNI-based cert selection
- Add certificate verification for server connections (peer certificate validation)
- Add certificate chain building and serialization (PEM/DER)

## Capabilities

### New Capabilities

- `tls-certs`: Certificate generation: CA root, intermediate CA, end-entity cert signing with SAN support
- `tls-store`: Certificate store: in-memory LRU cache, on-disk persistence, key generation (RSA/EC)
- `tls-config`: TLS configuration: server config with SNI dispatch, client config with verification, ALPN negotiation

### Modified Capabilities

- `connection-model`: The `Cert` type changes from placeholder (`Vec<u8>`) to real certificate struct with parsing, fingerprinting, and chain operations

## Impact

- **crates/mitm-certs**: Major implementation — CA generation, cert signing, store, config
- **crates/mitm-core**: `Cert` type replacement in `connection.rs`
- **crates/mitm-proxy**: Will consume TLS config for server/client TLS setup (future)
- **External dependencies**: `x509-parser`, `rcgen`, `rustls`, `pkcs8`, `pem`
