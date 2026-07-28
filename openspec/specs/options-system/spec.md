# Options System Specification

## Purpose

Define the `Options` struct with clap derive for CLI argument parsing and serde for config file serialization. This provides the foundation for the configuration system that allows users to manage proxy settings via CLI and config files.

## Requirements

### Requirement: Options struct with clap derive

The system SHALL provide an `Options` struct that derives `clap::Parser` for CLI argument parsing and `serde::{Serialize, Deserialize}` for config file serialization.

#### Scenario: Options struct exists with derive macros
- **WHEN** the `Options` struct is defined in `mitm_options::options`
- **THEN** it derives `Parser`, `Serialize`, `Deserialize`, `Clone`, and `Debug`

#### Scenario: Options has listen_host field
- **WHEN** user runs `mitmproxy --listen-host 0.0.0.0`
- **THEN** `Options.listen_host` is set to `"0.0.0.0"` with default `"127.0.0.1"`

#### Scenario: Options has listen_port field
- **WHEN** user runs `mitmproxy --listen-port 8080`
- **THEN** `Options.listen_port` is set to `8080` with default `8080`

#### Scenario: Options has mode field
- **WHEN** user runs `mitmproxy --mode upstream`
- **THEN** `Options.mode` is set to `ProxyMode::Upstream` with default `ProxyMode::Explicit`

#### Scenario: Options has ssl_insecure field
- **WHEN** user runs `mitmproxy --ssl-insecure`
- **THEN** `Options.ssl_insecure` is set to `true` with default `false`

#### Scenario: Options has conf_dir field
- **WHEN** user runs `mitmproxy --conf-dir ~/.mitmproxy`
- **THEN** `Options.conf_dir` is set to the specified path with default `~/.mitmproxy`

### Requirement: ProxyMode enum

The system SHALL define a `ProxyMode` enum with four variants: Explicit, Transparent, Upstream, Local.

#### Scenario: ProxyMode has Explicit variant
- **WHEN** user configures explicit proxy mode
- **THEN** `ProxyMode::Explicit` is used (client must be configured to use proxy)

#### Scenario: ProxyMode has Transparent variant
- **WHEN** user configures transparent proxy mode
- **THEN** `ProxyMode::Transparent` is used (intercepts all traffic without client config)

#### Scenario: ProxyMode has Upstream variant
- **WHEN** user configures upstream proxy mode
- **THEN** `ProxyMode::Upstream` is used (forwards to another proxy)

#### Scenario: ProxyMode has Local variant
- **WHEN** user configures local mode
- **THEN** `ProxyMode::Local` is used (no upstream, only local responses)

#### Scenario: ProxyMode implements FromStr
- **WHEN** parsing mode from string (CLI/config)
- **THEN** `"explicit"`, `"transparent"`, `"upstream"`, `"local"` are valid (case-insensitive)

#### Scenario: ProxyMode implements Display
- **WHEN** displaying mode to user
- **THEN** `ProxyMode::Explicit` displays as `"explicit"`
