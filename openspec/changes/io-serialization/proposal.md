## Why

mitmproxy-rs currently lacks persistent storage for captured flows. Users cannot save intercepted HTTP traffic to disk for later analysis, export to external tools, or share captured data. The Python mitmproxy supports JSON dump format with gzip compression (.jsonl.gz) and HAR export. This change implements a similar serialization system in Rust.

## What Changes

- **Flow serialization**: Serialize `HTTPFlow` to JSON using `serde_json`
- **Dump format**: Gzip-compressed sequential JSON lines (`.jsonl.gz`) for efficient streaming
- **FlowWriter**: Append flows to file with `BufWriter` + `GzEncoder` for efficient I/O
- **FlowReader**: Read flows from file with `GzDecoder` + line-by-line parsing
- **HAR export** (optional): Convert `HTTPFlow` to HAR 1.2 spec format

## Capabilities

### New Capabilities

- `flow-serialization`: Core serialization of HTTPFlow to JSON, deserialization back
- `dump-format`: Gzip-compressed JSONL format with streaming read/write
- `har-export`: Export flows to HAR 1.2 specification (optional)

### Modified Capabilities

- None (this is a new system, not modifying existing specs)

## Impact

**New Dependencies:**
- `serde_json` - JSON serialization
- `flate2` - Gzip compression/decompression
- `chrono` - Timestamps for HAR export

**Affected Code:**
- `crates/mitm-io/` - Core serialization logic
- `crates/mitm-proxy/` - Will use FlowWriter/FlowReader for dump functionality
- `crates/mitm-cli/` - Will add `--dump` and `--export-har` CLI flags

**API Changes:**
- New public API: `FlowWriter`, `FlowReader`, `FlowSerializer`, `HarExporter`
- `mitm-io` crate dependencies: serde_json, flate2, chrono
