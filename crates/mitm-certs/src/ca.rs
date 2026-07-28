//! CA certificate generation using rcgen with ECDSA P-256.

use rcgen::{CertificateParams, KeyPair};
use std::path::Path;
use thiserror::Error;

/// CA root certificate with ECDSA P-256 key.
#[derive(Debug, Clone)]
pub struct CaRoot {
    /// Certificate in DER format.
    cert_der: Vec<u8>,
    /// Private key in PKCS#8 format.
    key_pkcs8: Vec<u8>,
    /// Common Name from certificate.
    cn: String,
    /// Certificate in PEM format (for reuse in leaf generation).
    cert_pem: String,
}

/// Errors that can occur during CA operations.
#[derive(Error, Debug)]
pub enum CaError {
    #[error("rcgen error: {0}")]
    Rcgen(#[from] rcgen::Error),

    #[error("PEM encoding error: {0}")]
    Pem(#[from] pem::PemError),

    #[error("PKCS#8 error: {0}")]
    Pkcs8(#[from] pkcs8::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("CA not found at {path}")]
    NotFound { path: String },
}

impl CaRoot {
    /// Generate a new CA root certificate with ECDSA P-256.
    ///
    /// # Arguments
    /// * `cn` - Common Name for the CA (default: "mitmproxy-rs CA")
    ///
    /// # Returns
    /// Result with CaRoot or CaError
    pub fn generate(cn: &str) -> Result<Self, CaError> {
        let cn = if cn.is_empty() {
            "mitmproxy-rs CA".to_string()
        } else {
            cn.to_string()
        };

        // Generate ECDSA P-256 key pair
        let key_pair = KeyPair::generate_for(&rcgen::PKCS_ECDSA_P256_SHA256)
            .map_err(CaError::Rcgen)?;

        // Create certificate parameters
        let mut params = CertificateParams::new(vec![])?;
        params.distinguished_name = rcgen::DistinguishedName::new();
        params.distinguished_name.push(
            rcgen::DnType::CommonName,
            rcgen::DnValue::Utf8String(cn.clone()),
        );
        params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        params.key_usages = vec![
            rcgen::KeyUsagePurpose::KeyCertSign,
            rcgen::KeyUsagePurpose::CrlSign,
        ];
        params.not_before = time::OffsetDateTime::now_utc();
        params.not_after = params.not_before + time::Duration::days(365 * 10); // 10 years

        // Generate self-signed certificate
        let cert = params.self_signed(&key_pair)?;
        let key_der = key_pair.serialize_der();
        let cert_pem = cert.pem();

        Ok(Self {
            cert_der: cert.der().to_vec(),
            key_pkcs8: key_der.to_vec(),
            cn,
            cert_pem: cert_pem.to_string(),
        })
    }

    /// Get the certificate in PEM format.
    pub fn cert_pem(&self) -> Result<String, CaError> {
        let pem = pem::encode(&pem::Pem::new("CERTIFICATE", &self.cert_der[..]));
        Ok(pem)
    }

    /// Get the cached certificate PEM string.
    pub fn cert_pem_cached(&self) -> &str {
        &self.cert_pem
    }

    /// Get the private key in PKCS#8 PEM format.
    pub fn private_key_pem(&self) -> Result<String, CaError> {
        let pem = pem::encode(&pem::Pem::new("PRIVATE KEY", &self.key_pkcs8[..]));
        Ok(pem)
    }

    /// Get the certificate in DER format.
    pub fn cert_der(&self) -> &[u8] {
        &self.cert_der
    }

    /// Get the private key in PKCS#8 format.
    pub fn private_key_pkcs8(&self) -> &[u8] {
        &self.key_pkcs8
    }

    /// Get the Common Name.
    pub fn cn(&self) -> &str {
        &self.cn
    }

    /// Save CA to directory (~/.mitmproxy/).
    ///
    /// Creates ca_root.pem and ca_key.pem files.
    pub fn save(&self, dir: &Path) -> Result<(), CaError> {
        std::fs::create_dir_all(dir)?;

        let cert_pem = self.cert_pem()?;
        let key_pem = self.private_key_pem()?;

        let cert_path = dir.join("ca_root.pem");
        let key_path = dir.join("ca_key.pem");

        std::fs::write(&cert_path, cert_pem)?;
        std::fs::write(&key_path, key_pem)?;

        // Set file permissions to 0600 on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&key_path, std::fs::Permissions::from_mode(0o600))?;
        }

        Ok(())
    }

    /// Load CA from directory (~/.mitmproxy/).
    ///
    /// Expects ca_root.pem and ca_key.pem files.
    pub fn load(dir: &Path) -> Result<Self, CaError> {
        let cert_path = dir.join("ca_root.pem");
        let key_path = dir.join("ca_key.pem");

        if !cert_path.exists() || !key_path.exists() {
            return Err(CaError::NotFound {
                path: dir.display().to_string(),
            });
        }

        let cert_pem = std::fs::read_to_string(&cert_path)?;
        let key_pem = std::fs::read_to_string(&key_path)?;

        // Parse PEM
        let cert_pem_obj = pem::parse(&cert_pem).map_err(CaError::Pem)?;
        let key_pem_obj = pem::parse(&key_pem).map_err(CaError::Pem)?;

        if cert_pem_obj.tag() != "CERTIFICATE" {
            return Err(CaError::Pem(pem::PemError::MismatchedTags(
                "CERTIFICATE".to_string(),
                cert_pem_obj.tag().to_string(),
            )));
        }

        if key_pem_obj.tag() != "PRIVATE KEY" {
            return Err(CaError::Pem(pem::PemError::MismatchedTags(
                "PRIVATE KEY".to_string(),
                key_pem_obj.tag().to_string(),
            )));
        }

        Ok(Self {
            cert_der: cert_pem_obj.contents().to_vec(),
            key_pkcs8: key_pem_obj.contents().to_vec(),
            cn: String::new(), // CN will be extracted when needed
            cert_pem,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_ca_generation() {
        let ca = CaRoot::generate("Test CA").unwrap();
        assert_eq!(ca.cn(), "Test CA");
    }

    #[test]
    fn test_ca_default_cn() {
        let ca = CaRoot::generate("").unwrap();
        assert_eq!(ca.cn(), "mitmproxy-rs CA");
    }

    #[test]
    fn test_ca_pem_roundtrip() {
        let ca = CaRoot::generate("Test CA").unwrap();
        let cert_pem = ca.cert_pem().unwrap();
        let key_pem = ca.private_key_pem().unwrap();

        assert!(cert_pem.contains("BEGIN CERTIFICATE"));
        assert!(key_pem.contains("BEGIN PRIVATE KEY"));
    }

    #[test]
    fn test_ca_save_load() {
        let dir = TempDir::new().unwrap();
        let ca = CaRoot::generate("Test CA").unwrap();

        // Save
        ca.save(dir.path()).unwrap();

        // Verify files exist
        assert!(dir.path().join("ca_root.pem").exists());
        assert!(dir.path().join("ca_key.pem").exists());

        // Load
        let loaded = CaRoot::load(dir.path()).unwrap();
        assert_eq!(loaded.cert_der(), ca.cert_der());
        assert_eq!(loaded.private_key_pkcs8(), ca.private_key_pkcs8());
    }

    #[test]
    fn test_ca_load_not_found() {
        let dir = TempDir::new().unwrap();
        let result = CaRoot::load(dir.path());
        assert!(result.is_err());
    }
}
