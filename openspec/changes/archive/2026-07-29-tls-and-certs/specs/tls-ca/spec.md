## ADDED Requirements

### Requirement: CA root certificate generation

The system SHALL generate a CA root certificate using ECDSA P-256:

- Key type: ECDSA P-256 (secp256r1)
- X.509 v3 certificate
- Basic Constraints: CA=TRUE, pathlen=0
- Key Usage: keyCertSign, cRLSign
- Subject Common Name: configurable (default: "mitmproxy-rs CA")
- Validity: 10 years
- Serial number: random 64-bit unsigned integer

#### Scenario: Generate CA root

- **WHEN** `CaRoot::generate(cn: &str)` is called
- **THEN** a self-signed X.509 certificate with ECDSA P-256 key is returned

#### Scenario: CA root has correct extensions

- **WHEN** the generated CA root is parsed
- **THEN** BasicConstraints has CA=TRUE and pathlen=0

### Requirement: CA key serialization

The system SHALL serialize the CA private key:

- Format: PKCS#8 unencrypted
- Encoding: PEM
- File permissions: 0600 on Unix

#### Scenario: Key serializes to PEM

- **WHEN** `ca_root.private_key_pem()` is called
- **THEN** valid PKCS#8 PEM string is returned

### Requirement: CA certificate persistence

The system SHALL persist CA root to disk:

- Directory: `~/.mitmproxy/`
- Files: `ca_root.pem`, `ca_key.pem`
- Load on startup: scan directory and load if present
- Save on first run or if missing

#### Scenario: Persist and load CA

- **WHEN** CA is saved to `~/.mitmproxy/` and loaded back
- **THEN** the loaded CA matches the original
