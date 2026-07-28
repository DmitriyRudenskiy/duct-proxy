//! ModifyBody addon for modifying HTTP request/response bodies.

use crate::addon::{Addon, AddonError};
use mitm_proxy::Flow;

/// Addon to modify HTTP request/response bodies.
pub struct ModifyBody {
    /// String replacements.
    replacements: Vec<(String, String)>,
    /// Content-Type filter.
    content_type_filter: Option<String>,
}

impl ModifyBody {
    /// Create a new ModifyBody addon.
    pub fn new() -> Self {
        Self {
            replacements: Vec::new(),
            content_type_filter: None,
        }
    }

    /// Add a string replacement.
    pub fn replace(mut self, pattern: impl Into<String>, replacement: impl Into<String>) -> Self {
        self.replacements.push((pattern.into(), replacement.into()));
        self
    }

    /// Filter by Content-Type header.
    pub fn filter_by_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type_filter = Some(content_type.into());
        self
    }
}

impl Default for ModifyBody {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Addon for ModifyBody {
    async fn request(&mut self, _flow: &mut Flow) -> Result<(), AddonError> {
        // TODO: Modify request body
        Ok(())
    }

    async fn response(&mut self, _flow: &mut Flow) -> Result<(), AddonError> {
        // TODO: Modify response body
        Ok(())
    }
}
