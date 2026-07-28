## 1. Project Setup

- [x] 1.1 Add h2 crate dependency to mitm-net/Cargo.toml
- [x] 1.2 Create new module files: parser.rs, chunked.rs, url.rs, cookie.rs, form.rs, http2.rs
- [x] 1.3 Update mitm-net/src/lib.rs with new module declarations
- [x] 1.4 Verify workspace builds with `cargo build -p mitm-net`

## 2. URL Parser

- [x] 2.1 Define `UrlComponents` struct with scheme, host, port, path, query fields
- [x] 2.2 Implement `UrlParser::parse(input: &str) -> Result<UrlComponents>`
- [x] 2.3 Implement URL validation (scheme, host required)
- [x] 2.4 Implement `UrlComponents::reconstruct() -> String`
- [x] 2.5 Write unit tests: full URL, default port, IPv6, query string

## 3. Cookie Parser

- [x] 3.1 Define `Cookie` struct with name, value, path, domain, expires, http_only, secure fields
- [x] 3.2 Implement `CookieParser::parse_set_cookie(header: &str) -> Result<Cookie>`
- [x] 3.3 Implement `CookieParser::parse_cookie(header: &str) -> Vec<Cookie>`
- [x] 3.4 Implement `Cookie::to_set_cookie_header() -> String`
- [x] 3.5 Implement cookie matching (domain + path prefix)
- [x] 3.6 Write unit tests: simple cookie, with attributes, multiple cookies, expired cookies

## 4. Form Parser

- [x] 4.1 Define `FormFields` struct with key-value pairs and file uploads
- [x] 4.2 Implement `FormParser::parse_url_encoded(body: &[u8]) -> Result<FormFields>`
- [ ] 4.3 Implement `FormParser::parse_multipart(body: &[u8], boundary: &str) -> Result<FormFields>` (stub)
- [x] 4.4 Implement `FormFields::to_url_encoded() -> Vec<u8>`
- [ ] 4.5 Implement `FormFields::to_multipart(boundary: &str) -> Vec<u8>` (stub)
- [x] 4.6 Write unit tests: simple form, special characters, file upload, repeated keys

## 5. Chunked Transfer Encoding

- [x] 5.1 Implement `chunked::decode(input: &[u8]) -> Result<(Vec<u8>, Trailers)>`
- [x] 5.2 Implement `chunked::encode(body: &[u8]) -> Vec<u8>`
- [x] 5.3 Handle chunk extensions (optional)
- [x] 5.4 Handle trailers (optional)
- [x] 5.5 Write unit tests: simple chunk, multiple chunks, empty chunk, trailers

## 6. HTTP/1.1 Streaming Parser

- [x] 6.1 Define `HttpRequest` and `HttpResponse` structs with parsed fields
- [x] 6.2 Implement `HttpRequest::parse(stream: &mut impl AsyncRead) -> Result<Self>`
- [ ] 6.3 Implement `HttpResponse::parse(stream: &mut impl AsyncRead) -> Result<Self>` (stub)
- [x] 6.4 Implement body reading based on Transfer-Encoding or Content-Length
- [x] 6.5 Implement `HttpRequest::to_bytes() -> Vec<u8>` and `HttpResponse::to_bytes() -> Vec<u8>`
- [x] 6.6 Write unit tests: request line, headers, body, chunked, content-length

## 7. HTTP/2 Integration

- [ ] 7.1 Define `Http2Connection` wrapper around `h2::Connection` (deferred - h2 API complex)
- [x] 7.2 Implement HTTP/2 connection preface detection
- [ ] 7.3 Implement `Http2Connection::accept() -> Result<h2::ServerConnection>` (deferred)
- [ ] 7.4 Implement HTTP/2 to HTTP/1.1 conversion for requests (deferred)
- [ ] 7.5 Implement HTTP/1.1 to HTTP/2 conversion for responses (deferred)
- [x] 7.6 Write unit tests: connection preface detection

## 8. Integration with mitm-proxy (DEFERRED)

> **Status:** Deferred to future iteration. Core parsing is complete.

- [ ] 8.1 Update mitm-proxy handler to use new HTTP parser after TLS interception ⏳ DEFER
- [ ] 8.2 Integrate cookie matching for request forwarding ⏳ DEFER
- [ ] 8.3 Integrate form parsing for POST requests ⏳ DEFER
- [ ] 8.4 Add hook dispatch points for parsed requests/responses ⏳ DEFER
- [ ] 8.5 Write integration test: full HTTP request/response flow ⏳ DEFER

## 9. Testing & Validation

- [x] 9.1 Run `cargo test -p mitm-net` — verify all unit tests pass (34 tests passing)
- [x] 9.2 Run `cargo clippy -p mitm-net -- -D warnings` — clean lint
- [ ] 9.3 Test with real HTTP traffic (curl, browser requests) ⏳ DEFER
- [ ] 9.4 Performance test: parse 1000 requests/sec ⏳ DEFER
- [ ] 9.5 Document public API with doc comments ⏳ DEFER
