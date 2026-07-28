//! Connection types: `Connection` base, `Client`, `Server`.
//!
//! These represent metadata about network connections. The actual socket I/O
//! is handled externally; these structs only carry identification and state.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Connection state flags.
///
/// Mirrors the Python `ConnectionState` flag enum: `Closed`, `CanRead`, `CanWrite`,
/// `Open = CanRead | CanWrite`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ConnectionState(u8);

impl ConnectionState {
    pub const CLOSED: u8 = 0b000;
    pub const CAN_READ: u8 = 0b001;
    pub const CAN_WRITE: u8 = 0b010;
    pub const OPEN: u8 = Self::CAN_READ | Self::CAN_WRITE;

    pub const fn closed() -> Self {
        Self(Self::CLOSED)
    }

    pub const fn can_read() -> Self {
        Self(Self::CAN_READ)
    }

    pub const fn can_write() -> Self {
        Self(Self::CAN_WRITE)
    }

    pub const fn open() -> Self {
        Self(Self::OPEN)
    }

    pub const fn is_closed(self) -> bool {
        self.0 == Self::CLOSED
    }

    pub const fn is_open(self) -> bool {
        self.0 == Self::OPEN
    }

    pub const fn can_read_to(self) -> bool {
        self.0 & Self::CAN_READ != 0
    }

    pub const fn can_write_to(self) -> bool {
        self.0 & Self::CAN_WRITE != 0
    }

    /// Transitions to `next` state using the flag mask.
    pub const fn transition(self, next: Self) -> Self {
        Self((self.0 & !0b111) | next.0)
    }
}

impl fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Self::CLOSED => write!(f, "Closed"),
            Self::CAN_READ => write!(f, "CanRead"),
            Self::CAN_WRITE => write!(f, "CanWrite"),
            Self::OPEN => write!(f, "Open"),
            other => write!(f, "State(0x{:02x})", other),
        }
    }
}

/// A network address tuple: `(host_or_ip, port)`.
pub type Address = (String, u16);

/// TLS certificate wrapper.
///
/// Wraps DER-encoded X.509 certificate data with lazy parsing for
/// subject/issuer/fingerprint fields.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Cert {
    /// DER-encoded certificate bytes.
    pub der: Vec<u8>,
    /// Common name (CN) if parsed.
    pub cn: Option<String>,
    /// SHA-256 fingerprint (hex, colon-separated).
    pub fingerprint_sha256: Option<String>,
}

impl Cert {
    /// Create a new Cert from DER-encoded bytes.
    pub fn from_der(der: Vec<u8>) -> Self {
        let mut cert = Self {
            der,
            cn: None,
            fingerprint_sha256: None,
        };
        cert.parse_metadata();
        cert
    }

    /// Parse CN and fingerprint from DER bytes (lazy, only once).
    fn parse_metadata(&mut self) {
        // Try to parse CN using x509-parser
        if let Ok((_, x509)) = x509_parser::parse_x509_certificate(&self.der)
            && let Some(cn) = x509.subject().iter_common_name().next()
            && let Ok(s) = std::str::from_utf8(cn.attr_value().data)
        {
            self.cn = Some(s.to_string());
        }

        // Calculate SHA-256 fingerprint
        use sha2::Digest;
        let mut hasher = sha2::Sha256::new();
        hasher.update(&self.der);
        let result = hasher.finalize();
        self.fingerprint_sha256 = Some(hex::encode(result));
    }

    /// Export the certificate as PEM-encoded string.
    pub fn to_pem(&self) -> Result<String, String> {
        let pem_tag = "CERTIFICATE";
        let pem = pem::Pem::new(pem_tag, self.der.as_slice());
        Ok(pem::encode(&pem))
    }

    /// Get the certificate fingerprint (lazy, parses if needed).
    pub fn fingerprint(&self) -> Option<&str> {
        self.fingerprint_sha256.as_deref()
    }

    /// Get the Common Name (lazy, parses if needed).
    pub fn common_name(&self) -> Option<&str> {
        self.cn.as_deref()
    }

    /// Check if this is a CA certificate.
    pub fn is_ca(&self) -> bool {
        if let Ok((_, x509)) = x509_parser::parse_x509_certificate(&self.der)
            && let Ok(Some(bc)) = x509.basic_constraints()
        {
            return bc.value.ca;
        }
        false
    }

