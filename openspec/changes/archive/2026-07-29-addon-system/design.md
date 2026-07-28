## Context

mitmproxy-rs currently has basic hook system in mitm-proxy (hooks.rs) with `HttpRequestHook`, `HttpResponseHook`, and `ErrorHook` traits. However, this system is limited and doesn't support:
- Full lifecycle hooks (requestheaders, request, responseheaders, response)
- TCP/UDP/DNS flow hooks
- Built-in addons for common tasks
- Error isolation between addons
- Configuration and filtering

The Python mitmproxy has a robust addon system with `Addon` base class, `AddonManager`, and built-in addons like `modify.py`, `block.py`, `filter.py`. This change implements a similar system in Rust.

## Goals / Non-Goals

**Goals:**
- Define `Addon` trait with full lifecycle hooks (HTTP, TCP, UDP, DNS)
- Implement `AddonManager` with sequential dispatch and error isolation
- Create built-in addons: ModifyHeaders, ModifyBody, Block, Filter
- Integrate with mitm-proxy HookDispatcher for addon dispatch
- Support async addon execution with tokio

**Non-Goals:**
- Hot-reloading of addons (future enhancement)
- Addon packaging/distribution system (future enhancement)
- Addon configuration file parsing (use programmatic API)
- WebSocket addon hooks (out of scope for v1)

## Decisions

### Decision 1: Use trait objects (dyn Addon) instead of generics

**Choice:** Use `Vec<Box<dyn Addon>>` for addon storage instead of generic types.

**Rationale:**
- Runtime addon registration (users add addons at runtime)
- Heterogeneous addon types in the same manager
- Matches mitmproxy Python API design
- Simpler API for addon authors

**Alternatives considered:**
- Generic `AddonManager<T>` - requires all addons to be same type
- Enum-based dispatch - limits extensibility

### Decision 2: Sequential dispatch with error isolation

**Choice:** Dispatch hooks sequentially, stop on error (configurable).

**Rationale:**
- Predictable behavior (addons execute in registration order)
- Error isolation prevents one bad addon from affecting others
- Matches mitmproxy behavior
- Easier to debug than parallel dispatch

**Alternatives considered:**
- Parallel dispatch with `JoinSet` - harder to debug, order undefined
- Fire-and-forget - errors lost

### Decision 3: Use thiserror for addon errors

**Choice:** Use `thiserror` for `AddonError` enum.

**Rationale:**
- Clean error messages with context
- Automatic Display implementation
- Already used in other crates (mitm-certs, mitm-proxy)
- Compatible with anyhow for ergonomic propagation

### Decision 4: Separate crate for addon system

**Choice:** Create `mitm-addons` crate instead of adding to mitm-proxy.

**Rationale:**
- Single responsibility (addon system vs proxy engine)
- Reduces compile times (addon code independent)
- Clear dependency boundaries
- Follows existing crate structure (mitm-core, mitm-net, mitm-proxy, mitm-certs)

**Alternatives considered:**
- Add to mitm-proxy - mixes concerns, larger compile times
- Add to mitm-core - too low-level, adds dependency on async

### Decision 5: Flow types from mitm-core

**Choice:** Use `mitm_core::Flow` variants (HTTPFlow, TCPFlow, etc.) for addon hooks.

**Rationale:**
- Single source of truth for flow types
- Consistent with proxy engine
- Avoids duplication

**Alternatives considered:**
- Define new flow types in mitm-addons - duplication, sync issues

## Risks / Trade-offs

### Risk: Async complexity for addon authors
**Mitigation:** Provide sync-friendly API where possible. Use `async fn` but allow blocking operations inside (with warnings).

### Risk: Performance overhead from dyn dispatch
**Mitigation:** Profile first. If needed, add trait method caching or specialization later.

### Risk: Addon errors breaking proxy
**Mitigation:** Error isolation in AddonManager. Addon errors don't crash proxy, just skip remaining addons.

### Risk: Memory leaks from addon closures
**Mitigation:** Document best practices. Use `Arc` for shared state. Provide drop hooks for cleanup.

## Migration Plan

This change is additive - no breaking changes to existing APIs.

**Steps:**
1. Create mitm-addons crate with Addon trait and AddonError
2. Implement AddonManager with sequential dispatch
3. Create built-in addons (ModifyHeaders, ModifyBody, Block, Filter)
4. Integrate with mitm-proxy HookDispatcher
5. Add comprehensive tests
6. Document addon authoring guide

**Rollback:** Revert mitm-addons crate if issues found.

## Open Questions

1. Should addons support synchronous execution (no async)?
2. What's the maximum number of addons we should support before performance degrades?
3. Should we support addon configuration via YAML/JSON files?
4. How should addons handle flow modification (clone flow or modify in place)?
