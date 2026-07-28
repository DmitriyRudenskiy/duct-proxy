## 1. Project Setup

- [x] 1.1 Update `crates/mitm-io/Cargo.toml` with dependencies: serde_json, flate2, chrono
- [x] 1.2 Verify workspace builds with `cargo build -p mitm-io`
- [x] 1.3 Check existing `mitm-io/src/lib.rs` structure

## 2. Flow Serialization

- [x] 2.1 Define `FlowSerializer` trait with `serialize` and `deserialize` methods
- [x] 2.2 Implement `JsonFlowSerializer` using serde_json
- [x] 2.3 Add `#[derive(Serialize, Deserialize)]` to `HTTPFlow` (already present, added Send+Sync)
- [ ] 2.4 Handle binary content with base64 encoding (deferred)
- [x] 2.5 Write unit tests: serialize, deserialize, roundtrip (3 tests)

## 3. Dump Format - FlowWriter

- [x] 3.1 Define `FlowWriter` struct with `BufWriter<File>` and `GzEncoder`
- [x] 3.2 Implement `new(path: &Path) -> Result<Self>`
- [x] 3.3 Implement `write(flow: &HTTPFlow) -> Result<()>`
- [x] 3.4 Implement `close()` and `Drop` for proper cleanup
- [x] 3.5 Write unit tests: create file, write flow, close properly (3 tests)

## 4. Dump Format - FlowReader

- [x] 4.1 Define `FlowReader` struct with `BufReader<File>` and `GzDecoder`
- [x] 4.2 Implement `new(path: &Path) -> Result<Self>`
- [x] 4.3 Implement `read_next() -> Result<Option<HTTPFlow>>`
- [x] 4.4 Handle end-of-stream gracefully
- [x] 4.5 Write unit tests: read flows, handle missing file, EOF (3 tests)

## 5. Integration Tests

- [ ] 5.1 Write integration test: write flows, read them back, verify equality
- [ ] 5.2 Write integration test: verify gzip compression (file size reduction)
- [ ] 5.3 Verify compatibility with Python mitmproxy dump format (if possible)

## 6. HAR Export (Optional)

- [ ] 6.1 Define HAR types: `HarLog`, `HarEntry`, `HarRequest`, `HarResponse`
- [ ] 6.2 Implement `HarExporter` struct
- [ ] 6.3 Implement `add_entry(flow: &HTTPFlow)` conversion
- [ ] 6.4 Implement `export() -> Result<String>` for JSON output
- [ ] 6.5 Write unit tests: create log, add entry, export JSON (3 tests)

## 7. Testing & Validation

- [ ] 7.1 Run `cargo test -p mitm-io` — verify all unit tests pass (target: 15+ tests)
- [ ] 7.2 Run `cargo clippy -p mitm-io -- -D warnings` — clean lint
- [ ] 7.3 Run `cargo test --workspace` — verify no regressions
- [ ] 7.4 Document public API with doc comments
