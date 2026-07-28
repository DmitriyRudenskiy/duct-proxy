## Why

mitmproxy-rs needs a production-ready proxy engine that can handle TLS interception, HTTP/1.1 forwarding, and extensible hook points. The current `mitm-proxy` crate is a scaffold with no implementation. This change implements the core proxy loop with protocol detection, CONNECT tunneling, TLS interception using `mitm-certs`, and hook dispatch for addon integration.

## What Changes

- Implement TCP listener with Tokio `TcpListener` and accept loop
- Protocol detection: distinguish HTTP CONNECT, TLS ClientHello, and raw TCP
- Explicit proxy mode: CONNECT tunnel establishment with absolute-URI format
- TLS interception: dual handshake (client-side + server-side) with SNI extraction from `mitm-certs`
- HTTP/1.1 layer: request/response forwarding with headers parsing
- Hook dispatch points: `requestheaders`, `request`, `response`, `error` for addon integration
- Connection pool for upstream connections to reduce latency

## Capabilities

### New Capabilities

- `proxy-engine`: Core proxy loop with TCP listener, accept loop, and connection handling
- `protocol-detection`: Distinguish between HTTP CONNECT, TLS ClientHello, and raw TCP streams
- `tls-interception`: Dual TLS handshake with SNI-based leaf cert generation from `mitm-certs`
- `http-forwarding`: HTTP/1.1 request/response parsing and forwarding
- `hook-system`: Extension points for addons (requestheaders, request, response, error)
- `connection-pool`: Upstream connection pooling for performance

### Modified Capabilities

- `connection-model`: Extend with proxy-specific fields (proxy_mode, via, mitmcert)
- `flow-model`: Add proxy flow lifecycle (tunnel establishment, interception state)
- `stream-model`: Add bidirectional stream with proxy transform capability

## Impact

**Affected crates:**
- `mitm-proxy`: Primary implementation target (currently empty scaffold)
- `mitm-core`: Extend Connection, Client, Server with proxy fields
- `mitm-certs`: Integration with existing CA, CertStore, LeafCert
- `mitm-io`: Add serialization for proxy-specific flow data

**Dependencies:**
- `tokio`: TCP listener, async accept loop, channels
- `rustls`: TLS server/client handshakes
- `http` crate: HTTP message parsing
- `tracing`: Structured logging for proxy operations

**Breaking changes:** None for v1 — `mitm-proxy` is currently empty.
