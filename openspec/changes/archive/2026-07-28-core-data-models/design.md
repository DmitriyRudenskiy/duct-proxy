## Context

mitmproxy-rs is a Rust rewrite of the Python mitmproxy tool. The project currently has an empty workspace skeleton with eight crate stubs. No parsing, proxy logic, or addon system exists yet. The foundational data model — the types that represent network flows, connections, HTTP messages, DNS queries, and WebSocket sessions — has not been implemented.

The Python reference implementation uses inheritance-based OOP: `Flow` is an abstract base class with `HTTPFlow`, `TCPFlow`, `UDPFlow`, `DNSFlow` as subclasses. HTTP messages use `Message` → `Request`/`Response` with embedded `MessageData`. Headers are a custom case-insensitive MultiDict. Serialization uses a custom `get_state()`/`set_state()` pattern rather than reflection-based frameworks.

Constraints for the Rust rewrite:
- Must be idiomatic Rust, not a direct translation of Python patterns
- Must use serde for serialization (not custom get_state/set_state)
- Must work with tokio async runtime
- Must support the addon system (hooks that receive flow references)
- Must be efficient for high-throughput proxying

## Goals / Non-Goals

**Goals:**
- Define all core data types as Rust structs/enums
- Establish composition vs inheritance patterns appropriate for Rust
- Provide serde-based serialization matching mitmproxy's state format
- Create types that are ergonomic for addon authors and proxy internals
- Support the full mitmproxy flow lifecycle: create → intercept → process → resume/kill

**Non-Goals:**
- Implementing HTTP/TCP/UDP/DNS protocol parsing (handled by separate crates)
- Implementing the addon/plugin system (depends on these types)
- Implementing the proxy server (depends on these types)
- Certificate generation/management (separate crate)
- Web UI integration (consumes serialized forms)

## Decisions

### Decision 1: Flow hierarchy as enum, not trait

**Choice:** Use `enum Flow { Http(HTTPFlow), Tcp(TCPFlow), Udp(UDPFlow), Dns(DNSFlow) }` with a shared `FlowBase` struct embedded in each variant.

**Rationale:**
- Finite, known set of flow types — enum gives exhaustiveness checking
- Zero-cost dispatch via pattern matching (no vtable overhead)
- serde `#[serde(tag = "type")]` provides clean tagged serialization
- Each variant is a concrete struct — no runtime type checking needed

**Alternatives considered:**
- *Trait-based (`trait Flow`)*: More extensible but loses exhaustiveness, adds dynamic dispatch cost, harder serde.
- *Flat struct with discriminant*: Loses type safety — any field could be accessed on any variant.

**Trade-off:** Adding a new flow type requires updating the enum and all match sites. This is acceptable because flow types change infrequently.

### Decision 2: FlowBase embedded in each variant, not shared reference

**Choice:** Each flow variant contains `base: FlowBase` as a value, not `Arc<FlowBase>`.

**Rationale:**
- Simpler ownership — no lifetime complexity
- Flows are typically short-lived per-transaction objects
- Copy is explicit and cheap (UUID + timestamps + HashMap)

**Alternatives considered:**
- *Arc<FlowBase>*: Would allow cheap cloning but adds indirection for every field access.
- *Generic parameter `<B>`*: Overly complex for no benefit.

### Decision 3: Headers as custom type wrapping Vec + HashMap

**Choice:** `struct Headers { fields: Vec<(Vec<u8>, Vec<u8>)>, index: HashMap<String, Vec<usize>> }`

**Rationale:**
- Preserves insertion order (critical for HTTP — some servers care about header order)
- Case-insensitive lookup via lowercase HashMap index
- Multiple values per key (Set-Cookie!) supported via index → Vec<usize>
- Internal storage as bytes (matching HTTP wire format)

**Alternatives considered:**
- *indexmap::IndexMap<String, Vec<Vec<u8>>>*: Loses original casing of keys. Acceptable if casing doesn't matter, but mitmproxy preserves original casing for fidelity.
- *TwoHashMap<String, Vec<(Vec<u8>, Vec<u8>)>>*: More complex lookup logic.
- *vec! + linear scan*: Too slow for high-traffic proxy.

### Decision 4: Message content as Option<Vec<u8>> with explicit methods

**Choice:** Content is stored as `Option<Vec<u8>>` in `MessageData`. Access is via properties (`raw_content`, `content`, `text`, `json`) and setters (`set_content`, `set_text`, `decode`, `encode`).

**Rationale:**
- `Option<Vec<u8>>` clearly distinguishes "no body" from "empty body"
- Explicit methods handle content-length header management
- Decompression is lazy — `content` property decodes on access

**Alternatives considered:**
- *Eager decompression on set*: Would break streaming semantics.
- *Content as stream/iterator*: Overly complex for the common case; streaming is handled by `Message.stream`.

### Decision 5: Streaming mode as enum, not Box<dyn Fn>

**Choice:** `enum StreamMode { Buffered, Passthrough, Transform(Box<dyn FnMut(&[u8]) -> Vec<u8>>) }`

**Rationale:**
- The callable case is rare (addon transforms) — wrapping in enum makes it explicit
- `Box<dyn FnMut>` is unavoidable for the transform case (closure type is unnameable)
- Clear API: `message.stream = StreamMode::Passthrough` vs `message.stream = true`

**Alternatives considered:**
- *Boolean `stream: bool`*: Too limited — can't express transform functions.
- *Separate field for transform*: Inconsistent API.

