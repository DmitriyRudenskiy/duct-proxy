## ADDED Requirements

### Requirement: Parse URL components
The system MUST parse a URL string into its components: scheme, host, port, path, query.

#### Scenario: Parse full URL
- **WHEN** input is "https://example.com:8443/path?q=1#frag"
- **THEN** system extracts scheme="https", host="example.com", port=8443, path="/path", query="q=1"

#### Scenario: Parse URL with default port
- **WHEN** input is "http://example.com/path"
- **THEN** system extracts scheme="http", host="example.com", port=80 (default), path="/path"

#### Scenario: Parse URL with IPv6 host
- **WHEN** input is "http://[::1]:8080/path"
- **THEN** system extracts scheme="http", host="[::1]", port=8080, path="/path"

### Requirement: Validate URL components
The system MUST validate URL components for correctness.

#### Scenario: Validate valid URL
- **WHEN** input is "https://example.com/path"
- **THEN** system returns Ok with parsed components

#### Scenario: Reject invalid scheme
- **WHEN** input is "not-a-scheme://example.com"
- **THEN** system returns error for invalid scheme

#### Scenario: Reject missing host
- **WHEN** input is "http:///path"
- **THEN** system returns error for missing host

### Requirement: Reconstruct URL from components
The system MUST reconstruct a URL string from parsed components.

#### Scenario: Reconstruct URL
- **WHEN** system has scheme="https", host="example.com", port=443, path="/path"
- **THEN** system generates "https://example.com:443/path"

#### Scenario: Reconstruct URL with query
- **WHEN** system has scheme="http", host="example.com", path="/search", query="q=1&r=2"
- **THEN** system generates "http://example.com/search?q=1&r=2"

### Requirement: Extract host from SNI
The system MUST extract the hostname from a TLS SNI value for URL construction.

#### Scenario: Extract hostname from SNI
- **WHEN** SNI is "example.com"
- **THEN** system returns host="example.com"

#### Scenario: Extract hostname with port
- **WHEN** SNI is "example.com:8443"
- **THEN** system returns host="example.com", port=8443
