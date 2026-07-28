//! Leaf certificate generation for TLS interception.

use crate::ca::{CaError, CaRoot};
use rcgen::{CertificateParams, DnType, DnValue, KeyPair};
use x509_parser::prelude::{FromDer, X509Certificate};
use thiserror::Error;

/// Leaf certificate with ECDSA P-256 key signed by CA.
#[derive(Debug)]
pub struct LeafCert {
    /// Certificate in DER format.
    pub cert_der: Vec<u8>,
    /// Private key in PKCS#8 format.
    pub key_pkcs8: Vec<u8>,
    /// Domain name.
    pub domain: String,
}

/// Errors that can occur during leaf certificate operations.
#[derive(Error, Debug)]
pub enum LeafError {
    #[error("rcgen error: {0}")]
    Rcgen(#[from] rcgen::Error),

    #[error("CA error: {0}")]
    Ca(#[from] CaError),

    #[error("Invalid domain: {0}")]
    InvalidDomain(String),
}

impl LeafCert {
    /// Generate a leaf certificate for a domain, signed by the CA.
    ///
    /// # Arguments
    /// * `ca` - CA root certificate
    /// * `domain` - Domain name (e.g., "example.com")
    ///
    /// # Returns
    /// Result with LeafCert or LeafError
    pub fn generate(ca: &CaRoot, domain: &str) -> Result<Self, LeafError> {
        // Validate domain
        if domain.is_empty() {
            return Err(LeafError::InvalidDomain("domain cannot be empty".to_string()));
        }

        // Generate ECDSA P-256 key pair for leaf
        let key_pair = KeyPair::generate_for(&rcgen::PKCS_ECDSA_P256_SHA256)
            .map_err(LeafError::Rcgen)?;

        // Create certificate parameters for leaf
        let mut params = CertificateParams::new(vec![domain.to_string()])?;
        params.distinguished_name = rcgen::DistinguishedName::new();
        params.distinguished_name.push(
            rcgen::DnType::CommonName,
            rcgen::DnValue::Utf8String(domain.to_string()),
        );
        params.is_ca = rcgen::IsCa::NoCa;
        params.not_before = time::OffsetDateTime::now_utc();
        params.not_after = params.not_before + time::Duration::days(365); // 1 year

        // Load CA to sign the leaf certificate
        let ca_key_pem = ca.private_key_pem()?;
        let ca_key_pair = KeyPair::from_pem(&ca_key_pem)?;
        
        // Parse CA certificate using x509-parser to extract issuer info
        let (_, ca_x509) = X509Certificate::from_der(ca.cert_der())
            .map_err(|_| LeafError::Rcgen(rcgen::Error::CouldNotParseCertificate))?;
        
        // Create CA certificate parameters from parsed certificate
        let mut ca_cert_params = CertificateParams::new(vec![])?;
        ca_cert_params.distinguished_name = rcgen::DistinguishedName::new();
        ca_cert_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        ca_cert_params.key_usages = vec![
            rcgen::KeyUsagePurpose::KeyCertSign,
            rcgen::KeyUsagePurpose::CrlSign,
        ];
        
        // Set issuer distinguished name from parsed certificate
        if let Some(cn) = ca_x509.tbs_certificate.subject.iter_common_name().next()
            && let Ok(s) = cn.as_str()
        {
            ca_cert_params.distinguished_name.push(
                DnType::CommonName,
                DnValue::Utf8String(s.to_string()),
            );
        }
        if let Some(o) = ca_x509.tbs_certificate.subject.iter_organization().next()
            && let Ok(s) = o.as_str()
        {
            ca_cert_params.distinguished_name.push(
                DnType::OrganizationName,
                DnValue::Utf8String(s.to_string()),
            );
        }
        if let Some(ou) = ca_x509.tbs_certificate.subject.iter_organizational_unit().next()
            && let Ok(s) = ou.as_str()
        {
            ca_cert_params.distinguished_name.push(
                DnType::OrganizationalUnitName,
                DnValue::Utf8String(s.to_string()),
            );
        }
        
        // Generate a temporary certificate from CA params to use as issuer
        let ca_cert_temp = ca_cert_params.self_signed(&ca_key_pair)?;

        // Sign leaf certificate with CA
        let cert = params.signed_by(&key_pair, &ca_cert_temp, &ca_key_pair)?;
        let key_pem = key_pair.serialize_pem();

        Ok(Self {
            cert_der: cert.der().to_vec(),
            key_pkcs8: key_pem.as_bytes().to_vec(),
            domain: domain.to_string(),
        })
    }

    /// Get the certificate in PEM format.
    pub fn cert_pem(&self) -> Result<String, LeafError> {
        let pem = pem::encode(&pem::Pem::new("CERTIFICATE", &self.cert_der[..]));
        Ok(pem)
    }

    /// Get the private key in PKCS#8 PEM format.
    pub fn key_pem(&self) -> Result<String, LeafError> {
        let pem = pem::encode(&pem::Pem::new("PRIVATE KEY", &self.key_pkcs8[..]));
        Ok(pem)
    }

    /// Get the domain name.
    pub fn domain(&self) -> &str {
        &self.domain
    }

    /// Get the certificate in DER format.
    pub fn cert_der(&self) -> &[u8] {
        &self.cert_der
    }

    /// Get the private key in PKCS#8 format.
    pub fn key_pkcs8(&self) -> &[u8] {
        &self.key_pkcs8
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ca::CaRoot;

    #[test]
    fn test_leaf_generation() {
        let ca = CaRoot::generate("Test CA").unwrap();
        let leaf = LeafCert::generate(&ca, "example.com").unwrap();
        assert_eq!(leaf.domain(), "example.com");
        drop(leaf);
    }

    #[test]
    fn test_leaf_pem_roundtrip() {
        let ca = CaRoot::generate("Test CA").unwrap();
        let leaf = LeafCert::generate(&ca, "example.com").unwrap();
        let cert_pem = leaf.cert_pem().unwrap();
        let key_pem = leaf.key_pem().unwrap();

        assert!(cert_pem.contains("BEGIN CERTIFICATE"));
        assert!(key_pem.contains("BEGIN PRIVATE KEY"));
        drop(leaf);
    }

    #[test]
    fn test_leaf_invalid_domain() {
        let ca = CaRoot::generate("Test CA").unwrap();
        let result = LeafCert::generate(&ca, "");
        assert!(result.is_err());
    }
}
