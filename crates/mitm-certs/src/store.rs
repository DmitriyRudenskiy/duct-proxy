//! Certificate store with LRU eviction.

use crate::ca::CaRoot;
use crate::leaf::LeafCert;
use indexmap::IndexMap;
use std::time::Instant;
use thiserror::Error;

/// Entry in the certificate store.
#[derive(Debug)]
pub struct CertEntry {
    /// Domain name.
    pub domain: String,
    /// Certificate in DER format.
    pub cert_der: Vec<u8>,
    /// Private key in PKCS#8 format.
    pub key_pkcs8: Vec<u8>,
    /// Last access time.
    pub last_access: Instant,
}

/// Certificate store with LRU eviction policy.
pub struct CertStore {
    /// Maximum number of entries.
    max_entries: usize,
    /// Certificate entries indexed by domain.
    entries: IndexMap<String, CertEntry>,
}

/// Errors that can occur during store operations.
#[derive(Error, Debug)]
pub enum StoreError {
    #[error("Store full: cannot add more entries")]
    Full,

    #[error("Domain not found: {0}")]
    NotFound(String),

    #[error("Invalid domain: {0}")]
    InvalidDomain(String),

    #[error("Leaf certificate error: {0}")]
    Leaf(#[from] crate::leaf::LeafError),

    #[error("CA error: {0}")]
    Ca(#[from] crate::ca::CaError),
}

impl CertStore {
    /// Create a new certificate store with the given maximum capacity.
    ///
    /// # Arguments
    /// * `max_entries` - Maximum number of certificates to store
    pub fn new(max_entries: usize) -> Self {
        Self {
            max_entries,
            entries: IndexMap::with_capacity(max_entries),
        }
    }

    /// Insert a leaf certificate into the store.
    ///
    /// # Arguments
    /// * `ca` - CA root certificate
    /// * `domain` - Domain name
    ///
    /// # Returns
    /// Result with LeafCert or StoreError
    pub fn insert(&mut self, ca: &CaRoot, domain: &str) -> Result<LeafCert, StoreError> {
        if domain.is_empty() {
            return Err(StoreError::InvalidDomain("domain cannot be empty".to_string()));
        }

        // If domain already exists, remove it first
        if self.entries.contains_key(domain) {
            self.entries.swap_remove(domain);
        }

        // Generate leaf certificate
        let cert = LeafCert::generate(ca, domain)?;

        // Evict LRU entry if store is full
        if self.entries.len() >= self.max_entries {
            self.evict_lru();
        }

        let entry = CertEntry {
            domain: domain.to_string(),
            cert_der: cert.cert_der().to_vec(),
            key_pkcs8: cert.key_pkcs8().to_vec(),
            last_access: Instant::now(),
        };

        self.entries.insert(domain.to_string(), entry);

        Ok(cert)
    }

    /// Evict the least recently used entry.
    fn evict_lru(&mut self) {
        if let Some((_, entry)) = self.entries.iter().min_by_key(|(_, e)| e.last_access) {
            let domain = entry.domain.clone();
            self.entries.swap_remove(&domain);
        }
    }

    /// Get a certificate from the store by domain.
    ///
    /// Updates the access time on retrieval.
    ///
    /// # Arguments
    /// * `domain` - Domain name
    ///
    /// # Returns
    /// Some(LeafCert) if found, None otherwise
    pub fn get(&mut self, domain: &str) -> Option<LeafCert> {
        if let Some(entry) = self.entries.get_mut(domain) {
            entry.last_access = Instant::now();
            Some(LeafCert {
                cert_der: entry.cert_der.clone(),
                key_pkcs8: entry.key_pkcs8.clone(),
                domain: entry.domain.clone(),
            })
        } else {
            None
        }
    }

    /// Get the number of entries in the store.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Remove a certificate from the store by domain.
    ///
    /// # Arguments
    /// * `domain` - Domain name
    ///
    /// # Returns
    /// Result with removed LeafCert or StoreError
    pub fn remove(&mut self, domain: &str) -> Result<LeafCert, StoreError> {
        if let Some(entry) = self.entries.swap_remove(domain) {
            Ok(LeafCert {
                cert_der: entry.cert_der,
                key_pkcs8: entry.key_pkcs8,
                domain: entry.domain,
            })
        } else {
            Err(StoreError::NotFound(domain.to_string()))
        }
    }

    /// Get the maximum number of entries.
    pub fn max_entries(&self) -> usize {
        self.max_entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ca::CaRoot;

    #[test]
    fn test_store_insert_get() {
        let ca = CaRoot::generate("Test CA").unwrap();
        let mut store = CertStore::new(10);

        let cert = store.insert(&ca, "example.com").unwrap();
        assert_eq!(cert.domain(), "example.com");

        let retrieved = store.get("example.com").unwrap();
        assert_eq!(retrieved.domain(), "example.com");
    }

    #[test]
    fn test_store_lru_eviction() {
        let ca = CaRoot::generate("Test CA").unwrap();
        let mut store = CertStore::new(2);

        store.insert(&ca, "example.com").unwrap();
        store.insert(&ca, "test.com").unwrap();

        // Access example.com to make it recently used
        store.get("example.com");

        // Insert a new entry, which should evict test.com (LRU)
        store.insert(&ca, "new.com").unwrap();

        assert!(store.get("example.com").is_some());
        assert!(store.get("test.com").is_none());
        assert!(store.get("new.com").is_some());
    }

    #[test]
    fn test_store_remove() {
        let ca = CaRoot::generate("Test CA").unwrap();
        let mut store = CertStore::new(10);

        store.insert(&ca, "example.com").unwrap();

        let removed = store.remove("example.com").unwrap();
        assert_eq!(removed.domain(), "example.com");
        assert!(store.get("example.com").is_none());
    }

    #[test]
    fn test_store_remove_not_found() {
        let mut store = CertStore::new(10);
        let result = store.remove("nonexistent.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_store_len() {
        let ca = CaRoot::generate("Test CA").unwrap();
        let mut store = CertStore::new(10);

        assert_eq!(store.len(), 0);
        assert!(store.is_empty());

        store.insert(&ca, "example.com").unwrap();
        assert_eq!(store.len(), 1);
        assert!(!store.is_empty());
    }

    #[test]
    fn test_store_invalid_domain() {
        let ca = CaRoot::generate("Test CA").unwrap();
        let mut store = CertStore::new(10);

        let result = store.insert(&ca, "");
        assert!(result.is_err());
    }

    #[test]
    fn test_store_duplicate_insert() {
        let ca = CaRoot::generate("Test CA").unwrap();
        let mut store = CertStore::new(10);

        store.insert(&ca, "example.com").unwrap();
        store.insert(&ca, "example.com").unwrap();

        assert_eq!(store.len(), 1);
    }
}
