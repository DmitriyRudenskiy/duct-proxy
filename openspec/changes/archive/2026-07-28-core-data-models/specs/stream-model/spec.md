## ADDED Requirements

### Requirement: TCP message type

The system SHALL define a `TCPMessage` struct:

- `from_client: bool` — `true` if sent by the client, `false` if sent by the server
- `content: Vec<u8>` — the message payload bytes
- `timestamp: f64` — Unix timestamp of when the message was sent/received

TCP is a stream protocol — message boundaries are conceptual chunks, not protocol-enforced.

#### Scenario: TCP message tracks direction

- **WHEN** a byte sequence arrives from the client
- **THEN** a `TCPMessage` is created with `from_client = true`

#### Scenario: TCP message defaults timestamp to now

- **WHEN** a `TCPMessage` is created without an explicit timestamp
- **THEN** `timestamp` defaults to the current Unix time

### Requirement: UDP message type

The system SHALL define a `UDPMessage` struct structurally identical to `TCPMessage`:

- `from_client: bool`
- `content: Vec<u8>`
- `timestamp: f64`

#### Scenario: UDP message tracks direction

- **WHEN** a UDP datagram arrives
- **THEN** a `UDPMessage` is created with the correct `from_client` flag

### Requirement: TCP flow

The system SHALL define a `TCPFlow` struct containing:

- `base: FlowBase` — shared flow state
- `messages: Vec<TCPMessage>` — ordered list of all TCP messages in the session

The latest message MUST be accessible as `messages.last()`.

#### Scenario: TCP messages accumulate

- **WHEN** data flows in both directions on a TCP connection
- **THEN** each chunk is appended to `flow.messages`

#### Scenario: TCPFlow is a Flow variant

- **WHEN** a TCP connection is detected
- **THEN** it is represented as `Flow::Tcp(TCPFlow)`

### Requirement: UDP flow

The system SHALL define a `UDPFlow` struct containing:

- `base: FlowBase` — shared flow state
- `messages: Vec<UDPMessage>` — ordered list of all UDP datagrams in the session

The latest message MUST be accessible as `messages.last()`.

#### Scenario: UDP messages accumulate

- **WHEN** UDP datagrams are exchanged
- **THEN** each datagram is appended to `flow.messages`

#### Scenario: UDPFlow is a Flow variant

- **WHEN** a UDP connection is detected
- **THEN** it is represented as `Flow::Udp(UDPFlow)`
