## Why

mitmproxy-rs currently has no working binary. The `mitm-cli` crate exists but has no implementation. Users need a working command-line tool to run the proxy, configure it, and observe traffic. This change implements the complete CLI binary with all features needed for a usable mitmproxy replacement.

## What Changes

- **main.rs**: Complete CLI entry point with clap parsing, config loading, CA initialization, addon setup, proxy server startup
- **Modes**: Default proxy mode, dump mode (--dump), inline addon config (--set)
- **Graceful shutdown**: Ctrl+C handling with accept loop stop, connection draining, clean exit
- **Startup output**: Version, listening address, CA path, registered addons
- **Per-flow logging**: Formatted log lines showing request/response details
- **Dependencies**: All workspace crates + clap, tokio, tracing-subscriber

## Capabilities

### New Capabilities

- `cli-binary`: Working mitm-cli binary with all features
- `cli-modes`: Proxy, dump, and inline configuration modes
- `cli-logging`: Per-flow logging with timing and size information

### Modified Capabilities

- None (this is a new binary, not modifying existing specs)

## Impact

**New Dependencies:**
- `clap` v4 - CLI argument parsing
- `tracing-subscriber` - Structured logging
- All workspace crates: mitm-core, mitm-net, mitm-proxy, mitm-certs, mitm-addons, mitm-options, mitm-io

**Affected Code:**
- `crates/mitm-cli/` - Complete rewrite of main.rs
- `crates/mitm-proxy/` - May need minor adjustments for binary integration

**API Changes:**
- New binary: `mitm-cli` (replaces placeholder)
- CLI interface: `mitm-cli [OPTIONS]` with --port, --host, --mode, --dump, --set, --config flags
