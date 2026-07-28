## ADDED Requirements

### Requirement: WebSocket message type

The system SHALL define a `WebSocketMessage` struct:

- `from_client: bool` — `true` if sent by the client
- `msg_type: WebSocketOpcode` — message opcode (TEXT, BINARY, CLOSE, PING, PONG)
- `content: Vec<u8>` — message payload (always bytes, even for text frames)
- `timestamp: f64` — when the message was received
- `dropped: bool` — `true` if the message was dropped (not forwarded)
- `injected: bool` — `true` if the message was injected by a hook (not from either peer)

Fragmented WebSocket messages SHALL be reassembled into a single `WebSocketMessage`.

The system SHALL provide:

- `is_text: bool` — `true` if opcode is TEXT
- `text: String` — decoded text content (only on TEXT frames; panics/errors on BINARY)
- `drop()` — marks the message as dropped
- `kill()` — deprecated alias for `drop()`

#### Scenario: WebSocket text message is identifiable

- **WHEN** a WebSocketMessage has opcode TEXT
- **THEN** `msg.is_text` returns `true` and `msg.text` returns the decoded string

#### Scenario: WebSocket binary message has no text

- **WHEN** `msg.text` is accessed on a BINARY message
- **THEN** it returns an error or panics

#### Scenario: Dropping a message prevents forwarding

- **WHEN** `msg.drop()` is called
- **THEN** `msg.dropped` becomes `true` and the message is not forwarded

#### Scenario: Injected messages are flagged

- **WHEN** a hook injects a WebSocket message into the stream
- **THEN** `msg.injected` is `true` and `from_client` reflects the injection direction

### Requirement: WebSocket data container

The system SHALL define a `WebSocketData` struct:

- `messages: Vec<WebSocketMessage>` — all WebSocket messages in the session
- `closed_by_client: Option<bool>` — `true` if client initiated close, `false` if server, `None` if still active
- `close_code: Option<u16>` — WebSocket close code (RFC 6455 Section 7.1.5)
- `close_reason: Option<String>` — WebSocket close reason text
- `timestamp_end: Option<f64>` — when the WebSocket connection was closed

`WebSocketData` is attached to `HTTPFlow` via `flow.websocket` — it exists only for WebSocket upgrade connections.

#### Scenario: WebSocketData starts empty

- **WHEN** a WebSocket connection is first established
- **THEN** `websocket.messages` is empty and `closed_by_client` is `None`

#### Scenario: WebSocketData tracks close

- **WHEN** a WebSocket connection is closed by the client
- **THEN** `closed_by_client` is `Some(true)` and `close_code`/`close_reason` are populated

#### Scenario: WebSocketData is optional on HTTPFlow

- **WHEN** an HTTPFlow is for a regular HTTP request (no WebSocket upgrade)
- **THEN** `flow.websocket` is `None`

### Requirement: WebSocket opcode enum

The system SHALL define a `WebSocketOpcode` enum with:

- `TEXT` — opcode 1, text frame
- `BINARY` — opcode 2, binary frame
- `CLOSE` — opcode 8, connection close
- `PING` — opcode 9, ping
- `PONG` — opcode 10, pong

#### Scenario: Opcode values match RFC 6455

- **WHEN** `WebSocketOpcode` variants are checked
- **THEN** TEXT=1, BINARY=2, CLOSE=8, PING=9, PONG=10

### Requirement: WebSocket serialization

All WebSocket types SHALL be serializable to and deserializable from JSON:

- `WebSocketData` MUST serialize all messages and close state
- `WebSocketMessage` MUST serialize all fields including `dropped` and `injected` flags

#### Scenario: WebSocketData round-trips through JSON

- **WHEN** a WebSocketData is serialized and deserialized
- **THEN** all messages, close state, and timestamps are preserved
