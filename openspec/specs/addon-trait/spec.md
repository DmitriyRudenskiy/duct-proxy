# Addon Trait Specification

## Purpose

Define the `Addon` trait with async lifecycle hooks for HTTP, TCP, UDP, and DNS flows. This trait provides the foundation for the addon system that allows extending mitmproxy functionality.

## Requirements

### Requirement: Define Addon trait with lifecycle hooks

The system SHALL define an `Addon` trait with async methods for each lifecycle hook.

#### Scenario: Addon implements requestheaders hook
- **WHEN** addon implements `async fn requestheaders(&mut self, _flow: &mut FlowBase) -> Result<()>`
- **THEN** the method is called before request body is sent to upstream

#### Scenario: Addon implements request hook
- **WHEN** addon implements `async fn request(&mut self, _flow: &mut FlowBase) -> Result<()>`
- **THEN** the method is called after request body is complete

#### Scenario: Addon implements responseheaders hook
- **WHEN** addon implements `async fn responseheaders(&mut self, _flow: &mut FlowBase) -> Result<()>`
- **THEN** the method is called before response body is sent to client

#### Scenario: Addon implements response hook
- **WHEN** addon implements `async fn response(&mut self, _flow: &mut FlowBase) -> Result<()>`
- **THEN** the method is called after response body is complete

#### Scenario: Addon implements error hook
- **WHEN** addon implements `async fn error(&mut self, _error: &AddonError) -> Result<()>`
- **THEN** the method is called when an error occurs during flow processing

#### Scenario: Addon implements TCP message hook
- **WHEN** addon implements `async fn tcp_message(&mut self, _flow: &mut FlowBase, _message: &[u8]) -> Result<()>`
- **THEN** the method is called for each TCP message in a TCP flow

#### Scenario: Addon implements UDP message hook
- **WHEN** addon implements `async fn udp_message(&mut self, _flow: &mut FlowBase, _message: &[u8]) -> Result<()>`
- **THEN** the method is called for each UDP message in a UDP flow

#### Scenario: Addon implements DNS request hook
- **WHEN** addon implements `async fn dns_request(&mut self, _flow: &mut FlowBase, _message: &[u8]) -> Result<()>`
- **THEN** the method is called for DNS query messages

#### Scenario: Addon implements DNS response hook
- **WHEN** addon implements `async fn dns_response(&mut self, _flow: &mut FlowBase, _message: &[u8]) -> Result<()>`
- **THEN** the method is called for DNS response messages

### Requirement: Addon error type

The system SHALL define an `AddonError` enum for addon execution errors.

#### Scenario: Addon returns execution error
- **WHEN** addon returns `Err(AddonError::Execution("message".to_string()))`
- **THEN** the error is propagated to the addon manager and logged

#### Scenario: Addon returns timeout error
- **WHEN** addon returns `Err(AddonError::Timeout(duration))`
- **THEN** the addon execution is cancelled and error is logged

### Requirement: Addon lifetime management

The system SHALL ensure addons are Send + Sync for multithreaded dispatch.

#### Scenario: Addon is Send + Sync
- **WHEN** addon implements `Addon` trait
- **THEN** the addon type MUST implement `Send + Sync`
