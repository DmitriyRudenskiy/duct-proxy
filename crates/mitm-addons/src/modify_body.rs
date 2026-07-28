//! ModifyBody addon for modifying HTTP request/response bodies.

use crate::addon::{Addon, AddonError};
use mitm_core::FlowBase;
use regex::Regex;

/// Replace operation for body modification.
#[derive(Debug, Clone)]
pub struct ReplaceOp {
    /// Pattern to find (string or regex).
    pub pattern: String,
    /// Replacement string.
    pub replacement: String,
    /// Whether the pattern is a regex.
    pub is_regex: bool,
    /// Compiled regex (if is_regex is true).
    pub compiled_regex: Option<Regex>,
}

/// Addon to modify HTTP request/response bodies.
pub struct ModifyBody {
    /// Replace operations.
    replacements: Vec<ReplaceOp>,
    /// Content-Type filter (if set, only modify matching content types).
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
        self.replacements.push(ReplaceOp {
            pattern: pattern.into(),
            replacement: replacement.into(),
            is_regex: false,
            compiled_regex: None,
        });
        self
    }

    /// Add a regex replacement.
    pub fn replace_regex(mut self, pattern: impl AsRef<str>, replacement: impl Into<String>) -> Result<Self, regex::Error> {
        let regex = Regex::new(pattern.as_ref())?;
        self.replacements.push(ReplaceOp {
            pattern: pattern.as_ref().to_string(),
            replacement: replacement.into(),
            is_regex: true,
            compiled_regex: Some(regex),
        });
        Ok(self)
    }

    /// Filter by Content-Type header.
    pub fn filter_by_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type_filter = Some(content_type.into());
        self
    }

    /// Check if a content type matches the filter.
    pub fn matches_content_type(&self, content_type: &str) -> bool {
        match &self.content_type_filter {
            Some(filter) => content_type.contains(filter),
            None => true,
        }
    }

    /// Apply replacements to a body string.
    pub fn apply_replacements(&self, body: &str) -> String {
        let mut result = body.to_string();
        for replace_op in &self.replacements {
            if replace_op.is_regex {
                if let Some(regex) = &replace_op.compiled_regex {
                    result = regex.replace_all(&result, replace_op.replacement.as_str()).to_string();
                }
            } else {
                result = result.replace(&replace_op.pattern, &replace_op.replacement);
            }
        }
        result
    }
}

impl Default for ModifyBody {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Addon for ModifyBody {
    async fn request(&mut self, _flow: &mut FlowBase) -> Result<(), AddonError> {
        // TODO: Modify request body from flow
        Ok(())
    }

    async fn response(&mut self, _flow: &mut FlowBase) -> Result<(), AddonError> {
        // TODO: Modify response body from flow
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_string() {
        let addon = ModifyBody::new().replace("old", "new");
        assert_eq!(addon.apply_replacements("this is old text"), "this is new text");
    }

    #[test]
    fn test_replace_regex() {
        let addon = ModifyBody::new()
            .replace_regex(r"\d+", "NUM")
            .unwrap();
        assert_eq!(addon.apply_replacements("abc 123 def 456"), "abc NUM def NUM");
    }

    #[test]
    fn test_replace_multiple() {
        let addon = ModifyBody::new()
            .replace("foo", "bar")
            .replace("baz", "qux");
        assert_eq!(addon.apply_replacements("foo and baz"), "bar and qux");
    }

    #[test]
    fn test_content_type_filter() {
        let addon = ModifyBody::new()
            .filter_by_content_type("text/html")
            .replace("old", "new");
        assert!(addon.matches_content_type("text/html"));
        assert!(addon.matches_content_type("text/html; charset=utf-8"));
        assert!(!addon.matches_content_type("application/json"));
    }

    #[test]
    fn test_no_content_type_filter() {
        let addon = ModifyBody::new().replace("old", "new");
        assert!(addon.matches_content_type("any/content-type"));
    }

    #[test]
    fn test_empty_body() {
        let addon = ModifyBody::new().replace("old", "new");
        assert_eq!(addon.apply_replacements(""), "");
    }

    #[test]
    fn test_no_match() {
        let addon = ModifyBody::new().replace("old", "new");
        assert_eq!(addon.apply_replacements("no matches here"), "no matches here");
    }
}
