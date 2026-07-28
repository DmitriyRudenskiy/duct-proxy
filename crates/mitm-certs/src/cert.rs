//! X.509 Certificate wrapper type.
//!
//! This module provides a high-level `Cert` wrapper around `x509_parser` that
//! can parse DER-encoded certificates and export them as PEM.

use thiserror::Error;
use sha2::{Digest, Sha256};

/// Errors that can occur when working with certificates.
#[derive(Error, Debug)]
pub enum CertError {
    /// Failed to parse the certificate DER bytes.
    #[error("failed to parse certificate: {0}")]
    ParseError(String),

    /// Failed to encode certificate to PEM format.
    #[error("failed to encode to PEM: {0}")]
    PemError(String),
}

/// X.509 Certificate wrapper.
///
/// Wraps DER-encoded certificate data with lazy parsing for metadata.
#[derive(Debug, Clone)]
pub struct Cert {
    /// Raw DER bytes of the certificate.
    der_bytes: Vec<u8>,
    /// Parsed subject string.
    subject: Option<String>,
    /// Parsed issuer string.
    issuer: Option<String>,
    /// Parsed CN from subject.
    cn: Option<String>,
    /// SHA-256 fingerprint (hex, 64 chars).
    fingerprint: Option<String>,
    /// Whether this is a CA certificate.
    is_ca_flag: Option<bool>,
}

impl Cert {
    /// Parse a certificate from DER-encoded bytes.
    ///
    /// # Arguments
    /// * `bytes` - DER-encoded X.509 certificate
    ///
    /// # Errors
    /// Returns `CertError::ParseError` if the bytes are not valid DER.
    pub fn from_der(bytes: &[u8]) -> Result<Self, CertError> {
        let (_rest, x509) = x509_parser::parse_x509_certificate(bytes)
            .map_err(|e| CertError::ParseError(format!("{:?}", e)))?;

        if !_rest.is_empty() {
            return Err(CertError::ParseError(
                "trailing data after certificate".to_string(),
            ));
        }

        let subject = x509.subject().to_string();
        let issuer = x509.issuer().to_string();

        // Extract CN from subject
        let cn = x509
            .subject()
            .iter_common_name()
            .next()
            .and_then(|attr| {
                let value = attr.attr_value().data;
                std::str::from_utf8(value).ok()
            })
            .map(String::from);

        // Calculate SHA-256 fingerprint
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let result = hasher.finalize();
        let fingerprint = hex::encode(result);

        // Check if CA using basic_constraints
        let is_ca = x509
            .basic_constraints()
            .ok()
            .flatten()
            .map(|bc| bc.value.ca)
            .unwrap_or(false);

        Ok(Self {
            der_bytes: bytes.to_vec(),
            subject: Some(subject),
            issuer: Some(issuer),
            cn,
            fingerprint: Some(fingerprint),
            is_ca_flag: Some(is_ca),
        })
    }

    /// Export the certificate as PEM-encoded string.
    ///
    /// # Errors
    /// Returns `CertError::PemError` if encoding fails.
    pub fn to_pem(&self) -> Result<String, CertError> {
        let pem_tag = "CERTIFICATE";
        let pem = pem::Pem::new(pem_tag, self.der_bytes.as_slice());
        Ok(pem::encode(&pem))
    }

    /// Get the SHA-256 fingerprint of the certificate.
    ///
    /// Returns a hex-encoded string (lowercase, 64 characters).
    pub fn fingerprint_sha256(&self) -> &str {
        self.fingerprint
            .as_deref()
            .expect("fingerprint should be set after from_der")
    }

    /// Get the subject distinguished name as a string.
    pub fn subject(&self) -> &str {
        self.subject.as_deref().unwrap_or("")
    }

    /// Get the issuer distinguished name as a string.
    pub fn issuer(&self) -> &str {
        self.issuer.as_deref().unwrap_or("")
    }

