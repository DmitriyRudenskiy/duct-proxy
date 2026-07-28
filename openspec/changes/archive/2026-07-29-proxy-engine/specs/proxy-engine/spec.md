## ADDED Requirements

### Requirement: Proxy engine runs as async TCP server
The proxy engine SHALL accept TCP connections on a configurable port and handle them asynchronously using Tokio.

#### Scenario: Server starts on configured port
- **WHEN** proxy is started with `--listen 0.0.0.0:8080`
- **THEN** TCP listener binds to 0.0.0.0:8080 and accepts connections

#### Scenario: Server handles multiple concurrent connections
- **WHEN** 100 clients connect simultaneously
- **THEN** all 100 connections are accepted and processed concurrently without blocking

### Requirement: Proxy loops connections until shutdown
The proxy engine SHALL maintain an accept loop that processes connections until a shutdown signal is received.

#### Scenario: Accept loop runs continuously
- **WHEN** proxy is running and connections arrive
- **THEN** each connection is dispatched to a handler task without stopping the loop

#### Scenario: Graceful shutdown on SIGTERM
- **WHEN** user presses Ctrl+C or sends SIGTERM
- **THEN** proxy stops accepting new connections, waits for active connections to complete, then exits

### Requirement: Connection tasks use JoinSet
The proxy engine SHALL use Tokio's `JoinSet` to manage connection handler tasks.

#### Scenario: JoinSet tracks all active connections
- **WHEN** 50 connections are being processed
- **THEN** `JoinSet::len()` returns 50 and all tasks are tracked

#### Scenario: JoinSet handles task completion
- **WHEN** a connection handler task completes
- **THEN** the task is automatically removed from the JoinSet
