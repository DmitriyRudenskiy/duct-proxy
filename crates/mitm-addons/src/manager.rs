//! AddonManager for registration and sequential dispatch.

use crate::addon::{Addon, AddonError};
use mitm_core::FlowBase;
use tracing::{debug, error, info};

/// Error policy for addon dispatch.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorPolicy {
    /// Stop dispatch on first error.
    StopOnFirst,
    /// Continue dispatch on error, log and collect errors.
    ContinueOnError,
}

impl Default for ErrorPolicy {
    fn default() -> Self {
        Self::StopOnFirst
    }
}

/// Manager for addon registration and dispatch.
pub struct AddonManager {
    addons: Vec<Box<dyn Addon>>,
    error_policy: ErrorPolicy,
}

impl AddonManager {
    /// Create a new AddonManager.
    pub fn new() -> Self {
        Self {
            addons: Vec::new(),
            error_policy: ErrorPolicy::default(),
        }
    }

    /// Create a new AddonManager with the specified error policy.
    pub fn with_error_policy(mut self, policy: ErrorPolicy) -> Self {
        self.error_policy = policy;
        self
    }

    /// Set the error policy.
    pub fn set_error_policy(&mut self, policy: ErrorPolicy) {
        self.error_policy = policy;
    }

    /// Register an addon.
    pub fn register(&mut self, addon: Box<dyn Addon>) {
        info!("Registered addon");
        self.addons.push(addon);
    }

    /// Dispatch requestheaders hook to all addons.
    pub async fn dispatch_requestheaders(&mut self, flow: &mut FlowBase) -> Result<(), AddonError> {
        debug!("Dispatching requestheaders to {} addons", self.addons.len());
        for (i, addon) in self.addons.iter_mut().enumerate() {
            match addon.requestheaders(flow).await {
                Ok(()) => {
                    debug!("Addon {} completed requestheaders", i);
                }
                Err(e) => {
                    error!("Addon {} failed requestheaders: {}", i, e);
                    match self.error_policy {
                        ErrorPolicy::StopOnFirst => return Err(e),
                        ErrorPolicy::ContinueOnError => continue,
                    }
                }
            }
        }
        Ok(())
    }

    /// Dispatch request hook to all addons.
    pub async fn dispatch_request(&mut self, flow: &mut FlowBase) -> Result<(), AddonError> {
        debug!("Dispatching request to {} addons", self.addons.len());
        for (i, addon) in self.addons.iter_mut().enumerate() {
            match addon.request(flow).await {
                Ok(()) => {
                    debug!("Addon {} completed request", i);
                }
                Err(e) => {
                    error!("Addon {} failed request: {}", i, e);
                    match self.error_policy {
                        ErrorPolicy::StopOnFirst => return Err(e),
                        ErrorPolicy::ContinueOnError => continue,
                    }
                }
            }
        }
        Ok(())
    }

    /// Dispatch responseheaders hook to all addons.
    pub async fn dispatch_responseheaders(&mut self, flow: &mut FlowBase) -> Result<(), AddonError> {
        debug!("Dispatching responseheaders to {} addons", self.addons.len());
        for (i, addon) in self.addons.iter_mut().enumerate() {
            match addon.responseheaders(flow).await {
                Ok(()) => {
                    debug!("Addon {} completed responseheaders", i);
                }
                Err(e) => {
                    error!("Addon {} failed responseheaders: {}", i, e);
                    match self.error_policy {
                        ErrorPolicy::StopOnFirst => return Err(e),
                        ErrorPolicy::ContinueOnError => continue,
                    }
                }
            }
        }
        Ok(())
    }

    /// Dispatch response hook to all addons.
    pub async fn dispatch_response(&mut self, flow: &mut FlowBase) -> Result<(), AddonError> {
        debug!("Dispatching response to {} addons", self.addons.len());
        for (i, addon) in self.addons.iter_mut().enumerate() {
            match addon.response(flow).await {
                Ok(()) => {
                    debug!("Addon {} completed response", i);
                }
                Err(e) => {
                    error!("Addon {} failed response: {}", i, e);
                    match self.error_policy {
                        ErrorPolicy::StopOnFirst => return Err(e),
                        ErrorPolicy::ContinueOnError => continue,
                    }
                }
            }
        }
        Ok(())
    }

