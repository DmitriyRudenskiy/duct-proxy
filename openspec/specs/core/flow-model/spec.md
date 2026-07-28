## ADDED Requirements

### Requirement: Flow type hierarchy

The system SHALL define a `Flow` enum with four variants representing the supported flow types:

- `HTTPFlow` — HTTP request/response transactions
- `TCPFlow` — Raw TCP stream sessions
- `UDPFlow` — UDP datagram sessions
- `DNSFlow` — DNS query/response pairs

Each flow variant SHALL carry shared base state defined in `FlowBase`.

#### Scenario: Flow enum covers all protocol types

- **WHEN** the proxy detects an HTTP, TCP, UDP, or DNS connection
- **THEN** the flow is represented as the corresponding `Flow` enum variant

#### Scenario: Flow variants are exhaustive

- **WHEN** a developer matches on all `Flow` variants
- **THEN** the compiler enforces exhaustiveness (no unreachable code)

### Requirement: FlowBase shared state

All flow variants SHALL contain a `FlowBase` struct with the following fields:

- `id: String` — UUID4 unique identifier
- `client_conn: Client` — the client connection
- `server_conn: Server` — the server connection
- `error: Option<FlowError>` — connection or protocol error
- `intercepted: bool` — whether the flow is paused
- `marked: String` — user-set marker annotation
- `is_replay: Option<String>` — replay direction ("request" or "response")
- `live: bool` — whether the flow belongs to an active connection
- `timestamp_created: f64` — Unix timestamp of flow creation
- `metadata: HashMap<String, serde_json::Value>` — arbitrary user metadata
- `comment: String` — user comment

#### Scenario: All flows have a unique ID

- **WHEN** a new flow is created
- **THEN** it is assigned a UUID4 string in `FlowBase.id`

#### Scenario: FlowBase is cloned per variant

- **WHEN** a flow copy is made
- **THEN** each variant contains its own FlowBase clone

### Requirement: Flow error model

The system SHALL define a `FlowError` struct with:

- `msg: String` — human-readable error description
- `timestamp: f64` — Unix timestamp of when the error occurred

A flow MAY have both a successful response and an error (e.g., response received from server but error sending to client).

#### Scenario: Killed flow has error

- **WHEN** a flow is killed
- **THEN** `flow.error` is set to `Some(FlowError { msg: "Connection killed.", ... })`

#### Scenario: HTTPFlow can have both response and error

- **WHEN** a server responds but the connection to the client fails
- **THEN** `HTTPFlow.response` is `Some(Response)` and `flow.error` is `Some(FlowError)`

### Requirement: Flow intercept/resume lifecycle

The system SHALL support intercepting and resuming flows:

- `intercept()` sets `intercepted = true` and signals waiting coroutines
- `resume()` sets `intercepted = false` and wakes waiting coroutines
- `wait_for_resume()` blocks until `intercepted` becomes `false`
- Double-intercept is a no-op
- Resume without prior intercept is a no-op

#### Scenario: Intercept pauses flow processing

- **WHEN** a hook calls `flow.intercept()`
- **THEN** `flow.intercepted` becomes `true` and downstream processing stops

#### Scenario: Resume releases flow

- **WHEN** `flow.resume()` is called after intercept
- **THEN** `flow.intercepted` becomes `false` and `wait_for_resume()` returns

#### Scenario: Double intercept is idempotent

- **WHEN** `flow.intercept()` is called twice in succession
- **THEN** the flow remains intercepted without error

### Requirement: Flow kill

The system SHALL provide a `kill()` method on flows:

- `kill()` sets the error to `KILLED_MESSAGE`, clears `intercepted`, and sets `live = false`
- `kill()` on a non-killable flow (already killed or errored with kill message) MUST panic or return an error

#### Scenario: Kill stops flow forwarding

- **WHEN** `flow.kill()` is called on a live flow
- **THEN** the flow's error is set and it will not be forwarded to its destination

#### Scenario: killable property excludes killed flows

- **WHEN** a flow has error message equal to `KILLED_MESSAGE`
- **THEN** `flow.killable` returns `false`

### Requirement: Flow replay tracking

The system SHALL track replay state via `is_replay: Option<String>`:

- `"request"` — the request was artificially replayed to the server
- `"response"` — the response was set from a server replay
- `None` — the flow is a live (non-replayed) transaction

#### Scenario: Replay direction is tracked

- **WHEN** a stored flow's request is replayed
- **THEN** `flow.is_replay` is set to `Some("request")`

### Requirement: Flow copy

The system SHALL provide a `copy()` method that:

- Deep-copies all fields including flow-specific data (request, response, messages, etc.)
- Sets `live = false` on the copy
- Generates a new `id`

#### Scenario: Copy produces independent flow

- **WHEN** `flow.copy()` is called and the original is modified
- **THEN** the copy remains unchanged

### Requirement: Flow serialization

All flow types SHALL be serializable to and deserializable from JSON using serde:

- The serialized form MUST include a `"type"` tag identifying the flow variant
- The serialized form MUST include all `FlowBase` fields
- Flow-specific fields (request, response, messages, websocket) MUST be serialized within their variant

#### Scenario: HTTPFlow serializes round-trip

- **WHEN** an HTTPFlow is serialized to JSON and deserialized back
- **THEN** all fields (request, response, base state) are preserved

#### Scenario: Unknown flow type fails gracefully

- **WHEN** deserializing a JSON flow with an unrecognized `"type"` value
- **THEN** deserialization fails with an error (does not panic)
