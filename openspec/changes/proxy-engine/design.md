## Context

mitmproxy-rs is a Rust reimplementation of mitmproxy with a focus on TLS interception for HTTP debugging. The `mitm-proxy` crate exists as a scaffold but has no implementation. Core infrastructure (`mitm-core`, `mitm-certs`, `mitm-io`) is complete with 90+ tests.

Current state:
- `mitm-core`: Flow, Connection, HTTP types implemented
- `mitm-certs`: CA generation, CertStore, SNI extraction, leaf certs (ECDSA P-256)
- `mitm-io`: Serialization/deserialization for flows
- `mitm-proxy`: Empty scaffold, needs full implementation

## Goals / Non-Goals

**Goals:**
- TCP listener with async accept loop using Tokio
- Protocol detection: distinguish HTTP CONNECT, TLS ClientHello, raw TCP
- Explicit proxy mode: CONNECT tunnel with absolute-URI format
- TLS interception: dual handshake (client + server) with SNI from `mitm-certs`
- HTTP/1.1 request/response parsing and forwarding
- Hook dispatch for addon integration (requestheaders, request, response, error)
- Connection pool for upstream connections

**Non-Goals:**
- Transparent proxy mode (v2)
- WebSocket upgrade (deferred)
- HTTP/2 or HTTP/3 support (deferred)
- Addon API implementation (hooks are dispatch points only)
- Configuration file parsing (v1 uses CLI flags)

## Decisions

### Decision 1: Tokio for async runtime

**Choice:** Use Tokio for the entire proxy loop.

**Rationale:**
- Industry standard for Rust async
- Excellent TCP listener support with `TcpListener::from_std`
- `JoinSet` for connection task management
- Graceful shutdown via `tokio::signal`
- Already in workspace dependencies

**Alternatives:**
- *async-std*: Simpler API but less performant for network-heavy workloads
- *smol*: Too minimal for production proxy

### Decision 2: Protocol detection via first-byte peek

**Choice:** Peek at first 1-2 bytes to determine protocol.

**Implementation:**
- If first byte is 0x16 (TLS Handshake) → TLS ClientHello
- If first bytes match HTTP method (GET/POST/PUT/DELETE/PATCH/HEAD/OPTIONS) → HTTP
- Otherwise → raw TCP

**Rationale:**
- Zero-copy peek (1-2 bytes)
- Deterministic classification
- Matches mitmproxy Python approach

**Alternatives:**
- *ALPN inspection*: Only works after TLS handshake, too late for CONNECT
- *Port-based*: Fragile, doesn't work for non-standard ports

### Decision 3: Dual TLS handshake architecture

**Choice:** Create two separate TLS connections (client↔mitmproxy, mitmproxy↔server).

**Implementation:**
```
Client ←—— TLS (mitmcert signed by CA) ——→ mitmproxy ←—— TLS (real server cert) ——→ Server
```

**Rationale:**
- Full visibility into encrypted traffic
- Leaf cert generated per-domain using SNI from ClientHello
- Server cert validated against real CA (for upstream)
- Matches mitmproxy architecture

**Alternatives:**
- *Single TLS termination*: Can't inspect upstream traffic
- *TLS passthrough*: No inspection capability

### Decision 4: Hook system as async closures

**Choice:** Define hook traits with async methods, dispatch via `JoinSet`.

**Implementation:**
```rust
pub trait HttpRequestHook {
    async fn requestheaders(&self, flow: &mut HttpFlow) -> Result<()>;
    async fn request(&self, flow: &mut HttpFlow) -> Result<()>;
}
```

**Rationale:**
- Type-safe hook dispatch
- Async for I/O-heavy addon operations
- Trait-based for easy extension

**Alternatives:**
- *Callback-based sync*: Blocks proxy loop for slow addons
- *Event bus*: Overkill for v1, adds complexity

### Decision 5: Connection pool with LRU eviction

**Choice:** Use `linked-hash-map` or `indexmap` for LRU pool.

**Implementation:**
- Max 100 upstream connections per target
- Keys: `(host, port)` tuples
- Values: `(TcpStream, last_used)` pairs
- Evict least recently used when pool is full

**Rationale:**
- Reuse TCP connections reduces latency
- LRU prevents memory leaks from unused connections
- Simple implementation with indexmap

**Alternatives:**
- *No pooling*: Every request opens new TCP connection (slow)
- *Fixed pool*: Wastes resources for low-traffic targets

### Decision 6: Flow representation as enum

**Choice:** `Flow::Http(HttpFlow) | Tcp(TcpFlow) | Udp(UdpFlow)` enum in mitm-proxy.

**Rationale:**
- Type-safe flow handling
- Pattern matching for protocol-specific logic
- Consistent with mitm-core design

**Alternatives:**
- *Trait-based*: More flexible but harder to serialize
- *HashMap<String, Flow>*: Loses type safety

## Risks / Trade-offs

[Risk: TLS handshake performance] → [Mitigation: Use rustls with session caching. Pre-warm cert store.]

[Risk: Memory leaks from unclosed connections] → [Mitigation: Drop connections on flow close. Use RAII. Set TCP keepalive.]

[Risk: Hook blocking proxy loop] → [Mitigation: Run hooks in JoinSet. Timeout after 5s. Log warnings.]

[Risk: CONNECT tunnel hijacking] → [Mitigation: Validate absolute-URI format. Reject malformed requests. Log all tunnels.]

[Risk: CertStore capacity exhaustion] → [Mitigation: Default 1024 entries. LRU eviction. Log when evicting.]

## Migration Plan

1. **Phase 1 (this change):** Implement core proxy loop with TCP listener, protocol detection, CONNECT tunneling
2. **Phase 2:** Add TLS interception with dual handshake
3. **Phase 3:** Implement HTTP/1.1 forwarding layer
4. **Phase 4:** Add hook dispatch for addon integration
5. **Phase 5:** Connection pool optimization

Rollback: delete `mitm-proxy` crate and revert `mitm-core` Connection additions.

## Open Questions

1. **Should we support multiple listeners?** (e.g., HTTP on 8080, HTTPS on 8443) → Defer to v2
2. **What about proxy authentication?** → Defer to v2 (v1 is single-user)
3. **Should hooks be synchronous or asynchronous?** → Async (allows I/O in addons)
4. **Connection pool size per target or global?** → Per-target (100 max) to prevent single-host exhaustion
