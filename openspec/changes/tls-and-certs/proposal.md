## Why

mitmproxy-rs requires certificate management to perform TLS interception (MITM). The `mitm-certs` crate is empty, and the `Cert` type in `mitm-core` is a placeholder. Without real certificates, the proxy cannot intercept HTTPS traffic — the core functionality.

This change implements the certificate infrastructure: CA generation, on-the-fly leaf certificate generation, SNI-based cert selection, and PEM storage. This enables the TLS interception pipeline that downstream crates (mitm-proxy, mitm-net) will consume.

## What Changes

- Implement CA generation using `rcgen` with ECDSA P-256 (single key type for v1)
- Implement `CertStore` with LRU cache for in-memory cert/key storage
- Implement on-the-fly leaf certificate generation signed by CA
- Implement SNI extraction from raw TLS ClientHello bytes
- Implement upstream server certificate sniffing (extract cert from TLS handshake)
- Implement PEM file persistence at `~/.mitmproxy/`
- Replace placeholder `Cert` type in `mitm-core` with real certificate wrapper

## Capabilities

### New Capabilities

- `tls-ca`: CA certificate generation (ECDSA P-256, root + intermediate)
- `tls-store`: Certificate store with LRU eviction and PEM persistence
- `tls-sni`: SNI extraction from raw ClientHello bytes
- `tls-leaf`: On-the-fly leaf certificate generation signed by CA

### Modified Capabilities

- `connection-model`: `Cert` type changes from placeholder (`Vec<u8>`) to real certificate wrapper

## Impact

- **crates/mitm-certs**: Major implementation — CA, store, SNI, leaf certs, PEM
- **crates/mitm-core**: Update `Cert` type to use new certificate wrapper
- **crates/mitm-proxy**: Will consume CertStore for TLS interception (future)
- **External dependencies**: `rcgen`, `x509-parser`, `pem`, `pkcs8`
