## ADDED Requirements

### Requirement: Parse HTTP/1.1 request line
The system MUST parse HTTP request lines in the format "METHOD URI HTTP/version".

#### Scenario: Parse GET request
- **WHEN** input is "GET /index.html HTTP/1.1\r\n"
- **THEN** system extracts method="GET", uri="/index.html", version="HTTP/1.1"

#### Scenario: Parse POST request with query string
- **WHEN** input is "POST /api/data?field=value HTTP/1.1\r\n"
- **THEN** system extracts method="POST", uri="/api/data?field=value", version="HTTP/1.1"

#### Scenario: Parse request with custom method
- **WHEN** input is "CUSTOM /resource HTTP/1.1\r\n"
- **THEN** system extracts method="CUSTOM", uri="/resource", version="HTTP/1.1"

### Requirement: Parse HTTP/1.1 response line
The system MUST parse HTTP response lines in the format "HTTP/version status reason".

#### Scenario: Parse 200 OK response
- **WHEN** input is "HTTP/1.1 200 OK\r\n"
- **THEN** system extracts version="HTTP/1.1", status=200, reason="OK"

#### Scenario: Parse 404 Not Found response
- **WHEN** input is "HTTP/1.1 404 Not Found\r\n"
- **THEN** system extracts version="HTTP/1.1", status=404, reason="Not Found"

#### Scenario: Parse 500 Internal Server Error response
- **WHEN** input is "HTTP/1.1 500 Internal Server Error\r\n"
- **THEN** system extracts version="HTTP/1.1", status=500, reason="Internal Server Error"

### Requirement: Parse HTTP headers
The system MUST parse HTTP headers from a stream until the empty line (\r\n\r\n).

#### Scenario: Parse single header
- **WHEN** input is "Content-Type: text/html\r\n\r\n"
- **THEN** system extracts header name="Content-Type", value="text/html"

#### Scenario: Parse multiple headers
- **WHEN** input is "Content-Type: text/html\r\nContent-Length: 100\r\n\r\n"
- **THEN** system extracts two headers: Content-Type and Content-Length

#### Scenario: Parse headers with multiple values
- **WHEN** input is "Set-Cookie: a=1\r\nSet-Cookie: b=2\r\n\r\n"
- **THEN** system extracts two Set-Cookie headers with values "a=1" and "b=2"

### Requirement: Parse streaming HTTP request
The system MUST parse HTTP requests from an AsyncRead stream without loading the entire request into memory.

#### Scenario: Parse request line first, then headers
- **WHEN** stream contains request line, then headers, then body
- **THEN** system returns request with parsed line and headers before reading body

#### Scenario: Parse request with no body
- **WHEN** stream contains request line, headers, and empty line (no body)
- **THEN** system returns request with empty body

### Requirement: Parse streaming HTTP response
The system MUST parse HTTP responses from an AsyncRead stream without loading the entire response into memory.

#### Scenario: Parse response line first, then headers
- **WHEN** stream contains response line, then headers, then body
- **THEN** system returns response with parsed line and headers before reading body

#### Scenario: Parse response with no body
- **WHEN** stream contains response line, headers, and empty line (no body)
- **THEN** system returns response with empty body

### Requirement: Represent HTTP messages
The system MUST provide types for representing parsed HTTP requests and responses.

#### Scenario: Create HTTP request
- **WHEN** system creates an HTTP request with method, uri, version, and headers
- **THEN** request can be serialized back to bytes

#### Scenario: Create HTTP response
- **WHEN** system creates an HTTP response with version, status, reason, and headers
- **THEN** response can be serialized back to bytes
