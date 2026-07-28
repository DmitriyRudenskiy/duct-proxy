## ADDED Requirements

### Requirement: Detect TLS ClientHello by first byte
The protocol detector SHALL identify TLS connections by checking if the first byte is 0x16 (Handshake record type).

#### Scenario: TLS connection identified
- **WHEN** first byte of incoming data is 0x16
- **THEN** protocol is classified as TLS and handler routes to TLS interceptor

#### Scenario: Non-TLS connection not misidentified
- **WHEN** first byte is 0x47 ('G' from GET request)
- **THEN** protocol is NOT classified as TLS

### Requirement: Detect HTTP CONNECT by method prefix
The protocol detector SHALL identify HTTP CONNECT requests by checking if the first bytes match "CONNECT " (case-insensitive).

#### Scenario: CONNECT request identified
- **WHEN** first 8 bytes match "CONNECT " (case-insensitive)
- **THEN** protocol is classified as HTTP CONNECT and routed to tunnel handler

#### Scenario: Other HTTP methods not misidentified as CONNECT
- **WHEN** first bytes match "GET /"
- **THEN** protocol is classified as HTTP GET, not CONNECT

### Requirement: Detect HTTP request by method
The protocol detector SHALL identify HTTP requests by checking if the first bytes match a valid HTTP method.

#### Scenario: HTTP GET identified
- **WHEN** first bytes match "GET "
- **THEN** protocol is classified as HTTP

#### Scenario: HTTP POST identified
- **WHEN** first bytes match "POST "
- **THEN** protocol is classified as HTTP

### Requirement: Default to raw TCP for unrecognized data
The protocol detector SHALL classify connections as raw TCP when no known protocol pattern is detected.

#### Scenario: Unrecognized first bytes become raw TCP
- **WHEN** first bytes don't match TLS (0x16), HTTP methods, or CONNECT
- **THEN** protocol is classified as Raw TCP and routed to raw handler
