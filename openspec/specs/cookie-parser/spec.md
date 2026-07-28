## ADDED Requirements

### Requirement: Parse Set-Cookie header
The system MUST parse Set-Cookie header values according to RFC 6265.

#### Scenario: Parse simple cookie
- **WHEN** input is "name=value"
- **THEN** system extracts name="name", value="value"

#### Scenario: Parse cookie with attributes
- **WHEN** input is "name=value; Path=/; HttpOnly; Secure"
- **THEN** system extracts name="name", value="value", path="/", http_only=true, secure=true

#### Scenario: Parse cookie with expiration
- **WHEN** input is "name=value; Expires=Wed, 09 Jun 2021 10:18:14 GMT"
- **THEN** system extracts name="name", value="value", expires=Some(DateTime)

#### Scenario: Parse cookie with domain
- **WHEN** input is "name=value; Domain=.example.com; Path=/"
- **THEN** system extracts name="name", value="value", domain=".example.com", path="/"

### Requirement: Parse Cookie header
The system MUST parse Cookie header values (name=value pairs separated by semicolons).

#### Scenario: Parse single cookie
- **WHEN** input is "name=value"
- **THEN** system extracts cookie name="name", value="value"

#### Scenario: Parse multiple cookies
- **WHEN** input is "name1=value1; name2=value2; name3=value3"
- **THEN** system extracts three cookies: name1=value1, name2=value2, name3=value3

#### Scenario: Parse cookie with empty value
- **WHEN** input is "name="
- **THEN** system extracts name="name", value=""

### Requirement: Generate Set-Cookie header
The system MUST generate Set-Cookie header values from cookie components.

#### Scenario: Generate simple cookie
- **WHEN** system has name="name", value="value"
- **THEN** system generates "name=value"

#### Scenario: Generate cookie with attributes
- **WHEN** system has name="name", value="value", path="/", http_only=true, secure=true
- **THEN** system generates "name=value; Path=/; HttpOnly; Secure"

### Requirement: Match cookies to requests
The system MUST determine which cookies should be sent with a given request.

#### Scenario: Match cookies by domain and path
- **WHEN** request is to "http://example.com/path" and cookies have domains ".example.com" with paths "/" and "/path"
- **THEN** system returns cookies that match the request domain and path prefix

#### Scenario: Exclude expired cookies
- **WHEN** cookies include an expired cookie (expires in past)
- **THEN** system excludes expired cookies from match result
