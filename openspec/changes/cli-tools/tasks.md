## 1. Project Setup

- [x] 1.1 Update `crates/mitm-cli/Cargo.toml` with dependencies: clap, tracing-subscriber, and all workspace crates
- [x] 1.2 Verify workspace builds with `cargo build -p mitm-cli`

## 2. Main.rs Structure

- [x] 2.1 Create `crates/mitm-cli/src/main.rs` with async main function
- [x] 2.2 Define CLI args struct with clap derive: --host, --port, --mode, --config, --log-level, --version
- [x] 2.3 Implement startup output: version, listening address, CA path, registered addons
- [x] 2.4 Initialize tracing-subscriber for logging

## 3. Configuration Loading

- [x] 3.1 Load config.yaml from ~/.mitmproxy/ if present
- [x] 3.2 Merge CLI args with config file (CLI takes precedence)
- [ ] 3.3 Parse --set key=value pairs for inline addon configuration (deferred)

## 4. CA and Certificate Initialization

- [x] 4.1 Initialize CaRoot (load or generate)
- [ ] 4.2 Initialize CertStore with LRU eviction (deferred)
- [x] 4.3 Generate and save CA certificate to ~/.mitmproxy/ (fixed filename: mitmproxy-ca-cert.pem)
- [x] 4.4 Log CA certificate path on startup

## 5. Addon Registration

- [x] 5.1 Create AddonManager instance
- [x] 5.2 Register ModifyHeaders addon (from mitm-addons)
- [x] 5.3 Register Block addon (from mitm-addons)
- [ ] 5.4 Apply --set configurations to addons (deferred)
- [x] 5.5 Log registered addons on startup

## 6. Proxy Server Initialization

- [x] 6.1 Create ProxyServer::from_options(&options)
- [x] 6.2 Bind to listen address and port
- [x] 6.3 Log listening address on startup

## 7. Graceful Shutdown

- [x] 7.1 Implement Ctrl+C handler with tokio::signal::ctrl_c() (built into ProxyServer::run())
- [x] 7.2 Stop accept loop on shutdown signal (built into ProxyServer::run())
- [x] 7.3 Wait for existing connections to drain (built into ProxyServer::run())
- [x] 7.4 Clean exit with code 0 (built into ProxyServer::run())

## 8. Per-Flow Logging

- [x] 8.1 Implement flow logger function with timestamp (tracing::info!)
- [x] 8.2 Format log line: METHOD url → STATUS
- [x] 8.3 Log CONNECT tunnels: CONNECT host:port → TLS intercepted
- [x] 8.4 Integrate logger with proxy event hooks (in handler.rs)

## 9. Real HTTP Forwarding

- [x] 9.1 Implement real HTTP forwarding in HttpForwarder::forward()
- [x] 9.2 Parse request to extract host and path
- [x] 9.3 Connect to upstream server
- [x] 9.4 Rewrite request (absolute URL → relative path)
- [x] 9.5 Read response from upstream and write to client
- [x] 9.6 Add debug logging for each step

## 9. Dump Mode Integration

- [ ] 9.1 Create FlowWriter when --dump is specified
- [ ] 9.2 Register flow writer with proxy to receive completed flows
- [ ] 9.3 Write flows to JSONL file as they complete
- [ ] 9.4 Close file writer on shutdown

## 10. Integration Testing

- [ ] 10.1 Test binary starts and listens on specified port
- [ ] 10.2 Test curl through proxy receives response
- [ ] 10.3 Test --dump mode saves flows to file
- [ ] 10.4 Test --set configures addons correctly
- [ ] 10.5 Test Ctrl+C shuts down gracefully

## 11. Testing & Validation

- [ ] 11.1 Run `cargo test --workspace` — verify no regressions
- [ ] 11.2 Run `cargo clippy --workspace -- -D warnings` — clean lint
- [ ] 11.3 End-to-end test: run proxy, curl through it, verify flow log
- [ ] 11.4 Document public API with doc comments
