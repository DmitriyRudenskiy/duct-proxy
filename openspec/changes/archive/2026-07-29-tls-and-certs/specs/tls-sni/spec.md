## ADDED Requirements

### Requirement: SNI extraction from ClientHello

The system SHALL extract Server Name Indication (SNI) from raw TLS ClientHello bytes:

- Parse TLS ClientHello structure (RFC 8446 Section 4.1.2)
- Extract extension type 0x00 (server_name)
- Validate SNI is a valid hostname (RFC 1123)
- Return `Option<String>` (None if no SNI or invalid)

#### Scenario: Extract valid SNI

- **WHEN** ClientHello with SNI "example.com" is parsed
- **THEN** `extract_sni(bytes)` returns `Some("example.com")`

#### Scenario: No SNI returns None

- **WHEN** ClientHello without SNI extension is parsed
- **THEN** `extract_sni(bytes)` returns `None`

#### Scenario: Invalid SNI returns None

- **WHEN** ClientHello with invalid hostname (e.g., empty string) is parsed
- **THEN** `extract_sni(bytes)` returns `None`
