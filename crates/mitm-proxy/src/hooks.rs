//! Hook system for addon integration.

use std::sync::Arc;
use tracing::debug;

/// Hook error type.
#[derive(Debug, thiserror::Error)]
pub enum HookError {
    #[error("Hook execution failed: {0}")]
    Execution(String),

    #[error("Hook timeout after {0}ms")]
    Timeout(u64),

    #[error("Flow not found")]
    FlowNotFound,
}

/// Trait for HTTP request hooks.
#[async_trait::async_trait]
pub trait HttpRequestHook: Send + Sync {
    /// Called when request headers are received.
    async fn requestheaders(&self, _flow: &mut super::flows::Flow) -> Result<(), HookError>;

    /// Called when request body is complete.
    async fn request(&self, _flow: &mut super::flows::Flow) -> Result<(), HookError>;
}

/// Trait for HTTP response hooks.
#[async_trait::async_trait]
pub trait HttpResponseHook: Send + Sync {
    /// Called when response is received from upstream.
    async fn response(&self, _flow: &mut super::flows::Flow) -> Result<(), HookError>;
}

/// Trait for error hooks.
#[async_trait::async_trait]
pub trait ErrorHook: Send + Sync {
    /// Called when an error occurs during flow processing.
    async fn error(&self, _error: &HookError, _flow: Option<&mut super::flows::Flow>) -> Result<(), HookError>;
}

/// Hook dispatcher for managing and invoking hooks.
pub struct HookDispatcher {
    /// Registered request hooks.
    request_hooks: Vec<Arc<dyn HttpRequestHook>>,
    /// Registered response hooks.
    response_hooks: Vec<Arc<dyn HttpResponseHook>>,
    /// Registered error hooks.
    error_hooks: Vec<Arc<dyn ErrorHook>>,
    /// Addon manager for addon-based hooks.
    addon_manager: Option<mitm_addons::AddonManager>,
}

impl HookDispatcher {
    /// Create a new HookDispatcher.
    pub fn new() -> Self {
        Self {
            request_hooks: Vec::new(),
            response_hooks: Vec::new(),
            error_hooks: Vec::new(),
            addon_manager: None,
        }
    }

    /// Create a new HookDispatcher with an AddonManager.
    pub fn with_addon_manager(addon_manager: mitm_addons::AddonManager) -> Self {
        Self {
            request_hooks: Vec::new(),
            response_hooks: Vec::new(),
            error_hooks: Vec::new(),
            addon_manager: Some(addon_manager),
        }
    }

    /// Register a request hook.
    pub fn register_request_hook<H: HttpRequestHook + 'static>(&mut self, hook: H) {
        self.request_hooks.push(Arc::new(hook));
    }

    /// Register a response hook.
    pub fn register_response_hook<H: HttpResponseHook + 'static>(&mut self, hook: H) {
        self.response_hooks.push(Arc::new(hook));
    }

    /// Register an error hook.
    pub fn register_error_hook<H: ErrorHook + 'static>(&mut self, hook: H) {
        self.error_hooks.push(Arc::new(hook));
    }

    /// Dispatch requestheaders hook to all registered hooks and addons.
    pub async fn dispatch_requestheaders(&self, flow: &mut super::flows::Flow) -> Result<(), HookError> {
        debug!("Dispatching requestheaders to {} hooks and {} addons", self.request_hooks.len(), self.addon_manager.as_ref().map_or(0, |m| m.len()));

        // Dispatch to traditional hooks.
        for hook in &self.request_hooks {
            hook.requestheaders(flow).await?;
        }

        // Dispatch to addon manager if present.
        if let Some(addon_mgr) = &self.addon_manager {
            // Note: AddonManager uses mitm_addons::AddonError, not HookError.
            // We'll convert or just log for now.
            debug!("AddonManager has {} addons for requestheaders", addon_mgr.len());
        }

        Ok(())
    }

    /// Dispatch request hook to all registered hooks and addons.
    pub async fn dispatch_request(&self, flow: &mut super::flows::Flow) -> Result<(), HookError> {
        debug!("Dispatching request to {} hooks and {} addons", self.request_hooks.len(), self.addon_manager.as_ref().map_or(0, |m| m.len()));

        // Dispatch to traditional hooks.
        for hook in &self.request_hooks {
            hook.request(flow).await?;
        }

        // Dispatch to addon manager if present.
        if let Some(addon_mgr) = &self.addon_manager {
            debug!("AddonManager has {} addons for request", addon_mgr.len());
        }

        Ok(())
    }

    /// Dispatch response hook to all registered hooks and addons.
    pub async fn dispatch_response(&self, flow: &mut super::flows::Flow) -> Result<(), HookError> {
        debug!("Dispatching response to {} hooks and {} addons", self.response_hooks.len(), self.addon_manager.as_ref().map_or(0, |m| m.len()));

        // Dispatch to traditional hooks.
        for hook in &self.response_hooks {
            hook.response(flow).await?;
        }

        // Dispatch to addon manager if present.
        if let Some(addon_mgr) = &self.addon_manager {
            debug!("AddonManager has {} addons for response", addon_mgr.len());
        }

        Ok(())
    }

    /// Get a reference to the addon manager (if present).
    pub fn addon_manager(&self) -> Option<&mitm_addons::AddonManager> {
        self.addon_manager.as_ref()
    }

    /// Get a mutable reference to the addon manager (if present).
    pub fn addon_manager_mut(&mut self) -> Option<&mut mitm_addons::AddonManager> {
        self.addon_manager.as_mut()
    }
}

impl Default for HookDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hook_dispatcher_creation() {
        let _dispatcher = HookDispatcher::new();
        // Should not panic
    }

    #[tokio::test]
    async fn test_hook_dispatcher_with_addon_manager() {
        let addon_mgr = mitm_addons::AddonManager::new();
        let _dispatcher = HookDispatcher::with_addon_manager(addon_mgr);
        // Should not panic
    }
}
