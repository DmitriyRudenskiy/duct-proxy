## ADDED Requirements

### Requirement: Decode Chunked Transfer-Encoding
The system MUST decode HTTP bodies with Chunked transfer-encoding.

#### Scenario: Decode simple chunked body
- **WHEN** input is "5\r\nHello\r\n0\r\n\r\n"
- **THEN** system extracts body="Hello"

#### Scenario: Decode multiple chunks
- **WHEN** input is "4\r\nSpac\r\n5\r\nes\r\n0\r\n\r\n"
- **THEN** system extracts body="Spaces"

#### Scenario: Decode chunked with trailers
- **WHEN** input is "5\r\nHello\r\n0\r\nTrailer-Name: value\r\n\r\n"
- **THEN** system extracts body="Hello" and trailers: Trailer-Name="value"

#### Scenario: Decode empty chunked body
- **WHEN** input is "0\r\n\r\n"
- **THEN** system extracts empty body

### Requirement: Encode Chunked Transfer-Encoding
The system MUST encode HTTP bodies with Chunked transfer-encoding.

#### Scenario: Encode simple body
- **WHEN** system has body="Hello"
- **THEN** system generates "5\r\nHello\r\n0\r\n\r\n"

#### Scenario: Encode empty body
- **WHEN** system has empty body
- **THEN** system generates "0\r\n\r\n"

### Requirement: Read Content-Length body
The system MUST read exactly Content-Length bytes from a stream for the body.

#### Scenario: Read body with known length
- **WHEN** Content-Length is 100 and stream contains 100 bytes
- **THEN** system reads exactly 100 bytes for the body

#### Scenario: Read empty body
- **WHEN** Content-Length is 0
- **THEN** system returns empty body without reading

#### Scenario: Handle missing Content-Length
- **WHEN** response has no Content-Length and no Transfer-Encoding
- **THEN** system reads until stream end (for HTTP/1.0 compatibility)

### Requirement: Determine body reading strategy
The system MUST determine how to read the body based on headers.

#### Scenario: Use chunked for chunked transfer
- **WHEN** Transfer-Encoding is "chunked"
- **THEN** system uses chunked decoder

#### Scenario: Use content-length for content-length
- **WHEN** Content-Length header is present
- **THEN** system reads exact number of bytes

#### Scenario: Use content-length for HTTP/2
- **WHEN** protocol is HTTP/2
- **THEN** system uses content-length (HTTP/2 has no chunked encoding)
