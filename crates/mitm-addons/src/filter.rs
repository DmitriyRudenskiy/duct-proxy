//! Filter addon for filtering flows based on expressions.

use crate::addon::{Addon, AddonError};
use mitm_proxy::Flow;

/// Addon to filter flows based on URL, method, or header criteria.
pub struct Filter {
    /// URL pattern filter.
    url_pattern: Option<String>,
    /// HTTP method filter.
    method: Option<String>,
    /// Header filters.
    header_filters: Vec<(String, String)>,
}

impl Filter {
    /// Create a new Filter addon.
    pub fn new() -> Self {
        Self {
            url_pattern: None,
            method: None,
            header_filters: Vec::new(),
        }
    }

    /// Filter by URL pattern.
    pub fn url(mut self, pattern: impl Into<String>) -> Self {
        self.url_pattern = Some(pattern.into());
        self
    }

    /// Filter by HTTP method.
    pub fn method(mut self, method: impl Into<String>) -> Self {
        self.method = Some(method.into());
        self
    }

    /// Filter by header.
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.header_filters.push((name.into(), value.into()));
        self
    }

    /// Check if a flow matches this filter.
    pub fn matches(&self, _flow: &Flow) -> bool {
        // TODO: Implement filter matching logic
        true
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Addon for Filter {
    async fn requestheaders(&mut self, _flow: &mut Flow) -> Result<(), AddonError> {
        // TODO: Check if flow matches filter
        Ok(())
    }
}
