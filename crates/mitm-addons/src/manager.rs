//! AddonManager for registration and sequential dispatch.

use tracing::info;

use crate::addon::{Addon, AddonError};
use mitm_proxy::Flow;

/// Manager for addon registration and dispatch.
pub struct AddonManager {
    addons: Vec<Box<dyn Addon>>,
}

impl AddonManager {
    /// Create a new AddonManager.
    pub fn new() -> Self {
        Self {
            addons: Vec::new(),
        }
    }

    /// Register an addon.
    pub fn register(&mut self, addon: Box<dyn Addon>) {
        info!("Registered addon");
        self.addons.push(addon);
    }

    /// Dispatch requestheaders hook to all addons.
    pub async fn dispatch_requestheaders(&mut self, flow: &mut Flow) -> Result<(), AddonError> {
        for addon in &mut self.addons {
            addon.requestheaders(flow).await?;
        }
        Ok(())
    }

    /// Dispatch request hook to all addons.
    pub async fn dispatch_request(&mut self, flow: &mut Flow) -> Result<(), AddonError> {
        for addon in &mut self.addons {
            addon.request(flow).await?;
        }
        Ok(())
    }

    /// Dispatch responseheaders hook to all addons.
    pub async fn dispatch_responseheaders(&mut self, flow: &mut Flow) -> Result<(), AddonError> {
        for addon in &mut self.addons {
            addon.responseheaders(flow).await?;
        }
        Ok(())
    }

    /// Dispatch response hook to all addons.
    pub async fn dispatch_response(&mut self, flow: &mut Flow) -> Result<(), AddonError> {
        for addon in &mut self.addons {
            addon.response(flow).await?;
        }
        Ok(())
    }

    /// Dispatch error hook to all addons.
    pub async fn dispatch_error(&mut self, error: &AddonError) -> Result<(), AddonError> {
        for addon in &mut self.addons {
            addon.error(error).await?;
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
