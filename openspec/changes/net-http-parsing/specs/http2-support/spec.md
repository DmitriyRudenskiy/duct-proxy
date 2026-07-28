## ADDED Requirements

### Requirement: Handle HTTP/2 connection preface
The system MUST handle the HTTP/2 connection preface (CLIENT_PRIOR_KNOWLEDGE or TLS negotiation).

#### Scenario: Detect HTTP/2 cleartext connection
- **WHEN** client sends "PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n" as first bytes
- **THEN** system recognizes this as HTTP/2 cleartext connection preface

#### Scenario: Detect HTTP/2 over TLS
- **WHEN** TLS handshake includes ALPN protocol "h2"
- **THEN** system negotiates HTTP/2 protocol

### Requirement: Parse HTTP/2 frames
The system MUST parse HTTP/2 frames according to RFC 7540.

#### Scenario: Parse HEADERS frame
- **WHEN** input contains a valid HEADERS frame (9-byte header + payload)
- **THEN** system extracts frame type, flags, stream ID, and payload

#### Scenario: Parse DATA frame
- **WHEN** input contains a valid DATA frame
- **THEN** system extracts frame type, flags, stream ID, and data payload

#### Scenario: Parse SETTINGS frame
- **WHEN** input contains a valid SETTINGS frame
- **THEN** system extracts frame type, flags, stream ID, and settings parameters

### Requirement: Support HTTP/2 via h2 crate
The system MUST use the h2 crate for HTTP/2 connection management.

#### Scenario: Create HTTP/2 connection
- **WHEN** system creates an h2::Connection with a TcpStream
- **THEN** connection can accept incoming streams

#### Scenario: Handle HTTP/2 streams
- **WHEN** incoming HTTP/2 stream arrives
- **THEN** system can receive request headers and body on the stream

### Requirement: Convert HTTP/2 to HTTP/1.1 for upstream
The system MUST convert HTTP/2 requests to HTTP/1.1 for upstream forwarding.

#### Scenario: Convert HTTP/2 request to HTTP/1.1
- **WHEN** system receives HTTP/2 request with :method, :path, :scheme, :authority pseudo-headers
- **THEN** system generates HTTP/1.1 request with method, URI, and Host header

#### Scenario: Forward HTTP/2 response as HTTP/1.1
- **WHEN** system receives HTTP/2 response from upstream
- **THEN** system converts response to HTTP/1.1 format for client
