## ADDED Requirements

### Requirement: FlowSerializer trait

The system SHALL provide a `FlowSerializer` trait for converting `HTTPFlow` to JSON and back.

#### Scenario: Serialize HTTPFlow to JSON
- **WHEN** `FlowSerializer::serialize(flow)` is called
- **THEN** the flow is converted to a valid JSON string

#### Scenario: Deserialize JSON to HTTPFlow
- **WHEN** `FlowSerializer::deserialize(json)` is called
- **THEN** the JSON string is converted back to an `HTTPFlow`

#### Scenario: Roundtrip preserves data
- **WHEN** a flow is serialized then deserialized
- **THEN** the resulting flow is equal to the original

### Requirement: HTTPFlow serialization derives

The system SHALL derive `serde::Serialize` and `serde::Deserialize` on `HTTPFlow`.

#### Scenario: HTTPFlow can be serialized
- **WHEN** `HTTPFlow` is passed to `serde_json::to_string`
- **THEN** it produces valid JSON

#### Scenario: HTTPFlow can be deserialized
- **WHEN** valid JSON is passed to `serde_json::from_str`
- **THEN** it produces an `HTTPFlow`

### Requirement: Binary content encoding

The system SHALL encode binary content (body, certificates) as base64 in JSON.

#### Scenario: Binary body is base64 encoded
- **WHEN** a flow with binary body is serialized
- **THEN** the body is encoded as base64 string in JSON

#### Scenario: Base64 body is decoded on deserialization
- **WHEN** a JSON with base64 body is deserialized
- **THEN** the body is decoded back to bytes