    /// Get the Common Name from the subject.
    pub fn common_name(&self) -> Option<&str> {
        self.cn.as_deref()
    }

    /// Check if this is a CA certificate (has the CA basic constraint).
    pub fn is_ca(&self) -> bool {
        self.is_ca_flag.unwrap_or(false)
    }

    /// Get the validity period as (not_before, not_after) as Unix timestamps.
    pub fn validity(&self) -> Option<(i64, i64)> {
        let (_, x509) = x509_parser::parse_x509_certificate(&self.der_bytes).ok()?;
        let not_before = x509.validity().not_before;
        let not_after = x509.validity().not_after;

        // Convert ASN1Time to unix timestamp (timestamp() returns i64)
        Some((not_before.timestamp(), not_after.timestamp()))
    }

    /// Get the raw DER bytes.
    pub fn der_bytes(&self) -> &[u8] {
        &self.der_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ca::CaRoot;

    #[test]
    fn test_cert_from_der_roundtrip() {
        // Generate a CA certificate
        let ca = CaRoot::generate("Test CA").unwrap();
        let ca_pem = ca.cert_pem().unwrap();

        // Parse the PEM back to DER
        let pem = pem::parse(&ca_pem).unwrap();
        let cert = Cert::from_der(pem.contents()).unwrap();

        // Export to PEM and verify it matches
        let exported_pem = cert.to_pem().unwrap();
        assert_eq!(ca_pem, exported_pem);
    }

    #[test]
    fn test_cert_fingerprint_sha256() {
        let ca = CaRoot::generate("Test CA 2").unwrap();
        let pem = pem::parse(ca.cert_pem().unwrap()).unwrap();
        let cert = Cert::from_der(pem.contents()).unwrap();

        let fp = cert.fingerprint_sha256();

        // Verify fingerprint format (hex string, 64 chars for SHA-256)
        assert!(!fp.is_empty());
        assert_eq!(fp.len(), 64);

        // Same cert should produce same fingerprint
        let fp2 = cert.fingerprint_sha256();
        assert_eq!(fp, fp2);
    }

    #[test]
    fn test_cert_subject_issuer() {
        let ca = CaRoot::generate("My Test CA").unwrap();
        let pem = pem::parse(ca.cert_pem().unwrap()).unwrap();
        let cert = Cert::from_der(pem.contents()).unwrap();

        assert_eq!(cert.subject(), "CN=My Test CA");
        assert_eq!(cert.issuer(), "CN=My Test CA");
    }

    #[test]
    fn test_cert_common_name() {
        let ca = CaRoot::generate("Example CA").unwrap();
        let pem = pem::parse(ca.cert_pem().unwrap()).unwrap();
        let cert = Cert::from_der(pem.contents()).unwrap();

        assert_eq!(cert.common_name(), Some("Example CA"));
    }

    #[test]
    fn test_cert_is_ca() {
        let ca = CaRoot::generate("Test CA").unwrap();
        let pem = pem::parse(ca.cert_pem().unwrap()).unwrap();
        let cert = Cert::from_der(pem.contents()).unwrap();

        assert!(cert.is_ca(), "CA certificate should have CA=TRUE");
    }

    #[test]
    fn test_cert_validity() {
        let ca = CaRoot::generate("Test CA").unwrap();
        let pem = pem::parse(ca.cert_pem().unwrap()).unwrap();
        let cert = Cert::from_der(pem.contents()).unwrap();

        let validity = cert.validity().expect("validity should be parseable");
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Validity should span ~10 years (315360000 seconds)
        assert!(validity.0 <= now);
        assert!(validity.1 > now);
        assert!(validity.1 - validity.0 > 300000000);
    }

    #[test]
    fn test_cert_from_invalid_der() {
        let invalid_der = vec![0x00, 0x01, 0x02];
        assert!(Cert::from_der(&invalid_der).is_err());
    }
}
