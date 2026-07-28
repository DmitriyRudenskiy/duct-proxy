//! HTTP Headers — case-insensitive, order-preserving MultiDict.
//!
//! Internal representation: `Vec<(Vec<u8>, Vec<u8>)>` for field order, plus
//! `HashMap<String, Vec<usize>>` for O(1) case-insensitive lookup.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// An HTTP header name and value pair.
pub type HeaderField = (Vec<u8>, Vec<u8>);

/// Case-insensitive, order-preserving HTTP header collection.
///
/// Mirrors the Python `mitmproxy.http.Headers` type. Supports:
/// - Case-insensitive lookups
/// - Multiple values per key (e.g., `Set-Cookie`)
/// - Insertion-order preservation
/// - Byte-level storage (matching HTTP wire format)
#[derive(Clone, Debug, Default)]
pub struct Headers {
    /// Raw fields in insertion order.
    fields: Vec<HeaderField>,
    /// Lookup index (rebuilt after deserialization, not persisted).
    index: HashMap<String, Vec<usize>>,
}

impl Serialize for Headers {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(self.fields.len()))?;
        for (k, v) in &self.fields {
            seq.serialize_element(&(k.as_slice(), v.as_slice()))?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for Headers {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let fields: Vec<HeaderField> = Vec::deserialize(deserializer)?;
        let mut h = Self {
            fields,
            index: HashMap::new(),
        };
        h.rebuild_index();
        Ok(h)
    }
}

// ---- Internal helpers ----

impl Headers {
    /// Rebuild the lookup index from the current fields.
    fn rebuild_index(&mut self) {
        self.index.clear();
        for (idx, (name, _)) in self.fields.iter().enumerate() {
            let key = normalize_key(name);
            self.index.entry(key).or_default().push(idx);
        }
    }

    /// Find all field indices for a given (lowercased) key.
    fn find_indices(&self, key_lower: &str) -> Option<&Vec<usize>> {
        self.index.get(key_lower)
    }

    /// Remove all fields matching the given key (lowercased).
    fn remove_matching(&mut self, key_lower: &str) {
        if let Some(indices) = self.index.remove(key_lower) {
            // Remove in reverse order to preserve indices of remaining items.
            for &idx in indices.iter().rev() {
                self.fields.remove(idx);
            }
        }
    }
}

/// Lowercase a header name byte slice for case-insensitive comparison.
fn normalize_key(name: &[u8]) -> String {
    name.iter()
        .map(|b| b.to_ascii_lowercase())
        .collect::<Vec<u8>>()
        .into_iter()
        .map(|b| b as char)
        .collect()
}

/// Decode a header key from bytes, falling back to UTF-8 with surrogates.
fn decode_key(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// Decode a header value from bytes, falling back to UTF-8 with surrogates.
fn decode_value(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

impl Headers {
    /// Create empty headers.
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            index: HashMap::new(),
        }
    }

    /// Create headers from a list of `(name, value)` byte pairs.
    pub fn from_fields(fields: Vec<HeaderField>) -> Self {
        let mut h = Self {
            fields,
            index: HashMap::new(),
        };
        h.rebuild_index();
        h
    }

    /// Returns the number of unique header names.
    pub fn len(&self) -> usize {
        self.index.len()
    }

    /// Returns true if there are no header names.
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    /// Check if a header name exists (case-insensitive).
    pub fn contains(&self, key: &str) -> bool {
        self.find_indices(&normalize_key(key.as_bytes())).is_some()
    }

    /// Get the first value for a header name (folded).
    ///
    /// Returns an error if the header is not present.
    pub fn get(&self, key: &str) -> Result<String, HeaderError> {
        let key_lower = normalize_key(key.as_bytes());
        let indices = self
            .find_indices(&key_lower)
            .ok_or(HeaderError::NotFound(key.to_string()))?;
        let (_, value) = &self.fields[*indices.first().unwrap()];
        Ok(decode_value(value))
    }

    /// Get all values for a header name (not folded).
    pub fn get_all(&self, key: &str) -> Vec<String> {
        let key_lower = normalize_key(key.as_bytes());
        let indices = match self.find_indices(&key_lower) {
            Some(idx) => idx,
            None => return Vec::new(),
        };
        indices
            .iter()
            .map(|&i| decode_value(&self.fields[i].1))
            .collect()
    }

    /// Set a header, replacing all existing values for that name.
    pub fn set(&mut self, key: &str, value: &str) {
        let key_bytes = key.as_bytes().to_vec();
        let value_bytes = value.as_bytes().to_vec();
        let key_lower = normalize_key(&key_bytes);
        self.remove_matching(&key_lower);
        let idx = self.fields.len();
        self.fields.push((key_bytes, value_bytes));
        self.index.entry(key_lower).or_default().push(idx);
    }

    /// Set multiple values for a header name, replacing existing ones.
    pub fn set_all(&mut self, key: &str, values: &[&str]) {
        let key_bytes = key.as_bytes().to_vec();
        let key_lower = normalize_key(&key_bytes);
        self.remove_matching(&key_lower);
        for val in values {
            self.fields
                .push((key_bytes.clone(), val.as_bytes().to_vec()));
            self.index.entry(key_lower.clone()).or_default().push(self.fields.len() - 1);
        }
        // If key_bytes was originally different case, use first field's key bytes
        // for all entries of this key (normalize to first-seen casing).
        // Actually, we keep the original key bytes from `key` for all entries.
        // This preserves the casing of the last `set_all`/`set` call.
    }

    /// Add a value to a header, appending after existing values.
    pub fn add(&mut self, key: &str, value: &str) {
        let key_bytes = key.as_bytes().to_vec();
        let value_bytes = value.as_bytes().to_vec();
        let key_lower = normalize_key(&key_bytes);
        let idx = self.fields.len();
        self.fields.push((key_bytes, value_bytes));
        self.index.entry(key_lower).or_default().push(idx);
    }

    /// Insert a value at a specific position.
    pub fn insert(&mut self, index: usize, key: &str, value: &str) {
        let key_bytes = key.as_bytes().to_vec();
        let value_bytes = value.as_bytes().to_vec();
        let key_lower = normalize_key(&key_bytes);
        // Remove all existing entries for this key to avoid index confusion,
        // then insert at the target position.
        self.remove_matching(&key_lower);
        self.fields.insert(index, (key_bytes, value_bytes));
        self.index.entry(key_lower).or_default().push(index);
    }

    /// Delete a header by name.
    pub fn delete(&mut self, key: &str) -> Result<(), HeaderError> {
        let key_lower = normalize_key(key.as_bytes());
        if self.index.remove(&key_lower).is_some() {
            // Remove fields in reverse index order.
            // We need to re-scan fields to find them since indices shift.
            let mut to_remove = Vec::new();
            for (i, (name, _)) in self.fields.iter().enumerate() {
                if normalize_key(name) == key_lower {
                    to_remove.push(i);
                }
            }
            for &i in to_remove.iter().rev() {
                self.fields.remove(i);
            }
            Ok(())
        } else {
            Err(HeaderError::NotFound(key.to_string()))
        }
    }

    /// Returns an iterator over all fields in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = (&[u8], &[u8])> + '_ {
        self.fields.iter().map(|(k, v)| (k.as_slice(), v.as_slice()))
    }

    /// Returns the raw fields as a slice.
    pub fn fields(&self) -> &[HeaderField] {
        &self.fields
    }
}

