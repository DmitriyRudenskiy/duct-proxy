## 1. Project Setup

- [x] 1.1 Create `crates/mitm-options/` directory structure
- [x] 1.2 Add `mitm-options` to workspace Cargo.toml
- [x] 1.3 Add dependencies: clap, serde, serde_yaml, dirs, tracing
- [x] 1.4 Create `lib.rs` with module declarations
- [x] 1.5 Verify workspace builds with `cargo build -p mitm-options`

## 2. Options Struct

- [x] 2.1 Define `Options` struct with clap::Parser derive
- [x] 2.2 Add fields: listen_host, listen_port, mode, ssl_insecure, conf_dir
- [x] 2.3 Add serde Serialize/Deserialize derives
- [x] 2.4 Add Clone and Debug derives
- [x] 2.5 Set default values for all fields
- [x] 2.6 Write unit tests: struct creation, default values (6 tests)

## 3. ProxyMode Enum

- [x] 3.1 Define `ProxyMode` enum with Explicit, Transparent, Upstream, Local variants
- [x] 3.2 Implement `FromStr` for parsing from string (case-insensitive)
- [x] 3.3 Implement `Display` for formatting
- [x] 3.4 Implement `Serialize` and `Deserialize` (manual impl for string repr)
- [x] 3.5 Write unit tests: parsing, display, serialization (5 tests)

## 4. Config File Loading

- [ ] 4.1 Implement `Options::from_config(path: &Path) -> Result<Self>`
- [ ] 4.2 Use serde_yaml for deserialization
- [ ] 4.3 Handle missing config file gracefully (use defaults)
- [ ] 4.4 Handle invalid YAML with clear error messages
- [ ] 4.5 Use `dirs` crate for default config path
- [ ] 4.6 Write unit tests: load config, missing file, invalid YAML

## 5. CLI Override Merge

- [ ] 5.1 Implement merge logic: CLI args override config file
- [ ] 5.2 Create `Options::merge(cli: &Self, config: &Self) -> Self`
- [ ] 5.3 CLI fields with `Some(value)` override config fields
- [ ] 5.4 Config fields used as defaults for unset CLI fields
- [ ] 5.5 Write unit tests: merge precedence, partial CLI, empty CLI

## 6. OptManager

- [ ] 6.1 Define `OptManager` struct with `Arc<RwLock<Options>>`
- [ ] 6.2 Implement `new(options: Options) -> Self`
- [ ] 6.3 Implement `get() -> Result<Options>` with RwLock read
- [ ] 6.4 Implement `set(options: Options) -> Result<()>` with RwLock write
- [ ] 6.5 Implement `clone()` for shared ownership
- [ ] 6.6 Write unit tests: create, get, set, clone, thread safety

## 7. Validation

- [ ] 7.1 Define `ValidationError` enum
- [ ] 7.2 Implement `validate(&self) -> Result<()>` on Options
- [ ] 7.3 Validate port range (1-65535)
- [ ] 7.4 Validate host format (IP address)
- [ ] 7.5 Integrate validation into OptManager::set()
- [ ] 7.6 Write unit tests: valid options, invalid port, invalid host

## 8. Config File Generation

- [ ] 8.1 Implement `Options::to_config_yaml(&self) -> Result<String>`
- [ ] 8.2 Use serde_yaml for serialization
- [ ] 8.3 Implement `Options::save_config(path: &Path) -> Result<()>`
- [ ] 8.4 Create config directory if it doesn't exist
- [ ] 8.5 Write unit tests: generate YAML, save to file, directory creation

## 9. Integration with mitm-proxy

- [x] 9.1 Update `ProxyServer::bind()` to accept `&Options` (via from_options)
- [x] 9.2 Extract listen_host, listen_port, mode from Options
- [x] 9.3 Update mitm-proxy Cargo.toml to depend on mitm-options
- [x] 9.4 Write integration test: proxy with Options struct (1 test)

## 10. Integration with mitm-cli

- [ ] 10.1 Update mitm-cli main.rs to use clap-derived Options
- [ ] 10.2 Load config file and merge with CLI args
- [ ] 10.3 Create OptManager from merged options
- [ ] 10.4 Pass OptManager to ProxyServer
- [ ] 10.5 Write integration test: CLI with config file

## 11. Testing & Validation

- [ ] 11.1 Run `cargo test -p mitm-options` — verify all unit tests pass (target: 30+ tests)
- [ ] 11.2 Run `cargo clippy -p mitm-options -- -D warnings` — clean lint
- [ ] 11.3 Run `cargo test --workspace` — verify no regressions
- [ ] 11.4 Document public API with doc comments
- [ ] 11.5 Create config file example in docs/
