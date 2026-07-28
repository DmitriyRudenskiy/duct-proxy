//! ProxyMode enum for different proxy operating modes.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Proxy operating mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProxyMode {
    /// Explicit proxy: client must be configured to use proxy.
    Explicit,
    /// Transparent proxy: intercepts all traffic without client config.
    Transparent,
    /// Upstream proxy: forwards to another proxy.
    Upstream,
    /// Local mode: no upstream, only local responses.
    Local,
}

impl fmt::Display for ProxyMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Explicit => write!(f, "explicit"),
            Self::Transparent => write!(f, "transparent"),
            Self::Upstream => write!(f, "upstream"),
            Self::Local => write!(f, "local"),
        }
    }
}

impl Serialize for ProxyMode {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ProxyMode {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse::<ProxyMode>().map_err(serde::de::Error::custom)
    }
}

impl FromStr for ProxyMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "explicit" => Ok(Self::Explicit),
            "transparent" => Ok(Self::Transparent),
            "upstream" => Ok(Self::Upstream),
            "local" => Ok(Self::Local),
            other => Err(format!("Invalid proxy mode: '{}'", other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_mode_from_str() {
        assert_eq!("explicit".parse::<ProxyMode>().unwrap(), ProxyMode::Explicit);
        assert_eq!("EXPLICIT".parse::<ProxyMode>().unwrap(), ProxyMode::Explicit);
        assert_eq!("transparent".parse::<ProxyMode>().unwrap(), ProxyMode::Transparent);
        assert_eq!("upstream".parse::<ProxyMode>().unwrap(), ProxyMode::Upstream);
        assert_eq!("local".parse::<ProxyMode>().unwrap(), ProxyMode::Local);
    }

    #[test]
    fn test_proxy_mode_from_str_invalid() {
        assert!("invalid".parse::<ProxyMode>().is_err());
    }

    #[test]
    fn test_proxy_mode_display() {
        assert_eq!(ProxyMode::Explicit.to_string(), "explicit");
        assert_eq!(ProxyMode::Transparent.to_string(), "transparent");
        assert_eq!(ProxyMode::Upstream.to_string(), "upstream");
        assert_eq!(ProxyMode::Local.to_string(), "local");
    }

    #[test]
    fn test_proxy_mode_serialize() {
        let mode = ProxyMode::Explicit;
        let yaml = serde_yaml::to_string(&mode).unwrap();
        assert_eq!(yaml.trim(), "explicit");
    }

    #[test]
    fn test_proxy_mode_deserialize() {
        let mode: ProxyMode = serde_yaml::from_str("explicit").unwrap();
        assert_eq!(mode, ProxyMode::Explicit);
    }
}
