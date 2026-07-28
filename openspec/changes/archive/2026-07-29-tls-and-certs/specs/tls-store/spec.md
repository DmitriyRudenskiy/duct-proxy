## ADDED Requirements

### Requirement: Certificate store with LRU cache

The system SHALL provide an in-memory certificate store:

- Thread-safe access via `Arc<CertStore>`
- LRU eviction when capacity exceeded (default: 1024 entries)
- Fast lookup by domain name (for SNI dispatch)
- Fast lookup by certificate fingerprint

#### Scenario: Store and retrieve by domain

- **WHEN** a cert for "example.com" is stored and retrieved with `store.get("example.com")`
- **THEN** the certificate and key are returned

#### Scenario: LRU eviction

- **WHEN** store exceeds capacity and new entry is added
- **THEN** least-recently-used entry is evicted

### Requirement: Certificate fingerprinting

The system SHALL compute SHA-256 fingerprints:

- Format: colon-separated hex bytes (e.g., "AB:CD:EF:...")
- Deterministic for same certificate

#### Scenario: Fingerprint is deterministic

- **WHEN** fingerprint is computed twice on same cert
- **THEN** results are identical

#### Scenario: Different certs have different fingerprints

- **WHEN** fingerprints are computed on two different certs
- **THEN** results differ
