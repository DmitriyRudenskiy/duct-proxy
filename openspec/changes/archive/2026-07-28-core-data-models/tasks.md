## 1. Project Setup

- [x] 1.1 Add dependencies to workspace Cargo.toml: serde, serde_json, uuid, tokio, indexmap (optional, may implement Headers custom)
- [x] 1.2 Verify workspace builds with `cargo build` before any code changes
- [x] 1.3 Set up `mitm-core` crate as the foundation crate

## 2. Connection Types (mitm-core)

- [x] 2.1 Define `ConnectionState` flag enum (Closed, CanRead, CanWrite, Open)
- [x] 2.2 Define `Connection` base struct with all fields (id, peername, sockname, state, transport_protocol, error, tls, certificate_list, alpn, alpn_offers, cipher, cipher_list, tls_version, sni, timestamps)
- [x] 2.3 Implement `Connection` computed properties: `connected`, `tls_established`
- [x] 2.4 Define `Client` struct extending Connection with client-specific fields (mitmcert, proxy_mode)
- [x] 2.5 Define `Server` struct extending Connection with server-specific fields (address, tcp_setup_ts, via)
- [x] 2.6 Implement Server guard: reject address/via changes on open connection
- [x] 2.7 Add serde derives and verify serialization round-trip for Connection, Client, Server
- [x] 2.8 Write unit tests for ConnectionState bit operations

## 3. Flow Base Types (mitm-core)

- [x] 3.1 Define `FlowError` struct (msg, timestamp) with KILLED_MESSAGE constant
- [x] 3.2 Define `FlowBase` struct with all shared fields (id, client_conn, server_conn, error, intercepted, marked, is_replay, live, timestamp_created, metadata, comment)
- [x] 3.3 Define `Flow` enum: `Http(HTTPFlow) | Tcp(TCPFlow) | Udp(UDPFlow) | Dns(DNSFlow)`
- [x] 3.4 Implement `FlowType` serialization via `#[serde(tag = "type")]`
- [x] 3.5 Add serde derives on Flow enum with `#[serde(tag = "type")]`
- [x] 3.6 Write unit tests for Flow enum serialization round-trip

## 4. Flow Lifecycle (mitm-proxy)

- [x] 4.1 Implement `Flow::intercept()` — sets intercepted=true
- [x] 4.2 Implement `Flow::resume()` — sets intercepted=false
- [ ] 4.3 Implement `Flow::wait_for_resume()` — async method awaiting broadcast (deferred: needs tokio runtime)
- [x] 4.4 Implement `Flow::kill()` — sets error, clears intercepted, sets live=false
- [x] 4.5 Implement `Flow::killable` property
- [x] 4.6 Implement `Flow::copy()` — deep clone with new id, live=false
- [ ] 4.7 Add tokio::sync::broadcast for resume state (deferred: paired with 4.3)
- [x] 4.8 Write tests: intercept/resume lifecycle, kill behavior, copy independence

## 5. Headers Type (mitm-core)

- [x] 5.1 Define `Headers` struct with internal Vec<(Vec<u8>, Vec<u8>)> and HashMap<String, Vec<usize>> index
- [x] 5.2 Implement case-insensitive indexing (lowercase conversion)
- [x] 5.3 Implement `get(key)`, `get_all(key)`, `set(key, value)`, `set_all(key, values)`, `insert(index, key, value)`, `delete(key)`
- [x] 5.4 Implement `Fields` iterator preserving insertion order
- [x] 5.5 Implement `bytes(headers)` / `Display` for HTTP header block output
- [x] 5.6 Implement serde Serialize/Deserialize (serialize as list of (name, value) pairs)
- [x] 5.7 Write tests: case-insensitive lookup, order preservation, multi-value headers, Set-Cookie handling

## 6. HTTP Message Types (mitm-net)

- [x] 6.1 Define `StreamMode` enum: Buffered, Passthrough, Transform
- [x] 6.2 Define `MessageData` struct (http_version, headers, content, trailers, timestamps)
- [x] 6.3 Define `Message` struct with `data: MessageData` and `stream: StreamMode`
- [x] 6.4 Implement Message properties: `http_version`, `headers`, `trailers`, `timestamp_start`, `timestamp_end`
- [x] 6.5 Implement content management: `raw_content`, `content`, `text`, `json()`
- [x] 6.6 Implement content setters: `set_content()`, `set_text()` — auto-update content-length
- [x] 6.7 Implement `decode()` and `encode(encoding)` methods (encode is stub)
- [x] 6.8 Add serde derives on Message, MessageData

## 7. HTTP Request Type (mitm-net)