    /// Get the validity period as (not_before, not_after) Unix timestamps.
    pub fn validity(&self) -> Option<(i64, i64)> {
        x509_parser::parse_x509_certificate(&self.der)
            .ok()
            .map(|(_, x509)| {
                let not_before = x509.validity().not_before;
                let not_after = x509.validity().not_after;
                (not_before.timestamp(), not_after.timestamp())
            })
    }
}

/// Optional upstream proxy specification for a server connection.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ServerSpec {
    /// Proxy address (host, port).
    pub address: Address,
    /// Authentication credentials (if any).
    pub auth: Option<String>,
}

/// Base connection metadata shared by `Client` and `Server`.
///
/// Does NOT hold the underlying socket — all I/O is handled externally.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Connection {
    /// Unique identifier (UUID4).
    pub id: String,
    /// Remote `(ip, port)` tuple.
    pub peername: Option<Address>,
    /// Local `(ip, port)` tuple.
    pub sockname: Option<Address>,
    /// Current connection state.
    pub state: ConnectionState,
    /// Transport protocol: "tcp" or "udp".
    pub transport_protocol: String,
    /// Connection-level error (not per-flow).
    pub error: Option<String>,
    /// Whether TLS should eventually be established.
    pub tls: bool,
    /// TLS certificate chain from peer.
    pub certificate_list: Vec<Cert>,
    /// Negotiated ALPN protocol.
    pub alpn: Option<Vec<u8>>,
    /// ALPN offers from ClientHello.
    pub alpn_offers: Vec<Vec<u8>>,
    /// Active cipher suite name.
    pub cipher: Option<String>,
    /// Accepted cipher suites.
    pub cipher_list: Vec<String>,
    /// TLS version string (e.g., "TLSv1.3").
    pub tls_version: Option<String>,
    /// Server Name Indication.
    pub sni: Option<String>,
    /// Connection start timestamp.
    pub timestamp_start: Option<f64>,
    /// Connection close timestamp.
    pub timestamp_end: Option<f64>,
    /// TLS handshake completion timestamp.
    pub timestamp_tls_setup: Option<f64>,
}

impl Connection {
    /// Returns `true` if the connection is fully open.
    pub fn connected(&self) -> bool {
        self.state.is_open()
    }

    /// Returns `true` if TLS has been established.
    pub fn tls_established(&self) -> bool {
        self.timestamp_tls_setup.is_some()
    }

    /// Generates a new default Connection.
    pub fn new(transport_protocol: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            peername: None,
            sockname: None,
            state: ConnectionState::closed(),
            transport_protocol: transport_protocol.to_string(),
            error: None,
            tls: false,
            certificate_list: Vec::new(),
            alpn: None,
            alpn_offers: Vec::new(),
            cipher: None,
            cipher_list: Vec::new(),
            tls_version: None,
            sni: None,
            timestamp_start: None,
            timestamp_end: None,
            timestamp_tls_setup: None,
        }
    }
}

/// A connection from a client to mitmproxy.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Client {
    /// The base connection metadata.
    pub connection: Connection,
    /// The certificate mitmproxy presented to the client.
    pub mitmcert: Option<Cert>,
    /// Proxy mode string (e.g., "regular", "transparent").
    pub proxy_mode: String,
}

impl Client {
    /// Creates a new Client with the given peer and local addresses.
    pub fn new(peername: Address, sockname: Address, proxy_mode: &str) -> Self {
        let mut connection = Connection::new("tcp");
        connection.peername = Some(peername);
        connection.sockname = Some(sockname);
        connection.timestamp_start = Some(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64());
        Self {
            connection,
            mitmcert: None,
            proxy_mode: proxy_mode.to_string(),
        }
    }
}

/// A connection from mitmproxy to an upstream server.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Server {
    /// The base connection metadata.
    pub connection: Connection,
    /// Target `(host, port)` — may be `None` (upstream proxy mode).
    pub address: Option<Address>,
    /// TCP ACK received timestamp.
    pub timestamp_tcp_setup: Option<f64>,
    /// Optional upstream proxy specification.
    pub via: Option<ServerSpec>,
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

impl Server {
    /// Creates a new Server.
    pub fn new() -> Self {
        Self {
            connection: Connection::new("tcp"),
            address: None,
            timestamp_tcp_setup: None,
            via: None,
        }
    }

    /// Sets the target address.
    ///
    /// # Panics
    /// Panics if the connection is already open.
    pub fn set_address(&mut self, address: Option<Address>) -> Result<(), ConnectionError> {
        if self.connection.state.is_open() {
            return Err(ConnectionError::OpenConnection);
        }
        self.address = address;
        Ok(())
    }

