## Why

mitmproxy-rs is a Rust rewrite of the popular mitmproxy tool. The foundation of any proxy is its data model — the types that represent flows, connections, HTTP messages, DNS queries, and WebSocket sessions. Without a well-designed, idiomatic Rust data model, every subsequent layer (parsing, interception, addon system, serialization) will be built on shaky ground. This change establishes the core type system before any parsing or proxy logic is implemented.

## What Changes

- Define the complete flow type hierarchy in Rust: `Flow` enum with `HTTPFlow`, `TCPFlow`, `UDPFlow`, `DNSFlow` variants
- Define the connection type system: `Client` and `Server` structs with TLS metadata, state tracking, and address information
- Define HTTP message types: `Headers` (case-insensitive MultiDict), `Message` base, `Request`, `Response` with content management (raw/content/text/json)
- Define TCP/UDP stream message types: `TCPMessage`, `UDPMessage` with direction and content
- Define DNS types: `DNSMessage`, `Question`, `ResourceRecord` with wire-format encode/decode
- Define WebSocket types: `WebSocketMessage`, `WebSocketData` attached to HTTP flows
- Define the flow error and state model: `FlowError`, intercept/resume, replay tracking
- Establish serialization strategy (serde) matching mitmproxy's state format

## Capabilities

### New Capabilities

- `flow-model`: Core flow hierarchy — `Flow` enum, `FlowBase` shared state, intercept/resume lifecycle, error handling
- `connection-model`: Connection types — `Client`, `Server`, `ConnectionState` flag enum, TLS metadata, address tracking
- `http-model`: HTTP message types — `Headers` (order-preserving MultiDict), `Message`, `Request`, `Response`, content management (raw/content/text/json), query/cookie form accessors
- `stream-model`: TCP/UDP stream messages — `TCPMessage`, `UDPMessage` with from_client flag, content bytes, timestamps
- `dns-model`: DNS protocol types — `DNSMessage` with wire encode/decode, `Question`, `ResourceRecord` with typed accessors (A, AAAA, CNAME, TXT, HTTPS)
- `websocket-model`: WebSocket session types — `WebSocketMessage` with opcode/content, `WebSocketData` container with close tracking

### Modified Capabilities

- *(none — this is a fresh Rust codebase with no existing specs)*

## Impact

- **crates/mitm-core**: Will contain `flow.rs`, `connection.rs` — the base types every other crate depends on
- **crates/mitm-net**: Will contain `http.rs` — HTTP message types consumed by proxy layer
- **crates/mitm-proxy**: Will consume all model types — HTTPFlow, TCPFlow, UDPFlow, DNSFlow, WebSocketData
- **crates/mitm-io**: Will use serde serialization of all model types for .flow file format
- **crates/mitm-web**: Will use JSON representations for the web UI
- **External dependencies**: `serde`, `indexmap` (or custom for Headers), `uuid`, `tokio::sync` (for resume state)
