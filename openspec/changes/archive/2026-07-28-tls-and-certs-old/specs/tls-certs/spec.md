## ADDED Requirements

### Requirement: CA root certificate generation

The system SHALL generate a self-signed CA root certificate with:

- RSA 2048-bit or EC P-256 key pair
- X.509 v3 certificate
- Basic Constraints: CA=TRUE, pathlen=0
- Key Usage: keyCertSign, cRLSign
- Subject: configurable Common Name (default: "mitmproxy-rs CA")
- Validity: configurable (default: 10 years)
- Serial number: random 64-bit unsigned integer

#### Scenario: Generate CA root certificate

- **WHEN** `CaRoot::generate(cn, validity)` is called
- **THEN** a self-signed X.509 certificate with CA=TRUE is returned along with its private key

#### Scenario: CA root serial number is unique

- **WHEN** two CA roots are generated in sequence
- **THEN** their serial numbers differ

### Requirement: Intermediate CA certificate generation

The system SHALL generate an intermediate CA certificate signed by the root CA:

- Same key type as root (RSA or EC)
- Basic Constraints: CA=TRUE, pathlen=0
- Key Usage: keyCertSign, cRLSign
- Subject: configurable Common Name (default: "{cn} Intermediate CA")
- Validity: configurable (default: 5 years)
- Issuer: the root CA

#### Scenario: Intermediate CA is signed by root

- **WHEN** `IntermediateCa::generate(root, cn, validity)` is called
- **THEN** the returned certificate's issuer matches the root CA's subject

### Requirement: End-entity certificate generation

The system SHALL generate end-entity (server) certificates signed by an intermediate CA:

- Same key type as the signing CA
- Basic Constraints: CA=FALSE
- Key Usage: digitalSignature, keyEncipherment
- Extended Key Usage: serverAuth
- Subject Alternative Names (SANs): list of DNS names and/or IP addresses
- Serial number: random 64-bit unsigned integer
- Validity: configurable (default: 398 days per Mozilla policies)

#### Scenario: Generate cert for single domain

- **WHEN** `EndEntityCert::generate(intermediate_ca, cn, validity, sans)` is called with `sans = ["example.com"]`
- **THEN** the certificate has a DNS SAN for "example.com" and is signed by the intermediate CA

#### Scenario: Generate cert for multiple domains

- **WHEN** `EndEntityCert::generate(intermediate_ca, "example.com", 398.days, ["example.com", "www.example.com", "127.0.0.1"])` is called
- **THEN** the certificate has SANs for all three names (2 DNS + 1 IP)

#### Scenario: Generate cert with IP SAN

- **WHEN** `EndEntityCert::generate(intermediate_ca, "localhost", 398.days, ["localhost", "127.0.0.1", "::1"])` is called
- **THEN** the certificate has SANs including IPv4 and IPv6 addresses

### Requirement: Certificate chain building

The system SHALL build a certificate chain from end-entity to root:

- Chain order: [end-entity, intermediate_ca, root_ca]
- Each certificate verifies the signature of the preceding one
- The chain is suitable for TLS ServerHello certificate_list

#### Scenario: Build full chain

- **WHEN** `chain = build_chain(end_entity, intermediate_ca, root_ca)` is called
- **THEN** chain has 3 certificates in order: end-entity → intermediate → root

#### Scenario: Chain is verifiable

- **WHEN** the chain is verified against the root CA
- **THEN** verification succeeds (end-entity → intermediate → root)

### Requirement: Certificate serialization

The system SHALL serialize certificates and keys in standard formats:

- PEM encoding for certificates (RFC 7468)
- PEM encoding for private keys (PKCS#8 unencrypted)
- DER encoding for binary representation
- `to_pem() -> Result<String>` for certificate PEM
- `to_der() -> Result<Vec<u8>>` for certificate DER
- `private_key_pem() -> Result<String>` for key PEM

#### Scenario: PEM round-trip

- **WHEN** a certificate is serialized to PEM and deserialized back
- **THEN** the parsed certificate matches the original

#### Scenario: DER output is valid X.509

- **WHEN** `cert.to_der()` is called
- **THEN** the returned bytes parse as a valid X.509 certificate via x509-parser

## REMOVED Requirements

### Requirement: Placeholder Cert type

**Reason**: Replaced by real certificate types in tls-certs and tls-store capabilities
**Migration**: Use `x509_cert: X509Certificate` from x509-parser crate directly, or the wrapper types in mitm-certs
