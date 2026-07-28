# Changelog

## [0.1.0] - 2026-07-29

### Added
- HTTP explicit proxy with upstream forwarding
- HTTPS MITM interception (TLS 1.3, ECDSA P-256)
- On-the-fly certificate generation with LRU cache
- CA certificate management (~/.mitmproxy/)
- Addon system: ModifyHeaders, ModifyBody, Block
- Protocol detection (HTTP, TLS, CONNECT)
- Happy Eyeballs DNS resolution
- Graceful shutdown (Ctrl+C)
- Per-flow logging with tracing
- Config file support (YAML)
- Flow serialization (JSON + gzip)
- HAR export
- DNS wire format encode/decode
- WebSocket message types

### Architecture
- 8 crates: mitm-core, mitm-net, mitm-proxy, mitm-certs,
  mitm-addons, mitm-options, mitm-io, mitm-cli
- Spec-driven development with OpenSpec
- 150+ tests
