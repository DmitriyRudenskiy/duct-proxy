## ADDED Requirements

### Requirement: Config file loading

The system SHALL load options from a YAML config file (`~/.mitmproxy/config.yaml`) with serde_yaml.

#### Scenario: Config file is loaded from default location
- **WHEN** no config path is specified
- **THEN** the system loads `~/.mitmproxy/config.yaml` (using `dirs` crate for platform path)

#### Scenario: Config file is loaded from custom path
- **WHEN** user specifies `--config /path/to/config.yaml`
- **THEN** the system loads from the specified path

#### Scenario: Config file is missing
- **WHEN** config file does not exist
- **THEN** the system uses defaults and logs a debug message (no error)

#### Scenario: Config file has invalid YAML
- **WHEN** config file contains malformed YAML
- **THEN** the system returns an `IoError` with clear message

### Requirement: CLI override merge

The system SHALL merge CLI arguments with config file options, with CLI taking precedence.

#### Scenario: CLI args override config file
- **WHEN** config file has `listen_port: 8080` and user specifies `--listen-port 9090`
- **THEN** `Options.listen_port` is `9090` (CLI wins)

#### Scenario: Config file provides defaults
- **WHEN** config file has `listen_port: 8080` and user does not specify port
- **THEN** `Options.listen_port` is `8080` (from config)

#### Scenario: Empty config file uses struct defaults
- **WHEN** config file is empty or has no matching keys
- **THEN** struct defaults from clap apply

### Requirement: Config file generation

The system SHALL generate a sample config file from current options.

#### Scenario: Dump config to stdout
- **WHEN** user runs `mitmproxy --dump-config`
- **THEN** the system prints current options as YAML to stdout

#### Scenario: Save config to file
- **WHEN** user runs `mitmproxy --save-config /path/to/config.yaml`
- **THEN** the system writes current options to the specified file

### Requirement: Config file directory creation

The system SHALL create the config directory if it does not exist.

#### Scenario: Config directory is created
- **WHEN** `~/.mitmproxy/` does not exist
- **THEN** the system creates it with `std::fs::create_dir_all`

#### Scenario: Config directory creation fails
- **WHEN** user lacks permissions to create config directory
- **THEN** the system returns an `IoError` with clear message
