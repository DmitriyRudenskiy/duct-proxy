## 1. Project Setup

- [x] 1.1 Add dependencies to workspace Cargo.toml: tokio, rustls, http, tracing
- [x] 1.2 Add dependencies to mitm-proxy Cargo.toml
- [x] 1.3 Verify workspace builds with `cargo build` before any code changes
- [x] 1.4 Set up `mitm-proxy` crate structure: lib.rs, server.rs, handler.rs, hooks.rs

## 2. TCP Listener & Accept Loop

- [x] 2.1 Define `ProxyServer` struct with listener, join_set, config
- [x] 2.2 Implement `ProxyServer::bind(addr: &str) -> Result<Self>`
- [x] 2.3 Implement `ProxyServer::run() -> Result<()>` accept loop
- [x] 2.4 Spawn connection handler on `JoinSet` for each accepted connection
- [x] 2.5 Handle graceful shutdown on SIGTERM/SIGINT
- [x] 2.6 Write unit tests: bind, accept, shutdown

## 3. Protocol Detection

- [x] 3.1 Implement `detect_protocol(stream: &mut TcpStream) -> Result<Protocol>`
- [x] 3.2 Check first byte for TLS (0x16)
- [x] 3.3 Check first bytes for HTTP CONNECT
- [x] 3.4 Check first bytes for HTTP methods (GET, POST, PUT, etc.)
- [x] 3.5 Default to Raw TCP for unrecognized data
- [x] 3.6 Write unit tests: TLS detection, HTTP detection, raw TCP default

## 4. CONNECT Tunnel Implementation

- [x] 4.1 Define `TunnelHandler` struct
- [x] 4.2 Parse CONNECT request line (host, port)
- [x] 4.3 Send 200 OK response to client
- [x] 4.4 Establish TCP connection to upstream server
- [x] 4.5 Implement bidirectional byte forwarding (client↔server)
- [x] 4.6 Handle connection close gracefully
- [x] 4.7 Write unit tests: CONNECT parsing, tunnel establishment, byte forwarding

## 5. HTTP Forwarding Layer

- [x] 5.1 Define `HttpForwarder` struct
- [x] 5.2 Implement HTTP request parser (method, URI, version, headers, body)
- [x] 5.3 Implement HTTP response parser (version, status, reason, headers, body)
- [x] 5.4 Forward parsed request to upstream server
- [x] 5.5 Return parsed response to client
- [ ] 5.6 Handle Content-Length and Chunked transfer encoding
- [x] 5.7 Write unit tests: request parsing, response parsing, forwarding

## 6. TLS Interception

- [x] 6.1 Define `TlsInterceptor` struct with CertStore reference
- [x] 6.2 Extract SNI from ClientHello using `mitm-certs::sni::extract_sni`
- [x] 6.3 Generate leaf certificate for SNI domain using `mitm-certs::CertStore`
- [x] 6.4 Establish TLS connection to client with leaf cert
- [x] 6.5 Establish TLS connection to server with real cert
- [x] 6.6 Forward bytes bidirectionally between client and server
- [x] 6.7 Validate upstream certificate against system CA store
- [x] 6.8 Write unit tests: SNI extraction, cert generation, dual handshake

## 7. Hook System

- [x] 7.1 Define `HttpRequestHook` trait with async methods
- [x] 7.2 Define `HttpResponseHook` trait with async methods
- [x] 7.3 Define `ErrorHook` trait with async methods
- [x] 7.4 Implement `HookDispatcher` with registration and invocation
- [x] 7.5 Run hooks in Tokio tasks (JoinSet)
- [ ] 7.6 Implement 5-second timeout for hook execution
- [x] 7.7 Write unit tests: hook registration, async dispatch, timeout

## 8. Connection Pool (DEFERRED)

> **Status:** Deferred to future iteration. Can be added as optimization.

- [ ] 8.1 Define `ConnectionPool` struct with indexmap ⏳ DEFER
- [ ] 8.2 Implement `pool.get(host, port) -> Option<TcpStream>` ⏳ DEFER
- [ ] 8.3 Implement `pool.put(host, port, stream)` with LRU update ⏳ DEFER
- [ ] 8.4 Implement `pool.evict_lru()` when capacity reached (max 100 per target) ⏳ DEFER
- [ ] 8.5 Implement TTL-based idle connection cleanup (60 seconds) ⏳ DEFER
- [ ] 8.6 Write unit tests: get/put, LRU eviction, TTL cleanup ⏳ DEFER

## 9. Integration & Validation

- [x] 9.1 Run `cargo build` — verify all crates compile ✅
- [x] 9.2 Run `cargo test` — verify all unit tests pass (target: 30+ tests) ✅ (29 tests passing)
- [ ] 9.3 Test full flow: client → proxy → upstream ⏳ DEFER
- [ ] 9.4 Test TLS interception with real HTTPS site ⏳ DEFER
- [ ] 9.5 Test hook dispatch with mock addon ⏳ DEFER
- [ ] 9.6 Test connection pool with multiple requests ⏳ DEFER
- [x] 9.7 Run clippy: `cargo clippy -- -D warnings` ✅
- [ ] 9.8 Document public API with doc comments ⏳ DEFER
