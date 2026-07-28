## Context

mitmproxy-rs needs certificate management for TLS interception. The `mitm-certs` crate is empty, and `mitm-core::Cert` is a placeholder (`Vec<u8>`). The Python reference uses `cryptography` library for all cert operations.

For Rust v1, we focus on:
- ECDSA P-256 only (simpler than supporting RSA + EC)
- SNI-based leaf cert generation
- PEM persistence at `~/.mitmproxy/`
- LRU cert store for SNI dispatch

## Goals / Non-Goals

**Goals:**
- Generate CA root with ECDSA P-256
- Generate leaf certs on-the-fly signed by CA
- Extract SNI from raw ClientHello bytes
- Cache certs in memory with LRU eviction
- Persist CA to `~/.mitmproxy/`

**Non-Goals:**
- RSA key support (v1 is ECDSA only)
- Certificate revocation (CRL/OCSP)
- Custom CA import (user-provided)
- CT log integration
- Hardware security module (HSM)

## Decisions

### Decision 1: ECDSA P-256 only for v1

**Choice:** Single key type (ECDSA P-256).

**Rationale:**
- Smaller certs (65 bytes vs 256 bytes for RSA 2048)
- Faster handshake
- Modern browsers support it universally
- Simpler codebase for v1

**Alternatives:**
- *RSA 2048*: Larger certs, slower, but universal compatibility. Not needed for v1.
- *Both*: Double the code, no benefit for v1.

### Decision 2: rcgen for cert generation

**Choice:** Use `rcgen` crate for certificate generation.

**Rationale:**
- Simple API for generating self-signed and CA-signed certs
- Supports SANs, key types, validity
- Well-maintained, used by many projects

**Alternatives:**
- *ring + manual X.509 construction*: More control but much more code.
- *x509-parser + ring*: x509-parser is read-only, ring for signing. More complex.

### Decision 3: x509-parser for cert parsing

**Choice:** Use `x509-parser` for parsing certificates.

**Rationale:**
- Fast, safe (no unsafe code)
- Standard Rust X.509 parser
- Returns structured data (not raw bytes)

**Alternatives:**
- *rcgen parsed types*: rcgen has some parsing but limited.
- *Raw DER parsing*: Too low-level, error-prone.

### Decision 4: LRU store with indexmap

**Choice:** Use `indexmap::IndexMap` for LRU cache.

**Rationale:**
- `indexmap` provides `IndexMap` with `swap_remove` for O(1) LRU eviction
- Simpler than implementing LRU from scratch
- No extra dependency (indexmap is already in our tree via other crates)

**Alternatives:**
- *Custom LRU with Vec*: O(n) eviction, too slow for many certs.
- *DashMap*: Overkill for our size (<1000 certs typically).

### Decision 5: PEM persistence at ~/.mitmproxy/

**Choice:** Directory-based PEM storage.

**Rationale:**
- Human-readable, easy to inspect
- Standard format (PEM)
- Matches mitmproxy's approach

**Structure:**
```
~/.mitmproxy/
├── ca_root.pem
└── ca_key.pem
```

**Alternatives:**
- *SQLite*: Simpler lookup but adds dependency.
- *Single binary file*: Not human-readable.

### Decision 6: SNI parsing with manual TLS parsing

**Choice:** Manual TLS ClientHello parsing (no external crate).

**Rationale:**
- ClientHello structure is well-defined (RFC 8446)
- Only need to extract extension 0x00 (server_name)
- External crates (kaitaistruct) add complexity for simple extraction

**Implementation:**
- Parse TLS record header (5 bytes)
- Parse handshake header (4 bytes)
- Parse ClientHello structure
- Find extension type 0x00
- Extract hostname

**Alternatives:**
- *kaitaistruct*: Used by Python mitmproxy, but adds dependency.
- *rustls parser*: Internal API, not stable.

## Risks / Trade-offs

[Risk: rcgen doesn't support all X.509 extensions] → [Mitigation: For v1, we only need basic extensions (CA=TRUE, SANs). rcgen supports these.]

[Risk: ECDSA P-256 not supported by old clients] → [Mitigation: TLS 1.3 requires ECDSA or RSA. Most modern clients support P-256. For v1, this is acceptable.]

[Risk: LRU eviction removes certs prematurely] → [Mitigation: Default capacity 1024 is enough for typical usage. Can increase if needed.]

[Risk: SNI parsing fails on malformed ClientHello] → [Mitigation: Return None on parse errors. Don't panic.]

[Trade-off: PEM vs DER for persistence] → [Mitigation: PEM is human-readable, DER is binary. PEM for v1, can add DER export later.]

## Migration Plan

1. **Phase 1 (this change):** Implement CA, store, SNI, leaf certs in `mitm-certs`
2. **Phase 2:** Wire up TLS server config in `mitm-proxy` (future)
3. **Phase 3:** Implement TLS interception (future)

Rollback: delete `mitm-certs` crate and revert `mitm-core` Cert type.

## Open Questions

1. **Should we support cert regeneration?** Python mitmproxy regenerates if cert is compromised. For v1, no regeneration (delete and regenerate manually).

2. **Should we log cert generation?** Python mitmproxy logs cert generation. For v1, use `tracing` for visibility.

3. **What about cert chain?** Python mitmproxy sends full chain (leaf + intermediate + root). For v1, send leaf only (most clients accept it).
