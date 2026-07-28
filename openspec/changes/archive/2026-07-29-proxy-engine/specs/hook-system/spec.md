## ADDED Requirements

### Requirement: Dispatch requestheaders hook
The hook system SHALL invoke registered hooks when HTTP request headers are received.

#### Scenario: Hook receives request headers
- **WHEN** client sends HTTP request with headers
- **THEN** all registered `HttpRequestHook::requestheaders` handlers are called with the flow

#### Scenario: Hook can modify headers
- **WHEN** hook modifies flow.request.headers
- **THEN** modified headers are sent to upstream server

### Requirement: Dispatch request hook
The hook system SHALL invoke registered hooks when HTTP request body is received.

#### Scenario: Hook receives full request
- **WHEN** client sends HTTP request with body
- **THEN** all registered `HttpRequestHook::request` handlers are called after body is complete

#### Scenario: Hook can modify body
- **WHEN** hook modifies flow.request.body
- **THEN** modified body is sent to upstream server

### Requirement: Dispatch response hook
The hook system SHALL invoke registered hooks when HTTP response is received from upstream.

#### Scenario: Hook receives response
- **WHEN** upstream sends HTTP response
- **THEN** all registered `HttpResponseHook::response` handlers are called with the flow

#### Scenario: Hook can modify response
- **WHEN** hook modifies flow.response.headers or body
- **THEN** modified response is sent to client

### Requirement: Dispatch error hook on failure
The hook system SHALL invoke registered hooks when an error occurs during flow processing.

#### Scenario: Hook receives error
- **WHEN** upstream connection fails
- **THEN** all registered `ErrorHook::error` handlers are called with the error and flow

#### Scenario: Error hook can log or transform
- **WHEN** error hook logs the error
- **THEN** error is recorded in structured log format

### Requirement: Hooks run asynchronously
The hook system SHALL run hooks in Tokio tasks to avoid blocking the proxy loop.

#### Scenario: Hook execution doesn't block
- **WHEN** hook performs slow I/O (e.g., database lookup)
- **THEN** proxy loop continues processing other connections

#### Scenario: Hook timeout after 5 seconds
- **WHEN** hook takes longer than 5 seconds
- **THEN** hook is cancelled and error is logged
