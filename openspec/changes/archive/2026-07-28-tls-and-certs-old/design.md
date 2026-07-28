## Context

The `mitmproxy-rs` project has a placeholder `Cert` type in `mitm-core` (`Vec<u8>` with optional CN) and an empty `mitm-certs` crate. The data model references `certificate_list: Vec<Cert>` on `Connection`, but without real certificate types the proxy cannot perform TLS interception.

The Python reference uses `cryptography` (CFFI bindings) for all certificate operations. For Rust, the ecosystem offers:
- `x509-parser`: parsing X.509 certificates (read-only, fast)
- `rcgen`: generating certificates (simple API, no CA hierarchy)
- `rustls`: TLS implementation (our runtime)
- `ring` / `p384` / `rsa`: key generation and signing
- `pkcs8` / `pem`: key serialization

## Goals / Non-Goals

**Goals:**
- Real certificate types that can be parsed, generated, signed, and serialized
- CA hierarchy: root → intermediate → end-entity (matches mitmproxy's approach)
- In-memory store with LRU eviction for SNI-based cert dispatch
- On-disk persistence for CA keys (so they survive proxy restarts)
- TLS server config with SNI dispatch and ALPN
- TLS client config with verifiable peer certificates
- Integration with `rustls` for actual TLS handshake

**Non-Goals:**
- Implementing the TLS handshake itself (handled by `rustls` + proxy server)
- Certificate revocation checking (CRL/OCSP) — tracking only
- Certificate Transparency log integration (optional, v2)
- Hardware security module (HSM) support
- Certificate pinning (mitmproxy feature, not relevant for MITM proxy)

## Decisions

### Decision 1: Certificate representation — x509-parser wrapper

**Choice:** `struct Cert { inner: x509_parser::certificate::X509Certificate<'static> }`

**Rationale:**
- `x509-parser` is the standard Rust X.509 parser — fast, no unsafe, well-maintained
- `rcgen` generates certs but doesn't parse them; `x509-parser` parses but doesn't modify
- Wrapping `X509Certificate` with `'static` lifetime requires leaking/copying the parsed data
- Acceptable for our use case: certs are loaded once and served repeatedly

**Alternatives considered:**
- *rcgen Certificate*: Good for generation but not parsing. We need both.
- *raw DER bytes*: Too low-level; every operation requires re-parsing.
- *cryptography-rs bindings*: Heavier dependency, Python-only ecosystem.

### Decision 2: Key generation — ring crate

**Choice:** Use `ring` for RSA and EC key generation and signing.

**Rationale:**
- `ring` is the de facto standard for crypto in Rust (used by Cloudflare, Let's Encrypt)
- Provides RSA key pairs, EC key pairs (P-256, P-384), signing, and verification
- `rcgen` internally uses `ring` for signing — consistent with that dependency
- PKCS#8 serialization via `pkcs8` crate

**Alternatives considered:**
- *rsa crate*: Only RSA, no EC. We want EC support.
- *p384/p256 crates*: Only EC, no RSA. We want both.
- *openssl-sys bindings*: FFI, platform-dependent, complex.

### Decision 3: CA hierarchy — three-tier (root → intermediate → EE)

**Choice:** Generate a root CA, then intermediate CAs, then end-entity certs.

**Rationale:**
- Matches mitmproxy's architecture and browser trust models
- Root CA is stored offline (on disk, never leaves the machine)
- Intermediate CA signs EE certs — if compromised, only EE certs are affected
- Path length 0 on intermediate CA prevents further sub-CAs
- Browsers/OS trust stores typically trust root CAs, not intermediates

**Alternatives considered:**
- *Single root CA signing EE certs directly*: Simpler but less secure (root key exposed more often).
- *Flat cert store (no hierarchy)*: Doesn't work with browser trust chains.

### Decision 4: Store — Arc<Mutex<CertStore>> with LRU

**Choice:** `Arc<tokio::sync::RwLock<CertStore>>` with a simple LRU eviction policy.

**Rationale:**
- `Arc` allows sharing across async tasks (SNI dispatch runs in worker tasks)
- `RwLock` allows concurrent reads (cert lookup is read-heavy) with exclusive writes (store mutations)
- LRU eviction prevents unbounded memory growth
- Simple implementation: `Vec` with access-time tracking, or `LinkedHashMap` from `indexmap`

**Alternatives considered:**
- *DashMap*: Concurrent HashMap — overkill for our size (typically <1000 certs).
- *Simple Mutex*: Fine but unnecessary contention for read-heavy workload.
- *No eviction*: Unbounded memory growth if many domains are intercepted.

### Decision 5: Persistence — PEM files in directory

**Choice:** Store each cert and key as PEM files in a directory structure.

**Rationale:**
- PEM is human-readable and editable — useful for debugging
- Directory structure mirrors the certificate hierarchy
- Atomic save (write temp + rename) prevents corruption
- Easy to inspect: `ls ~/.mitmproxy-rs/certs/` shows all certs

**Structure:**
```
~/.mitmproxy-rs/
├── ca_root.pem          # Root CA certificate
├── ca_key.pem           # Root CA private key
├── intermediate.pem     # Intermediate CA certificate
├── intermediate_key.pem # Intermediate CA private key
└── certs/
    ├── example.com.pem
    ├── example.com_key.pem
    └── ...
```

**Alternatives considered:**
- *Single SQLite database*: Simpler lookup but adds dependency and complexity.
- *JSON metadata + DER bytes*: Not human-readable.
- *In-memory only*: Loses certs across restarts (annoying for developers).

### Decision 6: TLS config — rustls ServerConfig / ClientConfig wrappers

**Choice:** Thin wrappers around `rustls::ServerConfig` and `rustls::ClientConfig`.

**Rationale:**
- `rustls` is already in our dependency tree
- Wrappers provide our API surface (SNI dispatch, ALPN, verification) while delegating to rustls
- Keep our types independent of rustls internals where possible

**Alternatives considered:**
- *Direct rustls types*: Leaks dependency into our API. Harder to swap TLS implementation later.
- *Custom TLS state machine*: Massive effort, reinventing rustls.

## Risks / Trade-offs

[Risk: rcgen has limited CA hierarchy support] → [Mitigation: Implement the hierarchy manually using ring for signing. rcgen for simple EE certs is fine; for CA signing, use ring's sign API directly.]

[Risk: x509-parser's 'static lifetime requires memory leaks] → [Mitigation: Use `Box::leak()` or `Vec::into_boxed_slice().leak()` for parsed data. Acceptable since certs are long-lived.]

[Risk: Large number of concurrent TLS handshakes stresses cert store] → [Mitigation: RwLock + LRU cache. Typical mitmproxy handles thousands of concurrent TLS connections without issues.]

[Risk: CA key compromise if disk persistence is not secure] → [Mitigation: File permissions 0600 on Unix, 0600 equivalent on Windows. Document that the directory should not be shared.]

[Trade-off: RSA vs EC key types] → [Mitigation: Support both. Default to EC P-256 (smaller certs, faster handshake). RSA 2048 as fallback for compatibility.]

[Trade-off: On-disk persistence adds I/O latency on startup] → [Mitigation: Load certs async on startup. CA keys loaded first (needed for EE cert generation), then EE certs in parallel.]

## Migration Plan

This is a greenfield implementation — no migration from Python code needed.

1. **Phase 1 (this change):** Implement certificate types, store, and config in `mitm-certs`
2. **Phase 2:** Wire up TLS server config in `mitm-proxy` (future change)
3. **Phase 3:** Implement actual TLS interception using rustls (future change)

Rollback for this phase: delete the `mitm-certs` crate and revert `mitm-core` to placeholder Cert. No data loss — the on-disk store is in `~/.mitmproxy-rs/` which can be deleted.

## Open Questions

1. **Should we support cert preview/reload without restart?** Python mitmproxy supports reloading CA certs on SIGHUP. For v1, we load certs at startup only. Add signal-based reload in v2.

2. **Should the CA key be encrypted on disk?** Python mitmproxy stores the CA key unencrypted. For v1, we do the same. Add passphrase encryption in v2 if needed.

3. **Should we support custom root CA (user-provided)?** Python mitmproxy allows importing a custom CA. For v1, we auto-generate. Add import in v2.

4. **What about certificate pinning bypass?** Not relevant for a MITM proxy — we ARE the CA.

5. **Should end-entity certs include OCSP Must-Staple?** Python mitmproxy doesn't include it. For v1, skip. Add in v2 if needed.
