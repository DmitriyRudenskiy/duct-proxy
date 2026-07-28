//! Block addon for blocking requests matching filters.

use crate::addon::{Addon, AddonError};
use mitm_core::FlowBase;
use regex::Regex;

/// Filter criteria for blocking requests.
#[derive(Debug, Clone)]
pub enum BlockFilter {
    /// Block by URL pattern.
    Url {
        pattern: String,
    },
    /// Block by HTTP method.
    Method {
        method: String,
    },
    /// Block by header.
    Header {
        name: String,
        value: Option<String>,
    },
    /// Block by source IP pattern.
    SourceIp {
        pattern: String,
    },
}

/// Addon to block requests matching specified criteria.
pub struct Block {
    /// Filter criteria.
    filters: Vec<BlockFilter>,
    /// Compiled regex patterns for URL and IP filters.
    compiled_filters: Vec<BlockCompiledFilter>,
}

/// A compiled block filter for efficient matching.
#[derive(Debug, Clone)]
struct BlockCompiledFilter {
    filter: BlockFilter,
    regex: Option<Regex>,
}

impl Block {
    /// Create a new Block addon.
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
            compiled_filters: Vec::new(),
        }
    }

    /// Block by URL pattern.
    pub fn url(mut self, pattern: impl Into<String>) -> Self {
        let pattern = pattern.into();
        self.filters.push(BlockFilter::Url {
            pattern: pattern.clone(),
        });
        self
    }

    /// Block by HTTP method.
    pub fn method(mut self, method: impl Into<String>) -> Self {
        let method = method.into();
        self.filters.push(BlockFilter::Method {
            method: method.clone(),
        });
        self
    }

    /// Block by header name.
    pub fn header(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        self.filters.push(BlockFilter::Header {
            name: name.clone(),
            value: None,
        });
        self
    }

    /// Block by header name and value.
    pub fn header_with_value(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        let name = name.into();
        let value = value.into();
        self.filters.push(BlockFilter::Header {
            name: name.clone(),
            value: Some(value.clone()),
        });
        self
    }

    /// Block by source IP pattern.
    pub fn source_ip(mut self, pattern: impl Into<String>) -> Self {
        let pattern = pattern.into();
        self.filters.push(BlockFilter::SourceIp {
            pattern: pattern.clone(),
        });
        self
    }

    /// Compile all regex patterns.
    pub fn compile(&mut self) -> Result<(), regex::Error> {
        self.compiled_filters = self.filters.iter().map(|f| {
            let regex = match f {
                BlockFilter::Url { pattern } => Regex::new(pattern).ok(),
                BlockFilter::SourceIp { pattern } => Regex::new(pattern).ok(),
                _ => None,
            };
            BlockCompiledFilter {
                filter: f.clone(),
                regex,
            }
        }).collect();
        Ok(())
    }

    /// Check if a flow should be blocked based on filters.
    fn check_flow(&self, flow: &FlowBase) -> bool {
        // Extract information from flow to check against filters
        let source_ip = flow.client_conn.connection.peername.as_ref().map(|(host, _)| host.clone());
        
        // For now, we can only check source IP from FlowBase
        // URL and headers would require HTTP-specific access
        self.should_block(None, None, &[], source_ip.as_deref())
    }

    /// Check if a request matches any block filter.
    pub fn should_block(
        &self,
        url: Option<&str>,
        method: Option<&str>,
        headers: &[(String, String)],
        source_ip: Option<&str>,
    ) -> bool {
        for compiled in &self.compiled_filters {
            match &compiled.filter {
                BlockFilter::Url { pattern } => {
                    if let Some(url_pattern) = url {
                        if let Some(regex) = &compiled.regex {
                            if regex.is_match(url_pattern) {
                                return true;
                            }
                        } else if pattern == url_pattern {
                            return true;
                        }
                    }
                }
                BlockFilter::Method { method: method_pattern } => {
                    if let Some(method) = method
                        && method == method_pattern {
                            return true;
                        }
                }
                BlockFilter::Header { name, value } => {
                    for (header_name, header_value) in headers {
                        if header_name == name {
                            if let Some(value_pattern) = value {
                                if header_value == value_pattern {
                                    return true;
                                }
                            } else {
                                return true;
                            }
                        }
                    }
                }
                BlockFilter::SourceIp { pattern } => {
                    if let Some(ip_pattern) = source_ip {
                        if let Some(regex) = &compiled.regex {
                            if regex.is_match(ip_pattern) {
                                return true;
                            }
                        } else if pattern == ip_pattern {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}

impl Default for Block {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Addon for Block {
    async fn request(&mut self, flow: &mut FlowBase) -> Result<(), AddonError> {
        // Check if flow should be blocked based on filters
        let should_block = self.check_flow(flow);
        
        if should_block {
            // Mark the flow as blocked via metadata
            flow.metadata.insert(
                "blocked".to_string(),
                serde_json::json!("Blocked by Addon: Block"),
            );
            tracing::info!("Flow blocked by Block addon");
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_by_url() {
        let mut block = Block::new().url("https://evil.com/.*");
        block.compile().unwrap();
        assert!(block.should_block(Some("https://evil.com/admin"), None, &[], None));
        assert!(block.should_block(Some("https://evil.com/public"), None, &[], None));
        assert!(!block.should_block(Some("https://good.com"), None, &[], None));
    }

    #[test]
    fn test_block_by_method() {
        let mut block = Block::new().method("DELETE");
        block.compile().unwrap();
        assert!(block.should_block(None, Some("DELETE"), &[], None));
        assert!(!block.should_block(None, Some("GET"), &[], None));
    }

    #[test]
    fn test_block_by_header_name() {
        let mut block = Block::new().header("X-Bad-Header");
        block.compile().unwrap();
        assert!(block.should_block(None, None, &[("X-Bad-Header".to_string(), "value".to_string())], None));
        assert!(!block.should_block(None, None, &[("X-Good-Header".to_string(), "value".to_string())], None));
    }

    #[test]
    fn test_block_by_header_value() {
        let mut block = Block::new().header_with_value("X-Bad-Header", "banned");
        block.compile().unwrap();
        assert!(block.should_block(None, None, &[("X-Bad-Header".to_string(), "banned".to_string())], None));
        assert!(!block.should_block(None, None, &[("X-Bad-Header".to_string(), "allowed".to_string())], None));
    }

    #[test]
    fn test_block_by_source_ip() {
        let mut block = Block::new().source_ip("^192\\.168\\.1\\.100$");
        block.compile().unwrap();
        assert!(block.should_block(None, None, &[], Some("192.168.1.100")));
        assert!(!block.should_block(None, None, &[], Some("10.0.0.1")));
    }

    #[test]
    fn test_block_no_match() {
        let mut block = Block::new().url("https://evil.com/.*").method("DELETE");
        block.compile().unwrap();
        assert!(!block.should_block(Some("https://good.com"), None, &[], None));
        assert!(!block.should_block(None, Some("GET"), &[], None));
        assert!(!block.should_block(None, None, &[], Some("10.0.0.1")));
    }

    #[tokio::test]
    async fn test_block_sets_metadata_flag() {
        use mitm_core::connection::Client;
        use mitm_core::flow::FlowBase;

        let mut block = Block::new().source_ip("^192\\.168\\.1\\.100$");
        block.compile().unwrap();

        let mut flow = FlowBase::new(
            Client::new(
                ("192.168.1.100".to_string(), 12345),
                ("10.0.0.1".to_string(), 80),
                "regular",
            ),
            mitm_core::connection::Server::new(),
            true,
        );

        block.request(&mut flow).await.unwrap();
        
        // Check that the flow was marked as blocked
        assert!(flow.metadata.contains_key("blocked"));
        assert_eq!(
            flow.metadata.get("blocked").unwrap().as_str().unwrap(),
            "Blocked by Addon: Block"
        );
    }

    #[tokio::test]
    async fn test_block_no_match_no_metadata() {
        use mitm_core::connection::Client;
        use mitm_core::flow::FlowBase;

        let mut block = Block::new().source_ip("^192\\.168\\.1\\.100$");
        block.compile().unwrap();

        let mut flow = FlowBase::new(
            Client::new(
                ("10.0.0.1".to_string(), 12345),
                ("10.0.0.1".to_string(), 80),
                "regular",
            ),
            mitm_core::connection::Server::new(),
            true,
        );

        block.request(&mut flow).await.unwrap();
        
        // Check that the flow was NOT marked as blocked
        assert!(!flow.metadata.contains_key("blocked"));
    }
}
