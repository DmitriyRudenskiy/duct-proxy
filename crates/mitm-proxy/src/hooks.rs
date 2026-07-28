//! Hook system for addon integration.

use std::sync::Arc;
use tokio::task::JoinSet;
use tracing::error;

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
}

impl HookDispatcher {
    /// Create a new HookDispatcher.
    pub fn new() -> Self {
        Self {
            request_hooks: Vec::new(),
            response_hooks: Vec::new(),
            error_hooks: Vec::new(),
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

    /// Dispatch requestheaders hook to all registered hooks.
    /// Note: In full implementation, this would pass the actual flow.
    pub async fn dispatch_requestheaders(&self) -> Result<(), HookError> {
        let mut join_set: JoinSet<Result<(), HookError>> = JoinSet::new();

        for _hook in &self.request_hooks {
            join_set.spawn(async move {
                // For now, hooks don't have access to flow
                // This will be updated when flow is passed properly
                Ok(())
            });
        }

        while let Some(result) = join_set.join_next().await {
            if let Err(e) = result {
                error!("Hook task panicked: {}", e);
            }
        }

        Ok(())
    }

    /// Dispatch request hook to all registered hooks.
    pub async fn dispatch_request(&self) -> Result<(), HookError> {
        let mut join_set: JoinSet<Result<(), HookError>> = JoinSet::new();

        for _hook in &self.request_hooks {
            join_set.spawn(async move {
                Ok(())
            });
        }

        while let Some(result) = join_set.join_next().await {
            if let Err(e) = result {
                error!("Hook task panicked: {}", e);
            }
        }

        Ok(())
    }

    /// Dispatch response hook to all registered hooks.
    pub async fn dispatch_response(&self) -> Result<(), HookError> {
        let mut join_set: JoinSet<Result<(), HookError>> = JoinSet::new();

        for _hook in &self.response_hooks {
            join_set.spawn(async move {
                Ok(())
            });
        }

        while let Some(result) = join_set.join_next().await {
            if let Err(e) = result {
                error!("Hook task panicked: {}", e);
            }
        }

        Ok(())
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
}
