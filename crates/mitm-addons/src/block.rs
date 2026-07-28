//! Block addon for blocking requests matching filters.

use crate::addon::{Addon, AddonError};
use mitm_proxy::Flow;

/// Addon to block requests matching specified criteria.
pub struct Block {
    /// URL patterns to block.
    url_patterns: Vec<String>,
    /// Headers to block.
    header_filters: Vec<(String, String)>,
    /// Source IP ranges to block.
    ip_ranges: Vec<String>,
}

impl Block {
    /// Create a new Block addon.
    pub fn new() -> Self {
        Self {
            url_patterns: Vec::new(),
            header_filters: Vec::new(),
            ip_ranges: Vec::new(),
        }
    }

    /// Block by URL pattern.
    pub fn url_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.url_patterns.push(pattern.into());
        self
    }

    /// Block by header.
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.header_filters.push((name.into(), value.into()));
        self
    }

    /// Block by source IP range.
    pub fn source_ip(mut self, ip_range: impl Into<String>) -> Self {
        self.ip_ranges.push(ip_range.into());
        self
    }
}

impl Default for Block {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Addon for Block {
    async fn requestheaders(&mut self, _flow: &mut Flow) -> Result<(), AddonError> {
        // TODO: Check filters and send 403 if matched
        Ok(())
    }
}
