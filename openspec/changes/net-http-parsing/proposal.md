## Why

mitmproxy-rs requires robust HTTP/1.1 and HTTP/2 parsing capabilities to handle real-world web traffic. The existing mitm-net crate has basic HTTP types but lacks production-grade parsing for request/response lines, headers, body handling (Chunked, Content-Length), URL parsing, Cookie handling, and Form data parsing. Without these capabilities, the proxy cannot intercept and modify HTTP traffic effectively.

## What Changes

- **HTTP/1.1 Request Parsing**: Parse request line (method, URI, version), headers, and body from AsyncRead streams
- **HTTP/1.1 Response Parsing**: Parse response line (version, status, reason), headers, and body
- **Chunked Transfer-Encoding**: Decode and encode chunked bodies
- **Content-Length Body Reading**: Read exact byte counts for body parsing
- **HTTP/2 Support**: Connection preface, frame parsing via h2 crate
- **URL Parsing**: Extract scheme, host, port, path, query parameters
- **Cookie Parsing**: Parse Set-Cookie and Cookie headers (RFC 6265)
- **Form Parsing**: Parse application/x-www-form-urlencoded and multipart/form-data

## Capabilities

### New Capabilities

- `http-parser`: HTTP/1.1 request/response parsing with streaming support
- `http2-support`: HTTP/2 connection and frame handling via h2 crate
- `url-parser`: URL component extraction and validation
- `cookie-parser`: Cookie header parsing and generation
- `form-parser`: Form data parsing (URL-encoded and multipart)
- `transfer-encoding`: Chunked and Content-Length body handling

### Modified Capabilities

<!-- No existing capabilities need spec-level changes -->

## Impact

**Affected Code:**
- `crates/mitm-net/src/http.rs` - Major extension of existing HTTP types
- `crates/mitm-net/src/lib.rs` - New module exports
- `crates/mitm-proxy/src/handler.rs` - Integration with HTTP parser after TLS

**Dependencies:**
- `http` crate v1 - HTTP message types (already in workspace)
- `h2` crate v0.4 - HTTP/2 implementation (already in workspace)
- `bytes` crate v1 - Efficient byte buffers (already in workspace)

**Breaking Changes:** None - all additions are backward compatible with existing mitm-net types.
