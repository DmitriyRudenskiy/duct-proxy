## Why

mitmproxy-rs requires a plugin system to extend functionality without modifying core code. The Python mitmproxy has a robust addon system with hooks for request/response modification, filtering, and custom behavior. This change implements a similar extensible addon framework in Rust, enabling third-party developers to create custom proxy behavior through a well-defined trait interface.

## What Changes

- **Addon trait definition** with lifecycle hooks for HTTP, TCP, UDP, and DNS flows
- **AddonManager** for registration, sequential dispatch, and error isolation
- **Hook lifecycle** matching mitmproxy: requestheaders → request → responseheaders → response
- **Built-in addons**: ModifyHeaders, ModifyBody, Block, Filter
- **Integration** with mitm-proxy HookDispatcher for addon dispatch

## Capabilities

### New Capabilities

- `addon-trait`: Defines the Addon trait with lifecycle hooks (requestheaders, request, responseheaders, response, error, tcp_message, udp_message, dns_request, dns_response)
- `addon-manager`: Addon registration, sequential dispatch, error isolation, and lifecycle management
- `builtin-addons`: Built-in addons (ModifyHeaders, ModifyBody, Block, Filter) with configuration

### Modified Capabilities

<!-- No existing capabilities need spec-level changes -->

## Impact

**Affected Code:**
- `crates/mitm-addons/` - New crate for addon system
- `crates/mitm-proxy/src/hooks.rs` - Integration with HookDispatcher
- `crates/mitm-core/src/flow.rs` - Flow types used by addon hooks

**Dependencies:**
- `mitm-core` - Flow types (HTTPFlow, TCPFlow, etc.)
- `mitm-proxy` - HookDispatcher for addon dispatch

**Breaking Changes:** None - addon system is additive
