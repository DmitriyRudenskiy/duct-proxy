# Architecture

## Data Flow Diagram

```
Client
  ↓ HTTP/CONNECT/TLS
ProxyServer::accept_loop()
  ↓
handle_connection()
  ↓ detect_protocol()
  ├─ HTTP → HttpForwarder::forward() → Upstream
  ├─ CONNECT → 200 OK → intercept_tls() → forward_bidirectional()
  └─ TLS → intercept_tls() → forward_bidirectional()
```

## Crate Descriptions

### mitm-core
Базовые типы данных: Flow, HTTPFlow, Connection, Headers.
Сердце системы — все остальные crate'ы используют эти типы.

### mitm-net
HTTP/1.1 парсер, URL parser, Cookie parser, Form parser,
Chunked transfer-encoding, HTTP/2 preface detection.

### mitm-proxy
Proxy server с TCP listener, protocol detection, TLS interception,
HTTP forwarding, hook system, connection pool.

### mitm-certs
CA certificate management (ECDSA P-256), CertStore с LRU eviction,
Leaf certificate generation, SNI extraction.

### mitm-addons
Addon system: Addon trait с lifecycle hooks, AddonManager для sequential dispatch,
Built-in addons: ModifyHeaders, ModifyBody, Block, Filter.

### mitm-options
CLI args с clap derive, config file loading (YAML),
OptManager для runtime get/set с валидацией.

### mitm-io
Flow serialization (JSON), gzip-compressed JSONL dump format,
FlowWriter/FlowReader, HAR 1.2 export.

### mitm-cli
Binary entry point с async main, CLI args, config loading,
CA init, addon registration, proxy server startup, graceful shutdown.

## MITM Process (Step by Step)

1. Client connects to proxy (TCP)
2. Proxy peeks first bytes to detect protocol
3. If CONNECT:
   - Parse target host:port
   - Send "200 Connection Established"
   - Extract SNI from client TLS hello
   - Generate leaf certificate from CA
   - Complete TLS handshake with client
   - Connect to upstream (TCP + TLS)
   - Bidirectional forwarding
4. If plain HTTP:
   - Parse request line
   - Connect to upstream
   - Forward request/response

## Adding a New Addon

1. Implement the `Addon` trait:
```rust
use mitm_addons::Addon;

pub struct MyAddon {
    // fields
}

impl Addon for MyAddon {
    fn name(&self) -> &str { "my-addon" }
    
    async fn request(&self, flow: &mut HTTPFlow) -> Result<(), AddonError> {
        // Modify request
        Ok(())
    }
    
    async fn response(&self, flow: &mut HTTPFlow) -> Result<(), AddonError> {
        // Modify response
        Ok(())
    }
}
```

2. Register in main.rs:
```rust
let addon_mgr = AddonManager::new();
addon_mgr.add(MyAddon::new());
```

## Adding a New Protocol Layer

1. Add variant to `Protocol` enum in `handler.rs`
2. Add detection logic in `detect_protocol_from_bytes()`
3. Add handling in `handle_connection()` match
4. Implement protocol-specific forwarding logic
