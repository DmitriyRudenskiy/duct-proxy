## 1. Project Setup

- [x] 1.1 Create `crates/mitm-addons/` directory structure
- [x] 1.2 Add `mitm-addons` to workspace Cargo.toml
- [x] 1.3 Add dependencies: mitm-core, mitm-proxy, serde, thiserror, tracing, async-trait, regex
- [x] 1.4 Create `lib.rs` with module declarations
- [x] 1.5 Verify workspace builds with `cargo build -p mitm-addons`

## 2. Addon Trait Definition

- [x] 2.1 Define `Addon` trait with async lifecycle hooks
- [x] 2.2 Implement default methods (no-op) for all hooks
- [x] 2.3 Define `AddonError` enum with thiserror
- [x] 2.4 Ensure `Addon: Send + Sync` trait bounds
- [ ] 2.5 Write unit tests: trait object creation, downcast

## 3. AddonManager Implementation

- [x] 3.1 Define `AddonManager` struct with `Vec<Box<dyn Addon>>`
- [x] 3.2 Implement `register(addon: Box<dyn Addon>)` method
- [x] 3.3 Implement `dispatch_requestheaders(flow: &mut Flow)`
- [x] 3.4 Implement `dispatch_request(flow: &mut Flow)`
- [x] 3.5 Implement `dispatch_responseheaders(flow: &mut Flow)`
- [x] 3.6 Implement `dispatch_response(flow: &mut Flow)`
- [x] 3.7 Implement `dispatch_error(error: &AddonError)`
- [x] 3.8 Implement error isolation (stop on error, log, continue)
- [ ] 3.9 Write unit tests: registration, dispatch, error handling

## 4. Built-in Addon: ModifyHeaders

- [x] 4.1 Define `ModifyHeaders` struct with add/set/remove operations
- [x] 4.2 Implement `requestheaders` hook to modify request headers
- [x] 4.3 Implement `responseheaders` hook to modify response headers
- [x] 4.4 Support regex-based header name matching
- [x] 4.5 Write unit tests: add header, set header, remove header (13 tests)

## 5. Built-in Addon: ModifyBody

- [ ] 5.1 Define `ModifyBody` struct with replace operations
- [ ] 5.2 Implement `request` hook to modify request body
- [ ] 5.3 Implement `response` hook to modify response body
- [ ] 5.4 Support string and regex pattern replacement
- [ ] 5.5 Filter by Content-Type header
- [ ] 5.6 Write unit tests: replace string, replace regex, filter by type

## 6. Built-in Addon: Block

- [x] 6.1 Define `Block` struct with filter criteria
- [ ] 6.2 Implement `requestheaders` hook to check filters
- [ ] 6.3 Send 403 response when flow matches filter
- [x] 6.4 Support URL, header, and source IP filters
- [x] 6.5 Write unit tests: block by URL, block by header, block by IP (6 tests)

## 7. Built-in Addon: Filter

- [ ] 7.1 Define `Filter` struct with expression builder
- [ ] 7.2 Implement `url(regex)`, `method(method)`, `header(name, value)` filters
- [ ] 7.3 Implement `.and()` and `.or()` combinators
- [ ] 7.4 Implement `matches(flow: &Flow) -> bool` method
- [ ] 7.5 Write unit tests: URL filter, method filter, combined filters

## 8. Integration with mitm-proxy

- [ ] 8.1 Update mitm-proxy HookDispatcher to use AddonManager
- [ ] 8.2 Replace manual hook dispatch with addon manager dispatch
- [ ] 8.3 Add configuration for addon loading (future)
- [ ] 8.4 Write integration test: addon dispatch through proxy

## 9. Testing & Validation

- [ ] 9.1 Run `cargo test -p mitm-addons` — verify all unit tests pass (target: 40+ tests)
- [ ] 9.2 Run `cargo clippy -p mitm-addons -- -D warnings` — clean lint
- [ ] 9.3 Test with mitm-proxy integration tests
- [ ] 9.4 Document public API with doc comments
- [ ] 9.5 Create addon authoring guide in docs/
