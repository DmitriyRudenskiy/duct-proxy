## Context

mitmproxy-rs has all the necessary crates implemented but no working binary. The `mitm-cli` crate is a placeholder with no implementation. Users need a complete CLI tool that:
- Parses command-line arguments with clap
- Loads configuration from config.yaml if present
- Initializes CA certificate (load existing or generate new)
- Sets up certificate store
- Registers built-in addons (ModifyHeaders, Block)
- Starts the proxy server
- Handles graceful shutdown on Ctrl+C
- Logs per-flow information

## Goals / Non-Goals

**Goals:**
- Create working `mitm-cli` binary with complete main.rs
- Support CLI arguments: --host, --port, --mode, --dump, --set, --config
- Initialize CA and CertStore from mitm-certs
- Register built-in addons from mitm-addons
- Start ProxyServer with graceful shutdown
- Log per-flow information with timing and size
- Support dump mode for saving flows to JSONL

**Non-Goals:**
- Web interface (mitm-web crate is separate)
- Addon scripting language (future enhancement)
- Configuration file schema validation (future enhancement)
- Multi-process/threading for performance (out of scope for v1)

## Decisions

### Decision 1: Use clap for CLI parsing

**Choice:** Use `clap` v4 with derive macros.

**Rationale:**
- Industry standard for Rust CLI
- Type-safe argument parsing
- Automatic help generation
- Compatible with existing mitm-options crate

**Alternatives considered:**
- Manual argument parsing - more error-prone, verbose
- Structopt (deprecated, superseded by clap)

### Decision 2: tracing for logging

**Choice:** Use `tracing` with `tracing-subscriber` for structured logging.

**Rationale:**
- Industry standard for Rust logging
- Structured output (JSON or human-readable)
- Performance optimized
- Compatible with existing codebase

**Alternatives considered:**
- `log` + `env_logger` - less feature-rich
- Custom logging - unnecessary complexity

### Decision 3: Async runtime with tokio

**Choice:** Use `tokio` async runtime (already used in other crates).

**Rationale:**
- Consistent with existing codebase
- Production-ready async runtime
- Good ecosystem support
- Required by mitm-proxy

**Alternatives considered:**
- `async-std` - different ecosystem
- Synchronous main - simpler but less performant

### Decision 4: Single-threaded proxy for v1

**Choice:** Run proxy server on single thread with tokio runtime.

**Rationale:**
- Simpler implementation for v1
- tokio handles connection concurrency via async
- Can add multi-threading later if needed
- Matches Python mitmproxy behavior

**Alternatives considered:**
- Multi-threaded with `tokio::runtime::Builder` - more complex
- Separate thread per connection - inefficient

### Decision 5: Inline addon config with --set

**Choice:** Support `--set block_url=.*ads.*` for inline addon configuration.

**Rationale:**
- Matches Python mitmproxy UX
- Easy to use for simple configurations
- Extensible for future addon options

**Alternatives considered:**
- Configuration file only - less convenient for quick tests
- Environment variables - less readable

## Risks / Trade-offs

### Risk: CA certificate generation fails
**Mitigation:** Handle errors gracefully. Provide clear error messages. Allow user to specify existing CA path.

### Risk: Large flows impact performance
**Mitigation:** Use buffered I/O. Profile with large flows. Add streaming if needed.

### Risk: Ctrl+C handling misses connections
**Mitigation:** Use `tokio::signal::ctrl_c()` with proper cleanup. Drain connections before exit.

### Risk: Config file conflicts with CLI args
**Mitigation:** CLI args always override config file (documented behavior).

## Migration Plan

**Phase 1: Core binary structure**
1. Rewrite `crates/mitm-cli/src/main.rs`
2. Add dependencies to Cargo.toml
3. Implement basic clap argument parsing
4. Add startup output

**Phase 2: Proxy initialization**
1. Load config file if present
2. Initialize CaRoot and CertStore
3. Register built-in addons
4. Create ProxyServer from options

**Phase 3: Runtime features**
1. Implement graceful shutdown (Ctrl+C)
2. Add per-flow logging
3. Implement dump mode
4. Test with curl

**Rollback:** Revert mitm-cli changes if issues found.

## Open Questions

1. Should we support `--log-level` flag for trace/debug/info/warn/error?
2. Should dump mode write to stdout or file?
3. Should we add `--version` flag?
4. What's the default log format (human-readable vs JSON)?
