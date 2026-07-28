//! Config file loading and saving utilities.

use std::path::Path;

use crate::error::OptionsError;
use crate::options::Options;

/// Load options from a config file.
pub fn load_config(path: &Path) -> Result<Options, OptionsError> {
    if !path.exists() {
        return Err(OptionsError::ConfigNotFound(
            path.to_string_lossy().to_string(),
        ));
    }

    Options::from_config(path)
}

/// Save options to a config file.
pub fn save_config(options: &Options, path: &Path) -> Result<(), OptionsError> {
    options.save_config(path)
}

/// Dump options to a YAML string.
pub fn dump_config(options: &Options) -> Result<String, OptionsError> {
    options.to_config_yaml()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::Options;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_config_missing() {
        let result = load_config(Path::new("/nonexistent/config.yaml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_save_and_load_config() {
        let opts = Options::default();
        let tmp_file = NamedTempFile::new().unwrap();
        let path = tmp_file.path();

        save_config(&opts, path).unwrap();
        let loaded = load_config(path).unwrap();

        assert_eq!(loaded.listen_host, opts.listen_host);
        assert_eq!(loaded.listen_port, opts.listen_port);
    }

    #[test]
    fn test_dump_config() {
        let opts = Options::default();
        let yaml = dump_config(&opts).unwrap();
        assert!(yaml.contains("listen_host: 127.0.0.1"));
        assert!(yaml.contains("listen_port: 8080"));
    }
}
