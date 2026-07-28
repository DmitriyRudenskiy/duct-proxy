//! AddonManager for registration and sequential dispatch.

use crate::addon::{Addon, AddonError};
use mitm_proxy::Flow;
use tracing::{debug, error, info, warn};

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
    pub async fn dispatch_requestheaders(&mut self, flow: &mut Flow) -> Result<(), AddonError> {
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
    pub async fn dispatch_request(&mut self, flow: &mut Flow) -> Result<(), AddonError> {
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
    pub async fn dispatch_responseheaders(&mut self, flow: &mut Flow) -> Result<(), AddonError> {
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
    pub async fn dispatch_response(&mut self, flow: &mut Flow) -> Result<(), AddonError> {
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
