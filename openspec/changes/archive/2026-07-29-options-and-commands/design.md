## Context

mitmproxy-rs currently has no centralized configuration system. The proxy server accepts individual parameters (host, port, mode) but there is no way to:
- Persist settings across sessions
- Load configuration from a file
- Validate options before startup
- Change options at runtime

The Python mitmproxy has a mature options system (`mitmproxy/options.py`, `mitmproxy/optmanager.py`) that provides:
- CLI argument parsing with clap-like derive macros
- YAML config file loading
- Runtime options management with change notifications
- Validation and defaults

## Goals / Non-Goals

**Goals:**
- Define `Options` struct with clap derive for CLI and serde for config files
- Support config file (`~/.mitmproxy/config.yaml`) with CLI override semantics
- Provide `OptManager` for runtime get/set with validation
- Define `ProxyMode` enum: Explicit, Transparent, Upstream, Local
- Integrate with mitm-proxy and mitm-cli crates

**Non-Goals:**
- Hot-reloading config files (future enhancement)
- Environment variable overrides (future enhancement)
- Options schema validation against JSON Schema (future enhancement)
- GUI options editor (out of scope)

## Decisions

### Decision 1: Separate crate for options

**Choice:** Create `mitm-options` crate instead of adding to `mitm-proxy`.

**Rationale:**
- Single responsibility (options management vs proxy logic)
- Reduces compile times (options code independent)
- Clear dependency boundaries (mitm-cli and mitm-proxy both depend on it)
- Follows existing crate structure (mitm-core, mitm-net, mitm-proxy, mitm-certs, mitm-addons)

**Alternatives considered:**
- Add to `mitm-proxy` - mixes concerns, larger compile times
- Add to `mitm-core` - too low-level, adds dependency on async and CLI libs

### Decision 2: Use clap for derive-based CLI parsing

**Choice:** Use `clap` v4 with `Parser` derive macro.

**Rationale:**
- Industry standard for Rust CLI parsing
- Derive macros reduce boilerplate
- Good error messages and help text generation
- Supports nested structs for config file compatibility

**Alternatives considered:**
- `structopt` (deprecated, superseded by clap)
- Manual `Arg::new()` calls - more verbose, error-prone

### Decision 3: YAML for config files

**Choice:** Use `serde_yaml` for config file format.

**Rationale:**
- Human-readable format
- Native Rust support via `serde`
- Compatible with Python mitmproxy YAML config
- Better than JSON for comments and documentation

**Alternatives considered:**
- JSON - less readable, no comments
- TOML - less common for config files in Rust ecosystem
- INI - limited nesting support

### Decision 4: OptManager with Arc<RwLock>

**Choice:** Use `Arc<RwLock<Options>>` for runtime management.

**Rationale:**
- Thread-safe access to options
- Read-heavy workload (many reads, few writes)
- Compatible with async code (tokio::sync::RwLock available)
- Simple API: `get()` and `set()` methods

**Alternatives considered:**
- `Cell`/`RefCell` - not thread-safe
- Channel-based updates - more complex, unnecessary for current use case

### Decision 5: ProxyMode as enum with FromStr

**Choice:** Define `ProxyMode` as Rust enum with `FromStr` implementation.

**Rationale:**
- Type-safe mode selection
- Compile-time checking
- Easy parsing from string (CLI/config)
- Pattern matching for mode-specific logic

**Alternatives considered:**
- String-based modes - error-prone, no compile-time checking
- U32 constants - less readable

## Risks / Trade-offs

### Risk: Config file format changes break compatibility
**Mitigation:** Use serde's flexible deserialization. Add version field to config file in future if needed.

### Risk: OptManager contention in high-throughput scenarios
**Mitigation:** Use `RwLock` (read-heavy optimization). Profile first, switch to channel-based if needed.

### Risk: Clap derive limitations for complex nested structures
**Mitigation:** Clap v4 handles nested structs well. If issues arise, fall back to manual `Parser` impl.

### Risk: Config file directory creation fails
**Mitigation:** Use `dirs` crate for platform-specific paths. Handle `io::Error` gracefully with clear error messages.

## Migration Plan

**Phase 1: Create mitm-options crate**
1. Create `crates/mitm-options/` with Cargo.toml
2. Define `Options` struct with clap derive
3. Define `ProxyMode` enum
4. Implement basic `OptManager`
5. Add unit tests

**Phase 2: Config file integration**
1. Implement config file loading (YAML)
2. Implement CLI override merge logic
3. Add config file generation (dump current options)
4. Add integration tests

**Phase 3: Proxy integration**
1. Update `ProxyServer::bind()` to accept `&Options`
2. Update `mitm-cli` to use clap-derived Options
3. Add integration tests with real proxy

**Rollback:** Revert mitm-options crate and restore individual parameters in mitm-proxy.

## Open Questions

1. Should we support environment variable overrides (e.g., `MITM_PROXY_PORT`)?
2. Should config file support includes/imports for modular config?
3. Should we add `--dump-config` flag to generate a sample config file?
4. What's the precedence: CLI > Env > Config file?