- [x] 7.1 Define Request data inline (host, port, method, scheme, authority, path + MessageData)
- [x] 7.2 Define `Request` struct
- [x] 7.3 Implement Request properties: `method` (uppercase), `scheme`, `authority`, `host`, `port`, `path`
- [x] 7.4 Implement `url` property (computed from scheme+host+port+path) with setter that decomposes
- [x] 7.5 Implement `host_header` property (HTTP/1 vs HTTP/2 aware)
- [x] 7.6 Implement `pretty_host` and `pretty_url` properties
- [x] 7.7 Implement `first_line_format` property (authority/absolute/relative)
- [x] 7.8 Implement `query` as `query_pairs()` + `set_query()` methods (simplified view)
- [ ] 7.9 Implement `cookies` as a view type over Cookie header (deferred)
- [x] 7.10 Implement `path_components` property
- [ ] 7.11 Implement `urlencoded_form` and `multipart_form` accessors (deferred)
- [x] 7.12 Implement `anticache()`, `anticomp()`, `constrain_encoding()` methods
- [x] 7.13 Implement `Request::make()` factory method
- [x] 7.14 Add serde derives and test serialization

## 8. HTTP Response Type (mitm-net)

- [x] 8.1 Define Response data inline (status_code, reason + MessageData)
- [x] 8.2 Define `Response` struct
- [x] 8.3 Implement `status_code` property (u16)
- [x] 8.4 Implement `reason` property (ISO-8859-1 decoded)
- [ ] 8.5 Implement `cookies` as response cookie view with attribute parsing (deferred)
- [x] 8.6 Implement `refresh()` method for replay date adjustment (stub)
- [x] 8.7 Implement `Response::make()` factory method
- [x] 8.8 Add serde derives and test serialization

## 9. TCP/UDP Stream Types (mitm-proxy)

- [x] 9.1 Define `TCPMessage` struct (from_client, content, timestamp)
- [x] 9.2 Define `UDPMessage` struct (from_client, content, timestamp)
- [x] 9.3 Define `TCPFlow` struct (base: FlowBase, messages: Vec<TCPMessage>)
- [x] 9.4 Define `UDPFlow` struct (base: FlowBase, messages: Vec<UDPMessage>)
- [x] 9.5 Add serde derives and test serialization
- [x] 9.6 Write tests: message direction tracking, flow message accumulation

## 10. DNS Types (mitm-proxy)

- [x] 10.1 Define `Question` struct (name, type_, class_)
- [x] 10.2 Define `ResourceRecord` struct (name, type_, class_, ttl, data)
- [x] 10.3 Implement ResourceRecord typed accessors: `ipv4_address`, `ipv6_address`, `domain_name`, `text`
- [x] 10.4 Implement ResourceRecord constructors: `A()`, `AAAA()`, `CNAME()`, `PTR()`, `TXT()`
- [x] 10.5 Define `DNSMessage` struct (id, query, op_code, flags, rcode, questions, answers, authorities, additionals, timestamp)
- [x] 10.6 Implement `DNSMessage::succeed(answers)` factory
- [x] 10.7 Implement `DNSMessage::fail(rcode)` factory
- [x] 10.8 Implement `DNSMessage::copy()` with new random ID
- [x] 10.9 Implement DNS wire format: `packed` property and `DNSMessage::unpack(bytes)` parser
- [x] 10.10 Handle domain name compression pointers in unpack
- [x] 10.11 Implement `DNSMessage::to_json()` for web UI
- [x] 10.12 Define `DNSFlow` struct (base: FlowBase, request: DNSMessage, response: Option<DNSMessage>)
- [x] 10.13 Add serde derives and test serialization

## 11. WebSocket Types (mitm-proxy)

- [x] 11.1 Define `WebSocketOpcode` enum (TEXT=1, BINARY=2, CLOSE=8, PING=9, PONG=10)
- [x] 11.2 Define `WebSocketMessage` struct (from_client, msg_type, content, timestamp, dropped, injected)
- [x] 11.3 Implement WebSocketMessage properties: `is_text`, `text`, `drop()`, `kill()`
- [x] 11.4 Define `WebSocketData` struct (messages, closed_by_client, close_code, close_reason, timestamp_end)
- [x] 11.5 Integrate WebSocketData into HTTPFlow as optional field
- [x] 11.6 Add serde derives and test serialization

## 12. Integration & Validation

- [x] 12.1 Run `cargo build` — verify all crates compile
- [x] 12.2 Run `cargo test` — verify all unit tests pass (59 tests)
- [x] 12.3 Verify Flow enum serialization round-trip for all 4 variants
- [x] 12.4 Verify Headers type handles edge cases (empty, single, multi-value, non-UTF-8)
- [x] 12.5 Verify DNSMessage wire format with known test vectors
- [x] 12.6 Run clippy: `cargo clippy -- -D warnings` (passes clean)
- [ ] 12.7 Document public API with doc comments on all public types and methods (deferred)
