## ADDED Requirements

### Requirement: On-the-fly leaf certificate generation

The system SHALL generate leaf certificates signed by the CA:

- Key type: ECDSA P-256
- X.509 v3 certificate
- Basic Constraints: CA=FALSE
- Key Usage: digitalSignature, keyEncipherment
- Extended Key Usage: serverAuth
- Subject Alternative Names (SANs): DNS names from SNI
- Validity: 398 days (Mozilla policy)
- Serial number: random 64-bit

#### Scenario: Generate leaf for domain

- **WHEN** `LeafCert::generate(ca, "example.com")` is called
- **THEN** certificate with DNS SAN "example.com" is returned

#### Scenario: Leaf cert is signed by CA

- **WHEN** leaf cert is verified against CA
- **THEN** verification succeeds

### Requirement: Leaf certificate cache

The system SHALL cache generated leaf certificates:

- Key: domain name
- Value: (certificate, private key, generation timestamp)
- No expiration (regenerate only if explicitly removed)

#### Scenario: Cached leaf is reused

- **WHEN** leaf for "example.com" is generated twice
- **THEN** second call returns cached cert (same fingerprint)
