## ADDED Requirements

### Requirement: TLS server configuration

The system SHALL define a `TlsServerConfig` for TLS server (listening) sockets:

- Certificate and private key for the served domain
- Certificate chain (intermediate + root) for chain validation
- Supported TLS versions: TLS 1.2 and TLS 1.3 (TLS 1.0/1.1 disabled)
- Cipher suite configuration: prefer modern AEAD ciphers
- ALPN protocol negotiation: support "h2" and "http/1.1"
- Session ticket support for TLS 1.2 resumption
- OCSP stapling: optional, disabled by default

#### Scenario: Server config supports TLS 1.2 and 1.3

- **WHEN** a client connects with TLS 1.2 ClientHello
- **THEN** the server accepts the connection with a negotiated TLS 1.2 session

#### Scenario: Server config negotiates ALPN

- **WHEN** a client offers ALPN "h2,http/1.1"
- **THEN** the server negotiates the best common protocol (h2 preferred)

#### Scenario: Server config rejects TLS 1.1

- **WHEN** a client offers only TLS 1.1
- **THEN** the server rejects the handshake (protocol mismatch)

### Requirement: SNI-based certificate selection

The system SHALL select certificates based on the Server Name Indication (SNI) from the ClientHello:

- Match SNI against stored domain names
- If no match, use a default/fallback certificate
- Fallback certificate should be for a generic domain (e.g., "*.mitmproxy.local")
- SNI matching is case-insensitive for DNS names

#### Scenario: SNI matches stored cert

- **WHEN** a ClientHello has SNI "example.com" and a cert exists for "example.com"
- **THEN** the server presents the "example.com" certificate

#### Scenario: SNI has no matching cert

- **WHEN** a ClientHello has SNI "unknown.local" and no cert exists
- **THEN** the server presents the fallback certificate

### Requirement: TLS client configuration

The system SHALL define a `TlsClientConfig` for connecting to upstream servers:

- Certificate verification: enabled by default (verify peer certificate chain)
- Custom CA certificates: allow adding trusted CA certificates for self-signed servers
- SNI: enabled by default (send server hostname)
- ALPN: configurable list of protocols to offer
- Certificate revocation: CRL check optional, OCSP optional

#### Scenario: Verify server certificate by default

- **WHEN** connecting to "example.com" with default config
- **THEN** the server certificate is verified against system trust store

#### Scenario: Accept self-signed server cert

- **WHEN** connecting to a server with a self-signed cert and a custom CA is configured
- **THEN** the connection succeeds if the cert chains to the custom CA

### Requirement: TLS handshake state tracking

The system SHALL track TLS handshake progress and expose state:

- `timestamp_tls_start`: when handshake began
- `timestamp_tls_complete`: when handshake finished
- `tls_version`: negotiated TLS version
- `cipher_suite`: negotiated cipher suite name
- `alpn_protocol`: negotiated ALPN protocol
- `server_certificates`: list of server certificates received
- `server_cert_fingerprint`: SHA-256 fingerprint of end-entity server cert

#### Scenario: Track handshake timing

- **WHEN** a TLS handshake completes
- **THEN** `timestamp_tls_start` and `timestamp_tls_complete` are set with correct values

#### Scenario: Track negotiated parameters

- **WHEN** a TLS 1.3 handshake with AES_256_GCM completes
- **THEN** `tls_version` is "TLSv1.3" and `cipher_suite` is "TLS_AES_256_GCM_SHA384"

### Requirement: Certificate transparency log support (optional)

The system MAY support CT log URL configuration for end-entity certificate generation:

- Store CT log URLs in certificate metadata
- Include SCT (Signed Certificate Timestamp) in certificate if available
- This is a soft requirement — implementation is not mandatory for v1

#### Scenario: CT URL stored in config

- **WHEN** a TLS config includes `ct_log_urls = ["https://ct.example.com"]`
- **THEN** the configuration accepts the URLs without error
