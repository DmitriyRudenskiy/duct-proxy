//! ModifyHeaders addon for modifying HTTP headers.

use crate::addon::{Addon, AddonError};
use mitm_core::FlowBase;
use regex::Regex;
use std::collections::HashMap;

/// Action to perform on a header.
#[derive(Debug, Clone)]
pub enum HeaderAction {
    /// Add a header (append if exists).
    Add {
        name: String,
        value: String,
    },
    /// Set a header (replace if exists).
    Set {
        name: String,
        value: String,
    },
    /// Remove a header.
    Remove {
        name: String,
    },
}

/// Addon to modify HTTP request/response headers.
pub struct ModifyHeaders {
    /// Actions to perform on headers.
    actions: Vec<HeaderAction>,
    /// Regex patterns for header name matching.
    regex_patterns: Vec<(Regex, HeaderAction)>,
}

impl ModifyHeaders {
    /// Create a new ModifyHeaders addon.
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
            regex_patterns: Vec::new(),
        }
    }

    /// Add a header.
    pub fn add(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.actions.push(HeaderAction::Add {
            name: name.into(),
            value: value.into(),
        });
        self
    }

    /// Set a header (replace if exists).
    pub fn set(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.actions.push(HeaderAction::Set {
            name: name.into(),
            value: value.into(),
        });
        self
    }

    /// Remove a header.
    pub fn remove(mut self, name: impl Into<String>) -> Self {
        self.actions.push(HeaderAction::Remove {
            name: name.into(),
        });
        self
    }

    /// Add a header with regex pattern matching.
    pub fn add_regex(mut self, pattern: impl AsRef<str>, name: impl Into<String>, value: impl Into<String>) -> Result<Self, regex::Error> {
        let regex = Regex::new(pattern.as_ref())?;
        self.regex_patterns.push((
            regex,
            HeaderAction::Add {
                name: name.into(),
                value: value.into(),
            },
        ));
        Ok(self)
    }

    /// Set a header with regex pattern matching.
    pub fn set_regex(mut self, pattern: impl AsRef<str>, name: impl Into<String>, value: impl Into<String>) -> Result<Self, regex::Error> {
        let regex = Regex::new(pattern.as_ref())?;
        self.regex_patterns.push((
            regex,
            HeaderAction::Set {
                name: name.into(),
                value: value.into(),
            },
        ));
        Ok(self)
    }

    /// Remove a header with regex pattern matching.
    pub fn remove_regex(mut self, pattern: impl AsRef<str>, name: impl Into<String>) -> Result<Self, regex::Error> {
        let regex = Regex::new(pattern.as_ref())?;
        self.regex_patterns.push((
            regex,
            HeaderAction::Remove {
                name: name.into(),
            },
        ));
        Ok(self)
    }

    /// Apply actions to a headers map (helper method for testing).
    fn apply_to_headers(&self, headers: &mut HashMap<String, String>) {
        // Apply exact name actions first.
        for action in &self.actions {
            self.apply_action(headers, action);
        }

        // Then apply regex actions.
        for (regex, action) in &self.regex_patterns {
            // First, check if any existing headers match.
            let mut matched_names: Vec<String> = Vec::new();
            for name in headers.keys() {
                if regex.is_match(name) {
                    matched_names.push(name.clone());
                }
            }
            
            // For Add actions, if no existing headers match, add the new header.
            // For Set/Remove actions, only modify existing headers.
            match action {
                HeaderAction::Add { name, value } => {
                    if matched_names.is_empty() {
                        // No existing headers match, so add the new header.
                        headers.insert(name.clone(), value.clone());
                    }
                    // If there are matching headers, we don't add a new one to avoid duplicates.
                }
                HeaderAction::Set { name, value } => {
                    if matched_names.is_empty() {
                        // No existing headers match, so set the new header.
                        headers.insert(name.clone(), value.clone());
                    } else {
                        // Set all matching headers to the new value.
                        for name in &matched_names {
                            headers.insert(name.clone(), value.clone());
                        }
                    }
                }
                HeaderAction::Remove { name } => {
                    // Remove all matching headers.
                    for name in &matched_names {
                        headers.remove(name);
                    }
                }
            }
        }
    }

    /// Apply a single action to the headers map.
    fn apply_action(&self, headers: &mut HashMap<String, String>, action: &HeaderAction) {
        match action {
            HeaderAction::Add { name, value } => {
                headers
                    .entry(name.clone())
                    .and_modify(|v| {
                        v.push_str(", ");
                        v.push_str(value);
                    })
                    .or_insert_with(|| value.clone());
            }
            HeaderAction::Set { name, value } => {
                headers.insert(name.clone(), value.clone());
            }
            HeaderAction::Remove { name } => {
                headers.remove(name);
            }
        }
    }
}

