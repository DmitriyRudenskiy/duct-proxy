# cli-logging Specification

## Purpose
TBD - created by archiving change cli-tools. Update Purpose after archive.
## Requirements
### Requirement: Per-flow logging

The system SHALL log information for each HTTP flow with timing and size details.

#### Scenario: Log HTTP request/response
- **WHEN** an HTTP flow completes
- **THEN** the system logs a line like: `[14:32:01] GET http://example.com/ → 200 (1.2 KB, 45ms)`

#### Scenario: Log CONNECT tunnel
- **WHEN** a CONNECT tunnel is established
- **THEN** the system logs: `[14:32:02] CONNECT api.github.com:443 → TLS intercepted`

#### Scenario: Log includes timestamp
- **WHEN** a flow is logged
- **THEN** the log line includes the current time in HH:MM:SS format

#### Scenario: Log includes method and URL
- **WHEN** a flow is logged
- **THEN** the log line includes the HTTP method and full URL

#### Scenario: Log includes status code
- **WHEN** a flow is logged
- **THEN** the log line includes the response status code

#### Scenario: Log includes response size
- **WHEN** a flow is logged
- **THEN** the log line includes the response body size in human-readable format (B, KB, MB)

#### Scenario: Log includes timing
- **WHEN** a flow is logged
- **THEN** the log line includes the total request/response time in milliseconds

### Requirement: Log format

The system SHALL use a consistent log format for all flow entries.

#### Scenario: Human-readable format
- **WHEN** logging is enabled (default)
- **THEN** logs are in human-readable format with brackets and arrows

#### Scenario: Configurable log level
- **WHEN** user specifies `--log-level debug`
- **THEN** the system shows debug-level logs in addition to info-level