### Decision 6: DNS wire format with dedicated module

**Choice:** DNS parsing/serialization lives in `mitm-net` as a dedicated module. `DNSMessage` exposes `packed: Vec<u8>` (getter) and `DNSMessage::unpack(bytes)` (parser).

**Rationale:**
- DNS is a binary protocol with specific wire format requirements (compression pointers, etc.)
- Separating wire format from in-memory representation enables efficient network I/O
- `packed` as a property (computed from fields) avoids storing redundant bytes

**Alternatives considered:**
- *Store packed bytes alongside parsed fields*: Wastes memory, risks inconsistency.
- *Use a DNS crate (e.g., hickory-proto)*: Adds dependency; the protocol is simple enough to implement directly.

### Decision 7: WebSocket attached to HTTPFlow, not standalone flow type

**Choice:** `WebSocketData` is an optional field on `HTTPFlow`. There is no standalone `WebSocketFlow`.

**Rationale:**
- WebSocket always starts as an HTTP upgrade — the HTTP flow IS the WebSocket flow
- Matches the Python reference (changed in mitmproxy 6)
- Simplifies the flow enum — no separate WebSocket variant needed

### Decision 8: Resume state via tokio sync primitives

**Choice:** Each `Flow` has `resume_sender: tokio::sync::broadcast::Sender<()>` and `resume_receiver: tokio::sync::broadcast::Receiver<()>`. `wait_for_resume()` subscribes and awaits. `resume()` sends on the broadcast channel.

**Rationale:**
- Matches Python's `asyncio.Event` semantics
- Broadcast allows multiple waiters (multiple hooks can be paused on same flow)
- `broadcast::Receiver::changed()` provides efficient async waiting

**Alternatives considered:**
- *tokio::sync::Notify*: One-shot, no waiters. Would need a Vec<Notify> or mutex.
- *Arc<Mutex<bool>> + condvar*: Works in sync context but not idiomatic for async.
- *Channel-based (Sender<()>)*: One waiter only.

### Decision 9: Cert as wrapper around x509-parser

**Choice:** `struct Cert { inner: x509_parser::certificate::X509Certificate<'static> }` — thin wrapper providing convenience methods (fingerprint, cn, altnames).

**Rationale:**
- `x509-parser` is the standard Rust x509 crate
- Wrapping provides a stable API independent of crate internals
- `'static` lifetime requires leaking/copying — acceptable for cert data

**Alternatives considered:**
- *Direct x509-parser types*: Leaks dependency details into the data model.
- *cryptography-rs bindings*: Heavier dependency, Python ecosystem only.

## Risks / Trade-offs

[Risk: Headers HashMap index can get out of sync] → [Mitigation: Private API — all modifications go through methods that update both `fields` and `index` together. No public field access.]

[Risk: DNS compression pointers require two-pass parsing] → [Mitigation: Use a stateful parser with a name cache. The `unpack` method takes a buffer and offset, maintaining compression context internally.]

[Risk: Large flows in memory for high-traffic proxy] → [Mitigation: Streaming mode (`StreamMode::Passthrough`) avoids buffering bodies. For stored flows (.flow files), use serde_json with streaming deserializer.]

[Risk: Thread safety for concurrent addon access] → [Mitigation: Flows are processed sequentially per-connection. Cross-connection sharing uses Arc. Addon system design (future) will determine if Send/Sync bounds are needed.]

[Trade-off: Duplicating FlowBase fields across 4 flow variants increases binary size slightly] → [Mitigation: FlowBase is ~200 bytes; 4 copies = ~800 bytes per flow. Negligible compared to network buffers.]

[Trade-off: Custom Headers type vs indexmap dependency] → [Mitigation: Custom type avoids indexmap dependency but increases code to maintain. Acceptable for this scope.]

## Migration Plan

This is a greenfield Rust implementation — no migration from Python code needed. The implementation proceeds in phases:

1. **Phase 1 (this change):** Define all data types with serde serialization. No parsing, no proxy logic.
2. **Phase 2:** Implement HTTP request/response parser (produces `Request`/`Response` from bytes).
3. **Phase 3:** Implement TLS layer (produces `Client`/`Server` with TLS metadata).
4. **Phase 4:** Implement TCP/UDP/DNS parsers.
5. **Phase 5:** Wire up the addon system hooks.
6. **Phase 6:** Implement the proxy server orchestration.

Rollback for this phase: delete the crate modules and revert Cargo.toml. No data loss possible.

## Open Questions

1. **Should `FlowBase` have a `parent` field?** Python mitmproxy doesn't have this, but some proxy architectures track flow trees. Probably not needed for v1.

2. **Should Headers support binary keys?** Some HTTP/2 pseudo-headers start with `:`. Python stores everything as bytes, so yes — but Rust's `HashMap<String, ...>` index needs a string key. Solution: lowercase the bytes and convert via `from_utf8_lossy`. Edge case: truly non-UTF-8 header names.

3. **Should `Request.authority` use idna encoding?** Python uses `idna` decoding for authority. Rust can use the `idna` crate or `rustybellerophon`.

4. **DNS HTTPS record support:** The Python reference has `https_alpn` and `https_ech` accessors on ResourceRecord. Should these be implemented in v1, or deferred?

5. **Should we derive Clone on all types?** Most types will need Clone for flow copying. Derive everywhere except types with `Box<dyn FnMut>` (transform closures are not Clone).
