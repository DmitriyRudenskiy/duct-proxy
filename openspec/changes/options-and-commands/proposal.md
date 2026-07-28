## Why

mitmproxy-rs currently lacks a centralized configuration system. Users cannot persist settings across sessions, and there is no unified way to manage proxy options (listen address, mode, SSL settings, etc.). The Python mitmproxy has a robust options system (`mitmproxy/options.py`, `mitmproxy/optmanager.py`) that allows configuration via CLI args, config files, and runtime changes. This change implements a similar system in Rust.

## What Changes

- **New crate `mitm-options`**: Dedicated crate for configuration management
- **Options struct**: Derive `Parser` (clap) for CLI and `Serialize/Deserialize` (serde) for config files
- **Config file support**: YAML config file (`~/.mitmproxy/config.yaml`) with merge logic (CLI args override config file)
- **OptManager**: Runtime get/set with validation
- **ProxyMode enum**: Explicit, Transparent, Upstream, Local modes
- **CLI integration**: mitm-proxy and mitm-cli will use the new options system

## Capabilities

### New Capabilities

- `options-system`: Core Options struct, ProxyMode enum, clap derive, serde serialization
- `config-file`: YAML config file reading/writing, merge with CLI args
- `opt-manager`: Runtime options management, get/set, validation, change notifications

### Modified Capabilities

- None (this is a new system, not modifying existing specs)

## Impact

**New Dependencies:**
- `clap` (v4) - CLI argument parsing
- `serde` + `serde_yaml` - Config file serialization
- `dirs` - Platform-specific config directory paths
- `tracing` - Logging for config changes

**Affected Code:**
- `crates/mitm-proxy/` - Will accept Options struct instead of individual params
- `crates/mitm-cli/` - Will use clap-derived Options for CLI
- `crates/mitm-core/` - May expose ProxyMode if needed elsewhere

**API Changes:**
- `ProxyServer::bind(host, port, mode, ...)` → `ProxyServer::bind(options: &Options)`
- New public API: `Options`, `OptManager`, `ProxyMode`
