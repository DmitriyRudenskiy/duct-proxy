//! Error types for mitm-options.

use thiserror::Error;

/// Errors that can occur during options management.
#[derive(Error, Debug)]
pub enum OptionsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Config file not found: {0}")]
    ConfigNotFound(String),

    #[error("Invalid option: {0}")]
    InvalidOption(String),
}

/// Validation error details.
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Port must be between 1 and 65535, got {0}")]
    InvalidPort(u16),

    #[error("Invalid host format: {0}")]
    InvalidHost(String),

    #[error("Invalid mode: {0}")]
    InvalidMode(String),
}
