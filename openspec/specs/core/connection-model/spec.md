## ADDED Requirements

### Requirement: Connection base struct

The system SHALL define a `Connection` struct representing metadata about a network connection. The struct MUST NOT expose the underlying socket — all I/O is handled externally.

Fields:

- `id: String` — UUID4 unique identifier
- `peername: Option<(String, u16)>` — remote `(ip, port)` tuple
- `sockname: Option<(String, u16)>` — local `(ip, port)` tuple
- `state: ConnectionState` — current connection state (flag enum)
- `transport_protocol: String` — "tcp" or "udp"
- `error: Option<String>` — connection-level error (not per-flow)
- `tls: bool` — whether TLS should eventually be established
- `certificate_list: Vec<Cert>` — TLS certificate chain from peer
- `alpn: Option<Vec<u8>>` — negotiated ALPN protocol
- `alpn_offers: Vec<Vec<u8>>` — ALPN offers from ClientHello
- `cipher: Option<String>` — active cipher suite name
- `cipher_list: Vec<String>` — accepted cipher suites
- `tls_version: Option<String>` — TLS version string ("TLSv1.2", "TLSv1.3", etc.)
- `sni: Option<String>` — Server Name Indication
- `timestamp_start: Option<f64>` — connection start timestamp
- `timestamp_end: Option<f64>` — connection close timestamp
- `timestamp_tls_setup: Option<f64>` — TLS handshake completion timestamp

#### Scenario: Connection has unique ID

- **WHEN** a new Connection is created
- **THEN** it is assigned a UUID4 string

#### Scenario: Two connections with same ID are equal

- **WHEN** two Connection objects share the same `id`
- **THEN** they compare equal via `==`

### Requirement: Connection state flag enum

The system SHALL define `ConnectionState` as a flag enum:

- `Closed = 0`
- `CanRead = 1`
- `CanWrite = 2`
- `Open = CanRead | CanWrite = 3`

#### Scenario: Connection starts closed

- **WHEN** a new Connection is created
- **THEN** `state` is `ConnectionState::Closed`

#### Scenario: Connected property reflects Open state

- **WHEN** `Connection.connected` is checked
- **THEN** it returns `true` if and only if `state == ConnectionState::Open`

### Requirement: TLS establishment tracking

The system SHALL provide computed properties:

- `tls_established: bool` — returns `true` if `timestamp_tls_setup` is set
- `connected: bool` — returns `true` if `state == ConnectionState::Open`

#### Scenario: TLS established before any data

- **WHEN** a TLS connection completes handshake
- **THEN** `timestamp_tls_setup` is set and `tls_established` returns `true`

### Requirement: Client connection

The system SHALL define `Client` (a concrete Connection type) with additional fields:

- `peername: (String, u16)` — client address (not optional)
- `sockname: (String, u16)` — local accept address (not optional)
- `mitmcert: Option<Cert>` — certificate mitmproxy presented to client
- `proxy_mode: String` — proxy mode string (e.g., "regular", "transparent")
- `timestamp_start: f64` — defaults to current time (TCP SYN received)

#### Scenario: Client always has addresses

- **WHEN** a Client is created for an incoming connection
- **THEN** both `peername` and `sockname` are set (never `None`)

#### Scenario: Client timestamp defaults to creation time

- **WHEN** a new Client is constructed without explicit timestamp
- **THEN** `timestamp_start` defaults to the current Unix timestamp

### Requirement: Server connection

The system SHALL define `Server` (a concrete Connection type) with additional fields:

- `address: Option<(String, u16)>` — target `(host, port)`, may be `None`
- `peername: Option<(String, u16)>` — resolved IP address, may be `None` (upstream proxy)
- `sockname: Option<(String, u16)>` — local socket address
- `timestamp_start: Option<f64>` — connection attempt start
- `timestamp_tcp_setup: Option<f64>` — TCP ACK received
- `via: Option<ServerSpec>` — optional upstream proxy specification

The system SHALL enforce that `address` and `via` cannot be changed on an open connection.

#### Scenario: Server address cannot change on open connection

- **WHEN** a Server has `state == ConnectionState::Open` and code attempts to set `address`
- **THEN** the operation fails (panic or error)

#### Scenario: Server address may be None initially

- **WHEN** a Server is created for a flow where the target is unknown (e.g., upstream proxy mode)
- **THEN** `address` is `None` and can be set before connection

### Requirement: Connection serialization

All connection types SHALL be serializable to and deserializable from JSON:

- The serialized form MUST include all fields
- `certificate_list` entries MUST be serializable
- `timestamp_*` fields that are `None` MUST serialize as `null`

#### Scenario: Client connection round-trips

- **WHEN** a Client is serialized and deserialized
- **THEN** all fields including `peername`, `sockname`, `mitmcert` are preserved
