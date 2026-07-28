## ADDED Requirements

### Requirement: In-memory certificate store

The system SHALL provide an in-memory store for certificates and keys:

- Store CA root, intermediate CA, and per-domain end-entity certificates
- Store associated private keys alongside certificates
- Thread-safe access via `Arc<CertStore>`
- Fast lookup by domain name (hostname) for SNI dispatch
- Fast lookup by certificate fingerprint for certificate identity

#### Scenario: Store and retrieve by domain

- **WHEN** a cert for "example.com" is stored and later retrieved with `store.get("example.com")`
- **THEN** the certificate and private key are returned

#### Scenario: Store and retrieve by fingerprint

- **WHEN** a cert's SHA-256 fingerprint is computed and used with `store.get_by_fingerprint(fpr)`
- **THEN** the correct certificate is returned

### Requirement: LRU cache eviction

The system SHALL evict least-recently-used entries when the store exceeds its capacity:

- Default capacity: 1024 entries
- Capacity is configurable
- Eviction uses standard LRU policy (access time, not insertion time)
- Eviction removes both the certificate and its private key

#### Scenario: Eviction removes oldest entry

- **WHEN** the store is at capacity and a new entry is added
- **THEN** the least-recently-used entry is removed to make room

### Requirement: On-disk persistence

The system SHALL persist certificates and keys to disk:

- Directory-based storage: one PEM file per certificate, one PEM file per key
- Directory structure: `{ca_root.pem, ca_key.pem, intermediates/{ca.pem, ca_key.pem}, certs/{domain.pem, domain_key.pem}}`
- Load on startup: scan directory and populate in-memory store
- Save on change: write modified entries back to disk atomically (write to temp, then rename)

#### Scenario: Persist and reload

- **WHEN** a cert is stored, then the store is dropped and reloaded from the same directory
- **THEN** the cert is available by domain name

#### Scenario: Atomic save prevents corruption

- **WHEN** a save is interrupted mid-write
- **THEN** the original file on disk remains intact (no partial writes)

### Requirement: Key pair generation

The system SHALL generate cryptographic key pairs:

- RSA: 2048-bit or 4096-bit (configurable, default 2048)
- EC: P-256 (secp256r1) or P-384 (secp384r1) (configurable, default P-256)
- Keys stored in PKCS#8 format (unencrypted)
- Private keys MUST be protected: file permissions 0600 on Unix

#### Scenario: Generate RSA key pair

- **WHEN** `KeyPair::generate_rsa(bits=2048)` is called
- **THEN** a valid RSA key pair is returned with private key in PKCS#8 PEM

#### Scenario: Generate EC key pair

- **WHEN** `KeyPair::generate_ec(curve=P256)` is called
- **THEN** a valid EC key pair using P-256 is returned

### Requirement: Certificate fingerprinting

The system SHALL compute certificate fingerprints:

- SHA-256 fingerprint (primary, used for identification)
- SHA-1 fingerprint (legacy, for compatibility)
- Fingerprint format: colon-separated hex bytes (e.g., "AB:CD:EF:...")

#### Scenario: SHA-256 fingerprint

- **WHEN** `cert.fingerprint_sha256()` is called
- **THEN** a 64-character hex string in colon-separated format is returned

#### Scenario: Fingerprint is deterministic

- **WHEN** the fingerprint is computed twice on the same certificate
- **THEN** both results are identical
