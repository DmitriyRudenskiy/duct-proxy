//! OptManager for thread-safe runtime options management.

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::ValidationError;
use crate::options::Options;

/// Thread-safe manager for runtime options.
pub struct OptManager {
    options: Arc<RwLock<Options>>,
}

impl OptManager {
    /// Create a new OptManager with the given options.
    pub fn new(options: Options) -> Self {
        Self {
            options: Arc::new(RwLock::new(options)),
        }
    }

    /// Get a clone of the current options.
    pub async fn get(&self) -> Options {
        self.options.read().await.clone()
    }

    /// Set new options after validation.
    pub async fn set(&self, new_options: Options) -> Result<(), ValidationError> {
        new_options.validate()?;
        *self.options.write().await = new_options;
        Ok(())
    }

    /// Get the underlying Arc for cloning.
    pub fn clone_manager(&self) -> Self {
        Self {
            options: self.options.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::Options;

    #[tokio::test]
    async fn test_opt_manager_new() {
        let opts = Options::default();
        let manager = OptManager::new(opts);
        let retrieved = manager.get().await;
        assert_eq!(retrieved.listen_port, 8080);
    }

    #[tokio::test]
    async fn test_opt_manager_set() {
        let opts = Options::default();
        let manager = OptManager::new(opts);

        let mut new_opts = Options::default();
        new_opts.listen_port = 9090;
        manager.set(new_opts).await.unwrap();

        let retrieved = manager.get().await;
        assert_eq!(retrieved.listen_port, 9090);
    }

    #[tokio::test]
    async fn test_opt_manager_set_invalid() {
        let opts = Options::default();
        let manager = OptManager::new(opts);

        let mut new_opts = Options::default();
        new_opts.listen_port = 0;
        assert!(manager.set(new_opts).await.is_err());
    }

    #[tokio::test]
    async fn test_opt_manager_clone() {
        let opts = Options::default();
        let manager1 = OptManager::new(opts);
        let manager2 = manager1.clone_manager();

        let mut new_opts = Options::default();
        new_opts.listen_port = 7070;
        manager1.set(new_opts).await.unwrap();

        let retrieved = manager2.get().await;
        assert_eq!(retrieved.listen_port, 7070);
    }
}
