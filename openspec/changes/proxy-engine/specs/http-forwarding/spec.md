## ADDED Requirements

### Requirement: Parse HTTP request line
The HTTP layer SHALL parse the request line (method, URI, version) from incoming HTTP requests.

#### Scenario: Parse GET request
- **WHEN** client sends "GET /path HTTP/1.1\r\n"
- **THEN** parser extracts method=GET, uri=/path, version=HTTP/1.1

#### Scenario: Parse POST request with body
- **WHEN** client sends "POST /api HTTP/1.1\r\nContent-Length: 5\r\n\r\nhello"
- **THEN** parser extracts method=POST, uri=/api, version=HTTP/1.1, body="hello"

### Requirement: Parse HTTP response line
The HTTP layer SHALL parse the response line (version, status code, reason) from server responses.

#### Scenario: Parse 200 OK response
- **WHEN** server sends "HTTP/1.1 200 OK\r\n"
- **THEN** parser extracts version=HTTP/1.1, status=200, reason="OK"

#### Scenario: Parse 404 response
- **WHEN** server sends "HTTP/1.1 404 Not Found\r\n"
- **THEN** parser extracts version=HTTP/1.1, status=404, reason="Not Found"

### Requirement: Forward HTTP requests to upstream
The HTTP layer SHALL forward parsed HTTP requests to the upstream server and return the response.

#### Scenario: Forward GET request
- **WHEN** client sends GET request for https://example.com
- **THEN** proxy forwards request to example.com:443 and returns server response

#### Scenario: Forward POST with body
- **WHEN** client sends POST request with body
- **THEN** proxy forwards method, headers, and body to upstream

### Requirement: Handle CONNECT tunnel
The HTTP layer SHALL establish a tunnel for CONNECT requests by sending 200 OK and then transparently forwarding data.

#### Scenario: Respond 200 to CONNECT
- **WHEN** client sends "CONNECT example.com:443 HTTP/1.1\r\n"
- **THEN** proxy responds "HTTP/1.1 200 Connection established\r\n\r\n" and begins tunneling

#### Scenario: Tunnel forwards bytes bidirectionally
- **WHEN** tunnel is established
- **THEN** bytes from client are sent to server and vice versa until disconnect
