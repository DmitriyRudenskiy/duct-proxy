# cli-modes Specification

## Purpose
TBD - created by archiving change cli-tools. Update Purpose after archive.
## Requirements
### Requirement: Default proxy mode

The system SHALL run in default proxy mode, intercepting all HTTP/HTTPS traffic.

#### Scenario: Proxy intercepts HTTP requests
- **WHEN** client sends HTTP request to proxy
- **THEN** the proxy forwards it to upstream and returns response

#### Scenario: Proxy intercepts HTTPS requests
- **WHEN** client sends HTTPS request via CONNECT
- **THEN** the proxy performs TLS interception and forwards request

### Requirement: Dump mode

The system SHALL support dump mode to save all flows to a JSONL file.

#### Scenario: Enable dump mode with --dump
- **WHEN** user specifies `--dump flows.jsonl.gz`
- **THEN** all flows are saved to the specified file in gzip-compressed JSONL format

#### Scenario: Dump file is valid
- **WHEN** dump file is created
- **THEN** it can be read with `FlowReader` and contains valid flows

### Requirement: Inline addon configuration

The system SHALL support inline addon configuration with --set flag.

#### Scenario: Configure Block addon with --set
- **WHEN** user specifies `--set block_url=.*ads.*`
- **THEN** the Block addon is configured to block URLs matching the regex

#### Scenario: Configure ModifyHeaders addon with --set
- **WHEN** user specifies `--set add_header=X-Proxy:mitmproxy-rs`
- **THEN** the ModifyHeaders addon adds the specified header to all requests

