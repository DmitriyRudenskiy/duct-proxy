//! ModifyHeaders addon for modifying HTTP headers.

use crate::addon::{Addon, AddonError};
use mitm_proxy::Flow;

/// Addon to modify HTTP request/response headers.
pub struct ModifyHeaders {
    /// Headers to add.
    add_headers: Vec<(String, String)>,
    /// Headers to set (replace if exists).
    set_headers: Vec<(String, String)>,
    /// Header names to remove.
    remove_headers: Vec<String>,
}

impl ModifyHeaders {
    /// Create a new ModifyHeaders addon.
    pub fn new() -> Self {
        Self {
            add_headers: Vec::new(),
            set_headers: Vec::new(),
            remove_headers: Vec::new(),
        }
    }

    /// Add a header.
    pub fn add(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.add_headers.push((name.into(), value.into()));
        self
    }

    /// Set a header (replace if exists).
    pub fn set(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.set_headers.push((name.into(), value.into()));
        self
    }

    /// Remove a header.
    pub fn remove(mut self, name: impl Into<String>) -> Self {
        self.remove_headers.push(name.into());
        self
    }
}

impl Default for ModifyHeaders {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Addon for ModifyHeaders {
    async fn requestheaders(&mut self, _flow: &mut Flow) -> Result<(), AddonError> {
        // TODO: Modify request headers
        Ok(())
    }

    async fn responseheaders(&mut self, _flow: &mut Flow) -> Result<(), AddonError> {
        // TODO: Modify response headers
        Ok(())
    }
}