    /// Dispatch error hook to all addons.
    pub async fn dispatch_error(&mut self, error: &AddonError) -> Result<(), AddonError> {
        debug!("Dispatching error to {} addons", self.addons.len());
        for (i, addon) in self.addons.iter_mut().enumerate() {
            match addon.error(error).await {
                Ok(()) => {
                    debug!("Addon {} completed error handling", i);
                }
                Err(e) => {
                    error!("Addon {} failed error handling: {}", i, e);
                    match self.error_policy {
                        ErrorPolicy::StopOnFirst => return Err(e),
                        ErrorPolicy::ContinueOnError => continue,
                    }
                }
            }
        }
        Ok(())
    }

    /// Get the number of registered addons.
    pub fn len(&self) -> usize {
        self.addons.len()
    }

    /// Check if there are no registered addons.
    pub fn is_empty(&self) -> bool {
        self.addons.is_empty()
    }
}

impl Default for AddonManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Addon, AddonError, ModifyHeaders};
    use mitm_core::FlowBase;

    /// Test addon that tracks hook calls.
    struct CallTracker {
        called: std::sync::Arc<std::sync::atomic::AtomicBool>,
    }

    #[async_trait::async_trait]
    impl Addon for CallTracker {
        async fn requestheaders(&mut self, _flow: &mut FlowBase) -> Result<(), AddonError> {
            self.called.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }

        async fn request(&mut self, _flow: &mut FlowBase) -> Result<(), AddonError> {
            self.called.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }

        async fn responseheaders(&mut self, _flow: &mut FlowBase) -> Result<(), AddonError> {
            self.called.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }

        async fn response(&mut self, _flow: &mut FlowBase) -> Result<(), AddonError> {
            self.called.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        }
    }

    fn test_flow() -> FlowBase {
        FlowBase::new(
            mitm_core::connection::Client::new(
                ("127.0.0.1".to_string(), 12345),
                ("127.0.0.1".to_string(), 80),
                "regular",
            ),
            mitm_core::connection::Server::new(),
            true,
        )
    }

    #[tokio::test]
    async fn test_addon_manager_creation() {
        let mgr = AddonManager::new();
        assert!(mgr.is_empty());
        assert_eq!(mgr.len(), 0);
    }

    #[tokio::test]
    async fn test_addon_manager_register() {
        let mut mgr = AddonManager::new();
        let addon = ModifyHeaders::new().add("X-Test", "value");
        mgr.register(Box::new(addon));
        assert_eq!(mgr.len(), 1);
        assert!(!mgr.is_empty());
    }

    #[tokio::test]
    async fn test_addon_manager_dispatch_requestheaders() {
        let mut mgr = AddonManager::new();
        let called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called_clone = called.clone();

        mgr.register(Box::new(CallTracker {
            called: called_clone,
        }));

        let mut flow = test_flow();
        mgr.dispatch_requestheaders(&mut flow).await.unwrap();
        assert!(called.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_addon_manager_dispatch_request() {
        let mut mgr = AddonManager::new();
        let called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called_clone = called.clone();

        mgr.register(Box::new(CallTracker {
            called: called_clone,
        }));

        let mut flow = test_flow();
        mgr.dispatch_request(&mut flow).await.unwrap();
        assert!(called.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_addon_manager_dispatch_responseheaders() {
        let mut mgr = AddonManager::new();
        let called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called_clone = called.clone();

        mgr.register(Box::new(CallTracker {
            called: called_clone,
        }));

        let mut flow = test_flow();
        mgr.dispatch_responseheaders(&mut flow).await.unwrap();
        assert!(called.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_addon_manager_dispatch_response() {
        let mut mgr = AddonManager::new();
        let called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let called_clone = called.clone();

        mgr.register(Box::new(CallTracker {
            called: called_clone,
        }));

        let mut flow = test_flow();
        mgr.dispatch_response(&mut flow).await.unwrap();
        assert!(called.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_addon_manager_error_isolation() {
        let mut mgr = AddonManager::new();

        struct FailingAddon;

        #[async_trait::async_trait]
        impl Addon for FailingAddon {
            async fn requestheaders(&mut self, _flow: &mut FlowBase) -> Result<(), AddonError> {
                Err(AddonError::Execution("test error".to_string()))
            }
        }

        mgr.register(Box::new(FailingAddon));

        let mut flow = test_flow();
        let result = mgr.dispatch_requestheaders(&mut flow).await;
        assert!(result.is_err());
    }
}
