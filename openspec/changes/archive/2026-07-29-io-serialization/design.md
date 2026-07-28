## Context

mitmproxy-rs currently has no persistent storage for captured flows. The `mitm-io` crate exists but has no serialization implementation. Users need to:
- Save captured HTTP traffic to disk for later analysis
- Export flows in standard formats (JSON, HAR)
- Stream flows to files with compression for efficiency

The Python mitmproxy supports:
- JSON dump format (`.json` or `.jsonl.gz` with gzip)
- HAR 1.2 export for browser compatibility
- Streaming write with compression

## Goals / Non-Goals

**Goals:**
- Define `FlowSerializer` trait for converting `HTTPFlow` to JSON
- Implement `FlowWriter` for appending flows to `.jsonl.gz` files
- Implement `FlowReader` for reading flows from `.jsonl.gz` files
- Support HAR 1.2 export (optional, marked as future)
- Integrate with `mitm-proxy` for dump functionality

**Non-Goals:**
- Binary serialization formats (MessagePack, etc.)
- Database storage (SQLite, etc.)
- Real-time streaming to remote services
- GUI for flow inspection

## Decisions

### Decision 1: Use JSON Lines format (.jsonl)

**Choice:** Use JSON Lines (one JSON object per line) instead of a single JSON array.

**Rationale:**
- Streaming-friendly: can write flows as they arrive without buffering all
- Append-safe: can safely append to existing file
- Compatible with Unix tools (grep, awk, etc.)
- Standard format used by many tools (mitmproxy, nghttp, etc.)

**Alternatives considered:**
- Single JSON array - requires buffering all flows, not append-safe
- NDJSON (Newline Delimited JSON) - same as JSON Lines, just different name

### Decision 2: Gzip compression for dump files

**Choice:** Use `flate2` crate for gzip compression.

**Rationale:**
- Reduces file size significantly (typically 5-10x for text)
- Standard format, widely supported
- `flate2` is fast and well-maintained
- Compatible with Python mitmproxy dump files

**Alternatives considered:**
- No compression - larger files, faster I/O
- zstd - better compression ratio, less universal support
- bz2 - slower, less common

### Decision 3: Separate crate for serialization

**Choice:** Keep serialization in `mitm-io` crate (already exists).

**Rationale:**
- Single responsibility (I/O operations)
- Already exists in workspace
- Follows existing crate structure
- Reduces dependency overhead for other crates

**Alternatives considered:**
- Add to `mitm-proxy` - mixes concerns, larger compile times
- New `mitm-serialize` crate - unnecessary complexity

### Decision 4: HAR export as optional module

**Choice:** Implement HAR export as optional module, not core.

**Rationale:**
- HAR is a large spec with many optional fields
- Not all users need HAR export
- Can be added later without breaking core functionality
- Keeps `mitm-io` focused on core serialization

**Alternatives considered:**
- Include HAR in core - adds complexity, dependencies
- Separate `mitm-har` crate - unnecessary for v1

### Decision 5: Use serde for serialization

**Choice:** Use `serde_json` with derive macros for JSON serialization.

**Rationale:**
- Industry standard for Rust JSON
- Derive macros reduce boilerplate
- Automatic serialization/deserialization
- Compatible with existing `serde` usage in mitm-core

**Alternatives considered:**
- Manual serialization - more error-prone, verbose
- Custom serializer - unnecessary complexity

## Risks / Trade-offs

### Risk: Large dump files impact performance
**Mitigation:** Use gzip compression and async I/O. Profile with large flows.

### Risk: JSON serialization loses binary data
**Mitigation:** Base64 encode binary content in HAR export. Document limitation.

### Risk: Gzip compression adds CPU overhead
**Mitigation:** Use buffered I/O. Compression is fast for typical flow sizes.

### Risk: HAR export complexity
**Mitigation:** Implement basic HAR 1.2 first. Add advanced fields later.

## Migration Plan

**Phase 1: Core serialization**
1. Create `FlowSerializer` trait and JSON implementation
2. Add `#[derive(Serialize, Deserialize)]` to `HTTPFlow` (if not already)
3. Add unit tests for serialization/deserialization

**Phase 2: Dump format**
1. Implement `FlowWriter` with `BufWriter` + `GzEncoder`
2. Implement `FlowReader` with `GzDecoder` + line parsing
3. Add integration tests with real files

**Phase 3: HAR export (optional)**
1. Define HAR types (Log, Page, Request, Response)
2. Implement `HarExporter` converting `HTTPFlow` to HAR
3. Add tests for HAR 1.2 compliance

**Rollback:** Revert `mitm-io` changes if issues found.

## Open Questions

1. Should we support multiple dump formats (JSON, JSONL, HAR) in CLI?
2. Should `FlowWriter` support both compressed and uncompressed modes?
3. What's the maximum flow size we should handle efficiently?
4. Should we add a `--dump` CLI flag to mitm-cli?
