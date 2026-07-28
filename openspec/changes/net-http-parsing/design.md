## Context

mitmproxy-rs currently has basic HTTP types in mitm-net but lacks production-grade parsing capabilities. The proxy engine (mitm-proxy) needs to parse HTTP traffic after TLS interception to dispatch hooks and forward requests. Current implementation has stub types but no actual parsing logic for request/response lines, headers, or body handling.

The existing mitm-net crate provides:
- Basic `Request` and `Response` structs (from http crate)
- `Message` and `MessageData` enums for abstraction
- `StreamMode` for TCP/UDP distinction
- Query helper functions

Missing capabilities:
- Streaming parser for request/response from AsyncRead
- Chunked transfer-encoding decode/encode
- Content-Length body reading
- HTTP/2 support via h2 crate
- URL component parsing
- Cookie parsing (RFC 6265)
- Form data parsing (URL-encoded and multipart)

## Goals / Non-Goals

**Goals:**
- Implement streaming HTTP/1.1 parser that works with tokio AsyncRead
- Support Chunked and Content-Length transfer encodings
- Parse and generate cookies per RFC 6265
- Parse URL components (scheme, host, port, path, query)
- Parse form data (application/x-www-form-urlencoded and multipart/form-data)
- Integrate h2 crate for HTTP/2 connection management
- Convert HTTP/2 to HTTP/1.1 for upstream forwarding

**Non-Goals:**
- HTTP/3 (QUIC) support - deferred to future change
- HTTP/1.0 support beyond basic compatibility
- WebSocket upgrade handling - handled in mitm-proxy
- HTTP/2 push promises - not commonly used
- Cookie session storage - handled by addons

## Decisions

### Decision 1: Use bytes::BytesMut for parsing buffers

**Choice:** Use `bytes::BytesMut` for incremental parsing buffers instead of `Vec<u8>`.

**Rationale:**
- `BytesMut` provides zero-copy slicing and efficient growth
- Compatible with tokio's async I/O ecosystem
- Already a workspace dependency
- Avoids unnecessary copying for large bodies

**Alternatives considered:**
- `Vec<u8>` - simpler but requires copying on slice operations
- `io::Cursor<Vec<u8>>` - less efficient for incremental parsing

### Decision 2: Streaming parser with state machine

**Choice:** Implement HTTP parser as async state machine with explicit states.

**Rationale:**
- Handles partial reads correctly (common in network I/O)
- Memory efficient - doesn't require loading entire request/response
- Clear state transitions make debugging easier
- Compatible with tokio's async/await pattern

**States:**
1. `RequestLine` - parsing "METHOD URI HTTP/version"
2. `Headers` - parsing header lines until empty line
3. `Body` - reading body based on Transfer-Encoding or Content-Length
4. `Done` - request/response complete

**Alternatives considered:**
- Buffer entire message then parse - simpler but memory intensive
- Use regex for parsing - less efficient and harder to handle streaming

### Decision 3: Use h2 crate for HTTP/2

**Choice:** Use the `h2` crate (v0.4) for HTTP/2 connection management.

**Rationale:**
- Industry-standard HTTP/2 implementation in Rust
- Already in workspace dependencies
- Handles connection preface, frame parsing, flow control
- Actix-provided h2 is well-tested and maintained

**Alternatives considered:**
- Implement HTTP/2 from scratch - too complex and error-prone
- Use quinn for HTTP/3 - out of scope for this change

### Decision 4: Separate modules for parsing concerns

**Choice:** Organize mitm-net/src/ with separate modules:
- `http.rs` - HTTP message types and serialization
- `parser.rs` - Streaming HTTP/1.1 parser
- `chunked.rs` - Chunked transfer-encoding
- `url.rs` - URL parsing
- `cookie.rs` - Cookie parsing
- `form.rs` - Form data parsing
- `http2.rs` - HTTP/2 integration

**Rationale:**
- Single responsibility - each module has clear purpose
- Easier to test individual components
- Reduces compile times (parallel compilation)
- Follows Rust ecosystem conventions

**Alternatives considered:**
- Single large http.rs module - harder to navigate and test
- Separate crates for each parser - over-engineered for this scope

### Decision 5: Error type with thiserror

**Choice:** Use `thiserror` for parser error types.

**Rationale:**
- Clean error messages with context
- Automatic Display implementation
- Already used in mitm-certs and other crates
- Compatible with anyhow for ergonomic error propagation

## Risks / Trade-offs

### Risk: Streaming parser complexity
**Mitigation:** Start with Content-Length parser, add Chunked later. Use comprehensive tests for edge cases.

### Risk: HTTP/2 conversion overhead
**Mitigation:** Minimize allocations during HTTP/2→HTTP/1.1 conversion. Use Bytes for header values.

### Risk: Cookie parsing edge cases
**Mitigation:** Follow RFC 6265 strictly. Test with real-world cookie strings from popular sites.

### Risk: Multipart parsing performance
**Mitigation:** Use streaming multipart parser that doesn't load entire file into memory.

### Risk: URL parsing for invalid inputs
**Mitigation:** Use established URL parsing algorithms. Return clear errors for malformed URLs.

## Migration Plan

This change is additive - no breaking changes to existing mitm-net API.

**Steps:**
1. Add new modules to mitm-net (parser, chunked, url, cookie, form, http2)
2. Extend existing http.rs with new types and methods
3. Update mitm-net/Cargo.toml with new dependencies (h2)
4. Add comprehensive tests for each module
5. Integrate with mitm-proxy handler (separate change)
6. Update documentation

**Rollback:** Revert mitm-net changes if parsing bugs found in production.

## Open Questions

1. Should we support HTTP/1.0 backward compatibility for Content-Length reading?
2. Do we need to handle malformed headers gracefully or fail fast?
3. Should cookie parsing be case-insensitive for attribute names?
4. What's the maximum body size we should support before rejecting?