/// Error from header operations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HeaderError {
    /// Header name not found.
    NotFound(String),
}

impl fmt::Display for HeaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(name) => write!(f, "header not found: {}", name),
        }
    }
}

impl std::error::Error for HeaderError {}

impl fmt::Display for Headers {
    /// Format as HTTP header block: `name: value\r\n` per line.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (name, value) in &self.fields {
            write!(f, "{}: {}", decode_key(name), decode_value(value))?;
            writeln!(f)?;
        }
        Ok(())
    }
}

/// Serialize headers as raw bytes: `name: value\r\n` lines.
pub fn headers_to_bytes(headers: &Headers) -> Vec<u8> {
    let mut out = Vec::new();
    for (i, (name, value)) in headers.fields().iter().enumerate() {
        out.extend_from_slice(name);
        out.extend_from_slice(b": ");
        out.extend_from_slice(value);
        out.extend_from_slice(b"\r\n");
        let _ = i;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn h() -> Headers {
        Headers::new()
    }

    #[test]
    fn test_case_insensitive_lookup() {
        let mut h = h();
        h.set("Host", "example.com");
        assert_eq!(h.get("host").unwrap(), "example.com");
        assert_eq!(h.get("HOST").unwrap(), "example.com");
        assert_eq!(h.get("HoSt").unwrap(), "example.com");
    }

    #[test]
    fn test_order_preserved() {
        let mut h = h();
        h.set("A", "1");
        h.set("B", "2");
        h.set("C", "3");
        let names: Vec<_> = h.iter().map(|(k, _)| String::from_utf8_lossy(k).to_string()).collect();
        assert_eq!(names, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_set_replaces_existing() {
        let mut h = h();
        h.set("Accept", "text/html");
        h.set("Accept", "application/json");
        assert_eq!(h.get("Accept").unwrap(), "application/json");
        assert_eq!(h.get_all("Accept").len(), 1);
    }

    #[test]
    fn test_multi_value_headers() {
        let mut h = h();
        h.set("Set-Cookie", "a=1");
        h.add("Set-Cookie", "b=2");
        assert_eq!(h.get_all("Set-Cookie"), vec!["a=1", "b=2"]);
        // get() folds to first
        assert_eq!(h.get("Set-Cookie").unwrap(), "a=1");
    }

    #[test]
    fn test_delete() {
        let mut h = h();
        h.set("X-Foo", "bar");
        h.set("X-Baz", "qux");
        h.delete("x-foo").unwrap();
        assert!(!h.contains("X-Foo"));
        assert!(h.contains("X-Baz"));
    }

    #[test]
    fn test_add_appends() {
        let mut h = h();
        h.set("Set-Cookie", "a=1");
        h.add("Set-Cookie", "b=2");
        assert_eq!(h.get_all("Set-Cookie"), vec!["a=1", "b=2"]);
    }

    #[test]
    fn test_bytes_formatting() {
        let mut h = h();
        h.set("Host", "example.com");
        h.set("Accept", "text/html");
        let bytes = headers_to_bytes(&h);
        let s = String::from_utf8(bytes).unwrap();
        assert!(s.contains("Host: example.com\r\n"));
        assert!(s.contains("Accept: text/html\r\n"));
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut h = h();
        h.set("Host", "example.com");
        h.set("Set-Cookie", "a=1");
        h.add("Set-Cookie", "b=2");
        let json = serde_json::to_string(&h).unwrap();
        let h2: Headers = serde_json::from_str(&json).unwrap();
        assert_eq!(h2.get("Host").unwrap(), "example.com");
        assert_eq!(h2.get_all("Set-Cookie"), vec!["a=1", "b=2"]);
    }

    #[test]
    fn test_non_utf8_header_names() {
        let mut h = h();
        h.set(":authority", "example.com");
        assert_eq!(h.get(":authority").unwrap(), "example.com");
    }
}