impl Default for ModifyHeaders {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Addon for ModifyHeaders {
    async fn requestheaders(&mut self, _flow: &mut FlowBase) -> Result<(), AddonError> {
        // TODO: Modify request headers from flow
        Ok(())
    }

    async fn responseheaders(&mut self, _flow: &mut FlowBase) -> Result<(), AddonError> {
        // TODO: Modify response headers from flow
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_header() {
        let mut headers = HashMap::new();
        let addon = ModifyHeaders::new().add("X-Test", "value");
        addon.apply_to_headers(&mut headers);
        assert_eq!(headers.get("X-Test"), Some(&"value".to_string()));
    }

    #[test]
    fn test_set_header() {
        let mut headers = HashMap::from([("X-Test".to_string(), "old".to_string())]);
        let addon = ModifyHeaders::new().set("X-Test", "new");
        addon.apply_to_headers(&mut headers);
        assert_eq!(headers.get("X-Test"), Some(&"new".to_string()));
    }

    #[test]
    fn test_remove_header() {
        let mut headers = HashMap::from([("X-Test".to_string(), "value".to_string())]);
        let addon = ModifyHeaders::new().remove("X-Test");
        addon.apply_to_headers(&mut headers);
        assert!(!headers.contains_key("X-Test"));
    }

    #[test]
    fn test_add_regex() {
        let mut headers = HashMap::new();
        let addon = ModifyHeaders::new()
            .add_regex("X-Custom-.*", "X-Custom-Test", "value")
            .unwrap();
        addon.apply_to_headers(&mut headers);
        assert_eq!(headers.get("X-Custom-Test"), Some(&"value".to_string()));
    }

    #[test]
    fn test_set_regex() {
        let mut headers = HashMap::from([("X-Custom-Test".to_string(), "old".to_string())]);
        let addon = ModifyHeaders::new()
            .set_regex("X-Custom-.*", "X-Custom-Test", "new")
            .unwrap();
        addon.apply_to_headers(&mut headers);
        assert_eq!(headers.get("X-Custom-Test"), Some(&"new".to_string()));
    }

    #[test]
    fn test_remove_regex() {
        let mut headers = HashMap::from([("X-Custom-Test".to_string(), "value".to_string())]);
        let addon = ModifyHeaders::new()
            .remove_regex("X-Custom-.*", "X-Custom-Test")
            .unwrap();
        addon.apply_to_headers(&mut headers);
        assert!(!headers.contains_key("X-Custom-Test"));
    }

    #[test]
    fn test_multiple_actions() {
        let mut headers = HashMap::from([
            ("X-Old".to_string(), "value".to_string()),
            ("X-Keep".to_string(), "value".to_string()),
        ]);
        let addon = ModifyHeaders::new()
            .remove("X-Old")
            .set("X-New", "value")
            .add("X-Append", "first");
        addon.apply_to_headers(&mut headers);
        assert!(!headers.contains_key("X-Old"));
        assert_eq!(headers.get("X-New"), Some(&"value".to_string()));
        assert_eq!(headers.get("X-Append"), Some(&"first".to_string()));
        assert_eq!(headers.get("X-Keep"), Some(&"value".to_string()));
    }
}
