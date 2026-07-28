//! Options struct with clap derive for CLI and serde for config files.

use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::str::FromStr;

use crate::error::OptionsError;
use crate::proxy_mode::ProxyMode;

/// Proxy configuration options.
///
/// This struct can be used for both CLI argument parsing (via clap) and
/// config file deserialization (via serde).
#[derive(Parser, Serialize, Deserialize, Clone, Debug)]
#[command(name = "mitmproxy", about = "Interactive HTTPS proxy")]
pub struct Options {
    /// Host to listen on.
    #[arg(long, default_value = "127.0.0.1")]
    #[serde(default = "default_listen_host")]
    pub listen_host: String,

    /// Port to listen on.
    #[arg(short, long, default_value = "8080")]
    #[serde(default = "default_listen_port")]
    pub listen_port: u16,

    /// Proxy mode: explicit, transparent, upstream, local.
    #[arg(long, default_value = "explicit")]
    #[serde(default = "default_mode")]
    pub mode: String,

    /// Don't verify upstream server certificates.
    #[arg(long)]
    #[serde(default)]
    pub ssl_insecure: bool,

    /// Configuration directory path.
    #[arg(long)]
    #[serde(default = "default_conf_dir")]
    pub conf_dir: String,

    /// Config file path (overrides default).
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<String>,

    /// Dump current config to stdout.
    #[arg(long, hide = true)]
    pub dump_config: bool,

    /// Save config to file.
    #[arg(long)]
    #[serde(skip)]
    pub save_config: Option<String>,
}

fn default_listen_host() -> String {
    "127.0.0.1".to_string()
}

fn default_listen_port() -> u16 {
    8080
}

fn default_mode() -> String {
    "explicit".to_string()
}

fn default_conf_dir() -> String {
    dirs::home_dir()
        .unwrap_or_else(|| Path::new(".").to_path_buf())
        .join(".mitmproxy")
        .to_string_lossy()
        .to_string()
}

impl Default for Options {
    fn default() -> Self {
        Self {
            listen_host: default_listen_host(),
            listen_port: default_listen_port(),
            mode: default_mode(),
            ssl_insecure: false,
            conf_dir: default_conf_dir(),
            config: None,
            dump_config: false,
            save_config: None,
        }
    }
}

impl Options {
    /// Load options from a config file.
    pub fn from_config(path: &Path) -> Result<Self, OptionsError> {
        let content = std::fs::read_to_string(path)?;
        let options: Options = serde_yaml::from_str(&content)?;
        Ok(options)
    }

    /// Load options from default config path.
    pub fn from_default_config() -> Result<Self, OptionsError> {
        let config_path = Self::default_config_path();
        if config_path.exists() {
            Self::from_config(&config_path)
        } else {
            tracing::debug!("Config file not found, using defaults: {}", config_path.display());
            Ok(Self::default())
        }
    }

    /// Get the default config file path.
    pub fn default_config_path() -> std::path::PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| Path::new(".").to_path_buf())
            .join(".mitmproxy")
            .join("config.yaml")
    }

    /// Merge two Options structs: CLI args override config file.
    pub fn merge(cli: &Options, config: &Options) -> Self {
        let mut merged = config.clone();

        // CLI args override config file.
        if cli.listen_host != default_listen_host() {
            merged.listen_host = cli.listen_host.clone();
        }
        if cli.listen_port != default_listen_port() {
            merged.listen_port = cli.listen_port;
        }
        if cli.mode != default_mode() {
            merged.mode = cli.mode.clone();
        }
        if cli.ssl_insecure {
            merged.ssl_insecure = cli.ssl_insecure;
        }
        if cli.conf_dir != default_conf_dir() {
            merged.conf_dir = cli.conf_dir.clone();
        }
        if let Some(ref config_path) = cli.config {
            merged.config = Some(config_path.clone());
        }

        merged
    }

    /// Serialize options to YAML string.
    pub fn to_config_yaml(&self) -> Result<String, OptionsError> {
        let yaml = serde_yaml::to_string(self)?;
        Ok(yaml)
    }

    /// Save options to a config file.
    pub fn save_config(&self, path: &Path) -> Result<(), OptionsError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let yaml = self.to_config_yaml()?;
        std::fs::write(path, yaml)?;
        Ok(())
    }

    /// Validate options.
    pub fn validate(&self) -> Result<(), crate::error::ValidationError> {
        // Validate port.
        if self.listen_port == 0 {
            return Err(crate::error::ValidationError::InvalidPort(self.listen_port));
        }

        // Validate host (basic check).
        if self.listen_host.parse::<std::net::IpAddr>().is_err() {
            return Err(crate::error::ValidationError::InvalidHost(
                self.listen_host.clone(),
            ));
        }

        // Validate mode.
        if ProxyMode::from_str(&self.mode).is_err() {
            return Err(crate::error::ValidationError::InvalidMode(
                self.mode.clone(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_options_default() {
        let opts = Options::default();
        assert_eq!(opts.listen_host, "127.0.0.1");
        assert_eq!(opts.listen_port, 8080);
        assert_eq!(opts.mode, "explicit");
        assert!(!opts.ssl_insecure);
    }

    #[test]
    fn test_options_to_yaml() {
        let opts = Options::default();
        let yaml = opts.to_config_yaml().unwrap();
        assert!(yaml.contains("listen_host: 127.0.0.1"));
        assert!(yaml.contains("listen_port: 8080"));
    }

    #[test]
    fn test_options_merge_cli_overrides_config() {
        let config = Options::default();
        let mut cli = Options::default();
        cli.listen_port = 9090;

        let merged = Options::merge(&cli, &config);
        assert_eq!(merged.listen_port, 9090);
    }

    #[test]
    fn test_options_validate_valid() {
        let opts = Options::default();
        assert!(opts.validate().is_ok());
    }

    #[test]
    fn test_options_validate_invalid_port() {
        let mut opts = Options::default();
        opts.listen_port = 0;
        assert!(opts.validate().is_err());
    }

    #[test]
    fn test_options_validate_invalid_host() {
        let mut opts = Options::default();
        opts.listen_host = "invalid".to_string();
        assert!(opts.validate().is_err());
    }
}