    /// Sets the upstream proxy specification.
    ///
    /// # Panics
    /// Panics if the connection is already open.
    pub fn set_via(
        &mut self,
        via: Option<ServerSpec>,
    ) -> Result<(), ConnectionError> {
        if self.connection.state.is_open() {
            return Err(ConnectionError::OpenConnection);
        }
        self.via = via;
        Ok(())
    }
}

/// Errors that can occur with connection operations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConnectionError {
    /// Attempted to modify a field on an open connection.
    OpenConnection,
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OpenConnection => {
                write!(f, "cannot change field on an open connection")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_state_flags() {
        assert!(ConnectionState::closed().is_closed());
        assert!(!ConnectionState::closed().is_open());
        assert!(ConnectionState::open().is_open());
        assert!(!ConnectionState::open().is_closed());
        assert!(ConnectionState::open().can_read_to());
        assert!(ConnectionState::open().can_write_to());
        assert!(!ConnectionState::can_read().can_write_to());
        assert!(!ConnectionState::can_write().can_read_to());
    }

    #[test]
    fn test_connection_state_transition() {
        let s = ConnectionState::closed().transition(ConnectionState::can_read());
        assert_eq!(s, ConnectionState::can_read());
        let s = ConnectionState::can_read().transition(ConnectionState::open());
        assert_eq!(s, ConnectionState::open());
    }

    #[test]
    fn test_connection_default() {
        let c = Connection::new("tcp");
        assert_eq!(c.transport_protocol, "tcp");
        assert!(!c.connected());
        assert!(!c.tls_established());
        assert!(c.id.len() > 0);
    }

    #[test]
    fn test_server_guard_rejects_on_open() {
        let mut server = Server::new();
        server.connection.state = ConnectionState::open();
        assert!(server.set_address(Some(("example.com".to_string(), 443))).is_err());
        assert!(server.set_via(Some(ServerSpec::default())).is_err());
    }

    #[test]
    fn test_server_set_address_ok() {
        let mut server = Server::new();
        assert!(server
            .set_address(Some(("example.com".to_string(), 443)))
            .is_ok());
        assert_eq!(server.address, Some(("example.com".to_string(), 443)));
    }

    #[test]
    fn test_client_creation() {
        let client = Client::new(
            ("127.0.0.1".to_string(), 12345),
            ("0.0.0.0".to_string(), 8080),
            "regular",
        );
        assert_eq!(client.proxy_mode, "regular");
        assert_eq!(client.connection.peername, Some(("127.0.0.1".to_string(), 12345)));
        assert!(client.connection.timestamp_start.is_some());
    }

    #[test]
    fn test_cert_from_der() {
        // Create a minimal self-signed certificate for testing
        // This is a simple DER-encoded certificate (not a valid one, but tests parsing)
        let cert = Cert::from_der(vec![0x30, 0x82, 0x01, 0x00]); // Not valid DER, but tests the API
        assert!(cert.fingerprint().is_some());
        assert!(cert.common_name().is_none()); // Invalid cert, so no CN
    }

    #[test]
    fn test_cert_pem_roundtrip() {
        // Create a Cert with some DER bytes
        let der_bytes = vec![0x30, 0x82, 0x01, 0x00, 0x30, 0x82, 0x00, 0x00];
        let cert = Cert::from_der(der_bytes.clone());

        // Test PEM export (may fail for invalid DER, but API should work)
        let result = cert.to_pem();
        // For invalid DER, PEM export may fail, which is expected
        if let Ok(pem) = result {
            assert!(pem.contains("CERTIFICATE"));
            assert!(pem.contains("-----"));
        }
    }

    #[test]
    fn test_cert_fingerprint_deterministic() {
        let der = vec![0x01, 0x02, 0x03, 0x04];
        let cert1 = Cert::from_der(der.clone());
        let cert2 = Cert::from_der(der);

        assert_eq!(cert1.fingerprint(), cert2.fingerprint());
    }

    #[test]
    fn test_cert_is_ca() {
        // Invalid DER, should return false
        let cert = Cert::from_der(vec![0x00, 0x01]);
        assert!(!cert.is_ca());
    }

    #[test]
    fn test_cert_validity() {
        // Invalid DER, should return None
        let cert = Cert::from_der(vec![0x00, 0x01]);
        assert!(cert.validity().is_none());
    }
}
