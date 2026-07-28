## ADDED Requirements

### Requirement: CLI binary entry point

The system SHALL provide a working `mitm-cli` binary with complete main.rs implementation.

#### Scenario: Binary compiles and runs
- **WHEN** user runs `cargo run -p mitm-cli`
- **THEN** the binary starts and listens for connections

#### Scenario: Binary shows help
- **WHEN** user runs `mitm-cli --help`
- **THEN** the system displays usage information with all available options

#### Scenario: Binary shows version
- **WHEN** user runs `mitm-cli --version`
- **THEN** the system displays version information (0.1.0)

### Requirement: CLI argument parsing

The system SHALL parse command-line arguments using clap derive macros.

#### Scenario: Parse --host argument
- **WHEN** user specifies `--host 0.0.0.0`
- **THEN** the system uses 0.0.0.0 as listen address

#### Scenario: Parse --port argument
- **WHEN** user specifies `--port 9090`
- **THEN** the system uses port 9090

#### Scenario: Parse --mode argument
- **WHEN** user specifies `--mode transparent`
- **THEN** the system uses transparent proxy mode

#### Scenario: Parse --dump argument
- **WHEN** user specifies `--dump flows.jsonl.gz`
- **THEN** the system enables dump mode and writes flows to file

#### Scenario: Parse --set argument
- **WHEN** user specifies `--set block_url=.*ads.*`
- **THEN** the system configures the Block addon with the URL filter

### Requirement: Startup output

The system SHALL display startup information when the binary starts.

#### Scenario: Display version
- **WHEN** the binary starts
- **THEN** it displays "Mitmproxy-rs v0.1.0"

#### Scenario: Display listening address
- **WHEN** the binary starts
- **THEN** it displays "Listening on 127.0.0.1:8080 (explicit mode)"

#### Scenario: Display CA path
- **WHEN** the binary starts
- **THEN** it displays "CA: ~/.mitmproxy/mitmproxy-ca-cert.pem"

#### Scenario: Display registered addons
- **WHEN** the binary starts
- **THEN** it displays "Addons: ModifyHeaders, Block"

### Requirement: Graceful shutdown

The system SHALL handle Ctrl+C gracefully with proper cleanup.

#### Scenario: Stop accept loop on Ctrl+C
- **WHEN** user presses Ctrl+C
- **THEN** the accept loop stops accepting new connections

#### Scenario: Drain existing connections
- **WHEN** user presses Ctrl+C
- **THEN** existing connections are allowed to complete

#### Scenario: Clean exit
- **WHEN** all connections are drained
- **THEN** the binary exits with code 0
