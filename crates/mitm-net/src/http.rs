//! HTTP message types: `StreamMode`, `Message`, `MessageData`, `Request`, `Response`.
//!
//! These types represent individual HTTP messages (requests and responses)
//! with content management (raw/content/text/json), streaming control,
//! and URL/form parsing.

use mitm_core::headers::Headers;
use serde::{Deserialize, Serialize};

/// Transform function type for streaming.
type TransformFn = Box<dyn FnMut(&[u8]) -> Vec<u8> + Send>;

/// Controls how a message body is streamed.
#[derive(Default)]
pub enum StreamMode {
    /// Buffer the entire body before forwarding (enables body transformation).
    #[default]
    Buffered,
    /// Forward body immediately without buffering.
    Passthrough,
    /// Apply a transformation function to each chunk.
    Transform(TransformFn),
}

impl Clone for StreamMode {
    fn clone(&self) -> Self {
        match self {
            Self::Buffered => Self::Buffered,
            Self::Passthrough => Self::Passthrough,
            Self::Transform(_) => Self::Buffered,
        }
    }
}

impl std::fmt::Debug for StreamMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Buffered => write!(f, "Buffered"),
            Self::Passthrough => write!(f, "Passthrough"),
            Self::Transform(_) => write!(f, "Transform(...)"),
        }
    }
}

impl Serialize for StreamMode {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Buffered => serializer.serialize_str("buffered"),
            Self::Passthrough => serializer.serialize_str("passthrough"),
            Self::Transform(_) => serializer.serialize_str("transform"),
        }
    }
}

impl<'de> Deserialize<'de> for StreamMode {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "buffered" => Ok(Self::Buffered),
            "passthrough" => Ok(Self::Passthrough),
            "transform" => Ok(Self::Transform(Box::new(|data: &[u8]| data.to_vec()))),
            _ => Err(serde::de::Error::custom(format!("unknown stream mode: {}", s))),
        }
    }
}

/// Underlying data for `Message`, `Request`, and `Response`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageData {
    /// HTTP version (e.g., `b"HTTP/1.1"`).
    pub http_version: Vec<u8>,
    /// HTTP headers.
    pub headers: Headers,
    /// Raw (potentially compressed) body bytes.
    pub content: Option<Vec<u8>>,
    /// HTTP trailers.
    pub trailers: Option<Headers>,
    /// When headers were received.
    pub timestamp_start: f64,
    /// When last byte was received.
    pub timestamp_end: Option<f64>,
}

/// Base HTTP message (parent of `Request` and `Response`).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    /// Underlying data.
    pub data: MessageData,
    /// Streaming mode control.
    pub stream: StreamMode,
}

impl Message {
    /// Create a new message with default values.
    pub fn new() -> Self {
        Self {
            data: MessageData {
                http_version: b"HTTP/1.1".to_vec(),
                headers: Headers::new(),
                content: None,
                trailers: None,
                timestamp_start: mitm_core::flow::current_timestamp(),
                timestamp_end: None,
            },
            stream: StreamMode::default(),
        }
    }

    // ---- Properties ----

    /// HTTP version string (e.g., "HTTP/1.1").
    pub fn http_version(&self) -> String {
        String::from_utf8_lossy(&self.data.http_version).into_owned()
    }

    /// Set HTTP version from string.
    pub fn set_http_version(&mut self, version: &str) {
        self.data.http_version = version.as_bytes().to_vec();
    }

    /// Returns `true` if HTTP/1.0.
    pub fn is_http10(&self) -> bool {
        self.data.http_version == b"HTTP/1.0"
    }

    /// Returns `true` if HTTP/1.1.
    pub fn is_http11(&self) -> bool {
        self.data.http_version == b"HTTP/1.1"
    }

    /// Headers reference.
    pub fn headers(&self) -> &Headers {
        &self.data.headers
    }

    /// Mutable headers reference.
    pub fn headers_mut(&mut self) -> &mut Headers {
        &mut self.data.headers
    }

    /// Trailers reference (may be None).
    pub fn trailers(&self) -> Option<&Headers> {
        self.data.trailers.as_ref()
    }

    /// Set trailers.
    pub fn set_trailers(&mut self, trailers: Option<Headers>) {
        self.data.trailers = trailers;
    }

    /// Timestamp when headers were received.
    pub fn timestamp_start(&self) -> f64 {
        self.data.timestamp_start
    }

    /// Set timestamp when headers were received.
    pub fn set_timestamp_start(&mut self, ts: f64) {
        self.data.timestamp_start = ts;
    }

    /// Timestamp when last byte was received.
    pub fn timestamp_end(&self) -> Option<f64> {
        self.data.timestamp_end
    }

    /// Set timestamp when last byte was received.
    pub fn set_timestamp_end(&mut self, ts: Option<f64>) {
        self.data.timestamp_end = ts;
    }

    // ---- Content management ----

    /// Raw (potentially compressed) body bytes.
    pub fn raw_content(&self) -> Option<&[u8]> {
        self.data.content.as_deref()
    }

    /// Set raw content bytes.
    pub fn set_raw_content(&mut self, content: Option<Vec<u8>>) {
        self.data.content = content;
        self.update_content_length();
    }

    /// Uncompressed body bytes.
    ///
    /// For simplicity in v1, this returns raw_content directly
    /// (no decompression — that belongs to a future parsing layer).
    pub fn content(&self) -> Option<&[u8]> {
        self.data.content.as_deref()
    }

    /// Set uncompressed body. Auto-updates content-length header.
    pub fn set_content(&mut self, value: Option<Vec<u8>>) {
        self.data.content = value;
        self.update_content_length();
    }

    /// Body as text string.
    pub fn text(&self) -> Option<String> {
        self.data.content.as_ref().map(|c| String::from_utf8_lossy(c).into_owned())
    }

    /// Set body as text. Encodes to UTF-8 bytes.
    pub fn set_text(&mut self, value: Option<String>) {
        self.data.content = value.map(|s| s.into_bytes());
        self.update_content_length();
    }

    /// Parse body as JSON.
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, std::io::Error> {
        let content = self
            .data
            .content
            .as_ref()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "no content"))?;
        serde_json::from_slice(content).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
    }

    /// Update the Content-Length header based on current body.
    fn update_content_length(&mut self) {
        if self.data.content.is_some() && !self.data.headers.contains("transfer-encoding") {
            let len = self
                .data
                .content
                .as_ref()
                .map(|c| c.len())
                .unwrap_or(0);
            self.data.headers.set("content-length", &len.to_string());
        }
    }

    /// Decode body based on current Content-Encoding header, then remove it.
    ///
    /// In v1 this is a no-op (no decompression implemented).
    pub fn decode(&mut self) {
        self.data.headers.delete("content-encoding").ok();
    }

    /// Encode body with the given encoding (gzip, deflate, identity, br, zstd).
    ///
    /// Sets the Content-Encoding header. Full compression implementation is deferred.
    pub fn encode(&mut self, encoding: &str) {
        self.headers_mut().insert(0, "content-encoding", encoding);
    }
}

impl Default for Message {
    fn default() -> Self {
        Self::new()
    }
}

// ---- Request ----

/// An HTTP request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Request {
    /// Base message data.
    pub data: MessageData,
    /// Streaming mode.
    pub stream: StreamMode,
    /// Target host.
    pub host: String,
    /// Target port.
    pub port: u16,
    /// HTTP method (e.g., b"GET").
    pub method: Vec<u8>,
    /// URL scheme (e.g., b"http", b"https").
    pub scheme: Vec<u8>,
    /// HTTP/2 :authority or absolute-form target.
    pub authority: Vec<u8>,
    /// Request path including query.
    pub path: Vec<u8>,
}

impl Request {
    /// Create a new Request with default values.
    pub fn new() -> Self {
        Self {
            data: MessageData {
                http_version: b"HTTP/1.1".to_vec(),
                headers: Headers::new(),
                content: None,
                trailers: None,
                timestamp_start: mitm_core::flow::current_timestamp(),
                timestamp_end: None,
            },
            stream: StreamMode::default(),
            host: String::new(),
            port: 80,
            method: b"GET".to_vec(),
            scheme: b"http".to_vec(),
            authority: Vec::new(),
            path: b"/".to_vec(),
        }
    }

    /// Simplified factory: create a request from method + URL.
    pub fn make(
        method: &str,
        url: &str,
        content: Option<&[u8]>,
        headers: &[(String, String)],
    ) -> Self {
        let mut req = Self::new();
        req.method = method.as_bytes().to_vec();
        req.url_set(url);
        if let Some(body) = content {
            req.data.content = Some(body.to_vec());
            req.data.headers.set("content-length", &body.len().to_string());
        }
        for (k, v) in headers {
            req.data.headers.set(k, v);
        }
        req
    }

    // ---- Properties ----

    /// HTTP method as uppercase string (e.g., "GET").
    pub fn method(&self) -> String {
        String::from_utf8_lossy(&self.method).into_owned().to_uppercase()
    }

    /// Set HTTP method.
    pub fn set_method(&mut self, method: &str) {
        self.method = method.as_bytes().to_vec();
    }

    /// URL scheme as string.
    pub fn scheme(&self) -> String {
        String::from_utf8_lossy(&self.scheme).into_owned()
    }

    /// Set URL scheme.
    pub fn set_scheme(&mut self, scheme: &str) {
        self.scheme = scheme.as_bytes().to_vec();
    }

    /// Authority as string.
    pub fn authority(&self) -> String {
        String::from_utf8_lossy(&self.authority).into_owned()
    }

    /// Set authority.
    pub fn set_authority(&mut self, authority: &str) {
        self.authority = authority.as_bytes().to_vec();
    }

    /// Target host.
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Set target host (also updates Host header and authority).
    pub fn set_host(&mut self, host: &str) {
        self.host = host.to_string();
        if self.data.headers.contains("Host") || self.is_http2() {
            self.data.headers.set("Host", &self.hostport());
        }
        if !self.authority.is_empty() {
            self.authority = self.hostport().into_bytes();
        }
    }

    /// Target port.
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Set target port.
    pub fn set_port(&mut self, port: u16) {
        self.port = port;
        if self.data.headers.contains("Host") || self.is_http2() {
            self.data.headers.set("Host", &self.hostport());
        }
        if !self.authority.is_empty() {
            self.authority = self.hostport().into_bytes();
        }
    }

    /// Request path as string.
    pub fn path(&self) -> String {
        String::from_utf8_lossy(&self.path).into_owned()
    }

    /// Set request path.
    pub fn set_path(&mut self, path: &str) {
        self.path = path.as_bytes().to_vec();
    }

    /// Host:port string.
    pub fn hostport(&self) -> String {
        if self.port == default_port(&self.scheme()) {
            self.host.clone()
        } else {
            format!("{}:{}", self.host, self.port)
        }
    }

    /// Full URL string (computed from scheme+host+port+path).
    pub fn url(&self) -> String {
        if self.first_line_format() == "authority" {
            return self.authority();
        }
        let path = if self.path == b"*" {
            ""
        } else {
            &self.path()
        };
        format!("{}://{}{}", self.scheme(), self.hostport(), path)
    }

    /// Set URL (parses and updates scheme, host, port, path).
    pub fn url_set(&mut self, url: &str) {
        // Simple URL parser for common cases.
        let (scheme, rest) = match url.split_once("://") {
            Some((s, r)) => (s.to_string(), r),
            None => ("http".to_string(), url),
        };
        let scheme_clone = scheme.clone();
        self.scheme = scheme.into_bytes();

        let (host_port, path) = match rest.find('/') {
            Some(idx) => (&rest[..idx], &rest[idx..]),
            None => (rest, "/"),
        };

        let (host, port) = match host_port.rfind(':') {
            Some(idx) if idx > 0 => {
                let h = &host_port[..idx];
                let p = host_port[idx + 1..]
                    .parse::<u16>()
                    .unwrap_or(default_port(&scheme_clone));
                (h.to_string(), p)
            }
            _ => (host_port.to_string(), default_port(&scheme_clone)),
        };

        self.host = host;
        self.port = port;
        self.path = path.as_bytes().to_vec();
    }

    /// HTTP/1 Host header or HTTP/2 :authority.
    pub fn host_header(&self) -> Option<String> {
        if (self.is_http2() || self.is_http3()) && !self.authority.is_empty() {
            return Some(self.authority());
        }
        self.data.headers.get("Host").ok()
    }

    /// Host derived from Host header (preferred for display).
    pub fn pretty_host(&self) -> String {
        self.host_header()
            .map(|h| {
                if let Some(idx) = h.rfind(':') {
                    h[..idx].to_string()
                } else {
                    h
                }
            })
            .unwrap_or_else(|| self.host.clone())
    }

    /// Full URL using pretty_host.
    pub fn pretty_url(&self) -> String {
        if self.first_line_format() == "authority" {
            return self.authority();
        }
        let host = self.pretty_host();
        let port = self.port;
        let path = if self.path == b"*" {
            ""
        } else {
            &self.path()
        };
        format!("{}://{}{}", self.scheme(), format_host_port(&host, port), path)
    }

    /// URL path components as strings (e.g., ["api", "v1", "users"]).
    pub fn path_components(&self) -> Vec<String> {
        let path = self.path();
        path.split('/')
            .filter(|s| !s.is_empty())
            .map(url_decode)
            .collect()
    }

    // ---- Content management (delegated) ----

    /// Raw body bytes.
    pub fn raw_content(&self) -> Option<&[u8]> {
        self.data.content.as_deref()
    }

    /// Set raw content bytes.
    pub fn set_raw_content(&mut self, content: Option<Vec<u8>>) {
        self.data.content = content;
        self.update_content_length();
    }

    /// Uncompressed body bytes.
    pub fn content(&self) -> Option<&[u8]> {
        self.data.content.as_deref()
    }

    /// Set uncompressed body. Auto-updates content-length header.
    pub fn set_content(&mut self, value: Option<Vec<u8>>) {
        self.data.content = value;
        self.update_content_length();
    }

    /// Body as text string.
    pub fn text(&self) -> Option<String> {
        self.data.content.as_ref().map(|c| String::from_utf8_lossy(c).into_owned())
    }

    /// Set body as text. Encodes to UTF-8 bytes.
    pub fn set_text(&mut self, value: Option<String>) {
        self.data.content = value.map(|s| s.into_bytes());
        self.update_content_length();
    }

    /// Headers reference.
    pub fn headers(&self) -> &Headers {
        &self.data.headers
    }

    /// Headers mutable reference.
    pub fn headers_mut(&mut self) -> &mut Headers {
        &mut self.data.headers
    }

    /// Parse body as JSON.
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, std::io::Error> {
        let content = self
            .data
            .content
            .as_ref()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "no content"))?;
        serde_json::from_slice(content).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
    }

    /// Update Content-Length header based on current body.
    fn update_content_length(&mut self) {
        if self.data.content.is_some() && !self.data.headers.contains("transfer-encoding") {
            let len = self
                .data
                .content
                .as_ref()
                .map(|c| c.len())
                .unwrap_or(0);
            self.data.headers.set("content-length", &len.to_string());
        }
    }

    /// Query parameters as a list of (key, value) pairs parsed from the path query string.
    pub fn query_pairs(&self) -> Vec<(String, String)> {
        let path_str = self.path();
        if let Some(idx) = path_str.find('?') {
            decode_query(&path_str[idx + 1..])
        } else {
            Vec::new()
        }
    }

    /// Set the query string on the request path.
    pub fn set_query(&mut self, pairs: Vec<(String, String)>) {
        let path_str = self.path();
        let base = if let Some(idx) = path_str.find('?') {
            path_str[..idx + 1].to_string()
        } else {
            path_str.to_string()
        };
        let query = encode_query(&pairs);
        self.path = format!("{}{}", base, query).into_bytes();
    }

    /// HTTP request first-line format.
    pub fn first_line_format(&self) -> &str {
        if self.method() == "CONNECT" {
            "authority"
        } else if !self.authority.is_empty() {
            "absolute"
        } else {
            "relative"
        }
    }

    /// Returns `true` if HTTP/2.
    pub fn is_http2(&self) -> bool {
        self.data.http_version == b"HTTP/2.0"
    }

    /// Returns `true` if HTTP/3.
    pub fn is_http3(&self) -> bool {
        self.data.http_version == b"HTTP/3"
    }

    // ---- Anti-caching helpers ----

    /// Remove cache-validating headers (If-Modified-Since, If-None-Match).
    pub fn anticache(&mut self) {
        self.data.headers.delete("if-modified-since").ok();
        self.data.headers.delete("if-none-match").ok();
    }

    /// Force Accept-Encoding: identity (no compression).
    pub fn anticomp(&mut self) {
        self.data.headers.set("accept-encoding", "identity");
    }

    /// Limit Accept-Encoding to supported algorithms.
    pub fn constrain_encoding(&mut self) {
        if let Ok(accept_encoding) = self.data.headers.get("accept-encoding") {
            let supported: Vec<&str> = ["gzip", "identity", "deflate", "br", "zstd"]
                .iter()
                .filter(|e| accept_encoding.contains(*e))
                .copied()
                .collect();
            if !supported.is_empty() {
                self.data
                    .headers
                    .set("accept-encoding", &supported.join(", "));
            }
        }
    }
}

impl Default for Request {
    fn default() -> Self {
        Self::new()
    }
}

// ---- Response ----

/// An HTTP response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Response {
    /// Base message data.
    pub data: MessageData,
    /// Streaming mode.
    pub stream: StreamMode,
    /// HTTP status code.
    pub status_code: u16,
    /// Reason phrase (ISO-8859-1 encoded).
    pub reason: Vec<u8>,
}

impl Response {
    /// Create a new Response with default values (200 OK).
    pub fn new() -> Self {
        Self {
            data: MessageData {
                http_version: b"HTTP/1.1".to_vec(),
                headers: Headers::new(),
                content: None,
                trailers: None,
                timestamp_start: mitm_core::flow::current_timestamp(),
                timestamp_end: None,
            },
            stream: StreamMode::default(),
            status_code: 200,
            reason: b"OK".to_vec(),
        }
    }

    /// Simplified factory: create a response from status code + optional content + headers.
    pub fn make(
        status_code: u16,
        content: Option<&[u8]>,
        headers: &[(String, String)],
    ) -> Self {
        let reason = status_reason(status_code);
        let mut resp = Self {
            data: MessageData {
                http_version: b"HTTP/1.1".to_vec(),
                headers: Headers::new(),
                content: None,
                trailers: None,
                timestamp_start: mitm_core::flow::current_timestamp(),
                timestamp_end: None,
            },
            stream: StreamMode::default(),
            status_code,
            reason: reason.bytes().collect(),
        };
        if let Some(body) = content {
            resp.data.content = Some(body.to_vec());
            resp.data
                .headers
                .set("content-length", &body.len().to_string());
        }
        for (k, v) in headers {
            resp.data.headers.set(k, v);
        }
        resp
    }

    // ---- Properties ----

    // ---- Content management (delegated) ----

    /// Raw body bytes.
    pub fn raw_content(&self) -> Option<&[u8]> {
        self.data.content.as_deref()
    }

    /// Uncompressed body bytes.
    pub fn content(&self) -> Option<&[u8]> {
        self.data.content.as_deref()
    }

    /// Body as text string.
    pub fn text(&self) -> Option<String> {
        self.data.content.as_ref().map(|c| String::from_utf8_lossy(c).into_owned())
    }

    /// Headers reference.
    pub fn headers(&self) -> &Headers {
        &self.data.headers
    }

    /// Parse body as JSON.
    pub fn json<T: serde::de::DeserializeOwned>(&self) -> Result<T, std::io::Error> {
        let content = self
            .data
            .content
            .as_ref()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "no content"))?;
        serde_json::from_slice(content).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
    }

    /// HTTP status code.
    pub fn status_code(&self) -> u16 {
        self.status_code
    }

    /// Set HTTP status code.
    pub fn set_status_code(&mut self, code: u16) {
        self.status_code = code;
        self.reason = status_reason(code).bytes().collect();
    }

    /// Reason phrase as string (ISO-8859-1 decoded).
    pub fn reason(&self) -> String {
        String::from_utf8_lossy(&self.reason).into_owned()
    }

    /// Set reason phrase.
    pub fn set_reason(&mut self, reason: &str) {
        self.reason = reason.bytes().collect();
    }

    /// Refresh date-related headers for replay.
    ///
    /// Adjusts Date, Expires, Last-Modified headers and Set-Cookie expirations
    /// by the elapsed time since the original timestamp.
    pub fn refresh(&mut self) {
        let now = mitm_core::flow::current_timestamp();
        let delta = now - self.data.timestamp_start;
        let date_headers = ["date", "expires", "last-modified"];
        for header in &date_headers {
            if let Ok(value) = self.data.headers.get(header) {
                // In v1, date adjustment is a no-op (would need date parsing).
                let _ = (value, delta);
            }
        }
    }
}

impl Default for Response {
    fn default() -> Self {
        Self::new()
    }
}

// ---- Helpers ----

/// Default port for a scheme.
fn default_port(scheme: &str) -> u16 {
    match scheme {
        "http" | "ws" => 80,
        "https" | "wss" => 443,
        _ => 80,
    }
}

/// Format host:port, omitting port if it's the default.
fn format_host_port(host: &str, port: u16) -> String {
    if port == default_port("http") || port == default_port("https") {
        host.to_string()
    } else {
        format!("{}:{}", host, port)
    }
}

/// Get the standard reason phrase for a status code.
fn status_reason(code: u16) -> &'static str {
    match code {
        100 => "Continue",
        101 => "Switching Protocols",
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        303 => "See Other",
        304 => "Not Modified",
        307 => "Temporary Redirect",
        308 => "Permanent Redirect",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        408 => "Request Timeout",
        409 => "Conflict",
        410 => "Gone",
        411 => "Length Required",
        413 => "Payload Too Large",
        414 => "URI Too Long",
        415 => "Unsupported Media Type",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        _ => "Unknown",
    }
}

/// Simple URL percent-decoding.
fn url_decode(s: &str) -> String {
    let mut result = String::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                result.push(byte as char);
                i += 3;
                continue;
            }
            result.push(bytes[i] as char);
            i += 1;
            continue;
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_make() {
        let req = Request::make("GET", "http://example.com/path?q=1", None, &[]);
        assert_eq!(req.method(), "GET");
        assert_eq!(req.host(), "example.com");
        assert_eq!(req.port(), 80);
        assert_eq!(req.path(), "/path?q=1");
        assert_eq!(req.scheme(), "http");
    }

    #[test]
    fn test_request_url_set() {
        let mut req = Request::new();
        req.url_set("https://example.com:8443/api/v1");
        assert_eq!(req.scheme(), "https");
        assert_eq!(req.host(), "example.com");
        assert_eq!(req.port(), 8443);
        assert_eq!(req.path(), "/api/v1");
    }

    #[test]
    fn test_request_url_roundtrip() {
        let req = Request::make("GET", "http://example.com/path?q=1", None, &[]);
        let url = req.url();
        assert_eq!(url, "http://example.com/path?q=1");
    }

    #[test]
    fn test_request_content_management() {
        let mut req = Request::new();
        req.set_method("POST");
        req.set_content(Some(b"hello world".to_vec()));
        assert_eq!(req.content().unwrap(), b"hello world");
        assert_eq!(req.headers().get("content-length").unwrap(), "11");
        assert_eq!(req.text().unwrap(), "hello world");
    }

    #[test]
    fn test_request_set_text() {
        let mut req = Request::new();
        req.set_text(Some("hello".to_string()));
        assert_eq!(req.content().unwrap(), b"hello");
    }

    #[test]
    fn test_request_path_components() {
        let req = Request::make("GET", "http://example.com/api/v1/users", None, &[]);
        assert_eq!(req.path_components(), vec!["api", "v1", "users"]);
    }

    #[test]
    fn test_request_first_line_format() {
        let mut req = Request::make("GET", "http://example.com/", None, &[]);
        assert_eq!(req.first_line_format(), "relative");
        req.authority = b"example.com:443".to_vec();
        assert_eq!(req.first_line_format(), "absolute");
        req.method = b"CONNECT".to_vec();
        assert_eq!(req.first_line_format(), "authority");
    }

    #[test]
    fn test_request_anticache() {
        let mut req = Request::make(
            "GET",
            "http://example.com/",
            None,
            &[
                ("if-modified-since".to_string(), "Wed, 21 Oct 2015".to_string()),
                ("if-none-match".to_string(), "xyzzy".to_string()),
            ],
        );
        req.anticache();
        assert!(!req.headers().contains("if-modified-since"));
        assert!(!req.headers().contains("if-none-match"));
    }

    #[test]
    fn test_response_make() {
        let resp = Response::make(200, Some(b"OK".as_ref()), &[]);
        assert_eq!(resp.status_code(), 200);
        assert_eq!(resp.reason(), "OK");
        assert_eq!(resp.content().unwrap(), b"OK");
    }

    #[test]
    fn test_response_status_change() {
        let mut resp = Response::new();
        resp.set_status_code(404);
        assert_eq!(resp.status_code(), 404);
        assert_eq!(resp.reason(), "Not Found");
    }

    #[test]
    fn test_stream_mode_default() {
        let mode = StreamMode::default();
        assert!(matches!(mode, StreamMode::Buffered));
    }
}

/// Encode query parameters as a URL query string.
pub fn encode_query(pairs: &[(String, String)]) -> String {
    pairs
        .iter()
        .map(|(k, v)| {
            format!(
                "{}={}",
                url_encode(k),
                url_encode(v)
            )
        })
        .collect::<Vec<_>>()
        .join("&")
}

/// Decode a URL query string into pairs.
pub fn decode_query(query: &str) -> Vec<(String, String)> {
    if query.is_empty() {
        return Vec::new();
    }
    query
        .split('&')
        .filter_map(|pair| {
            let mut iter = pair.splitn(2, '=');
            let key = iter.next()?.to_string();
            let value = iter.next().unwrap_or("").to_string();
            Some((url_decode(&key), url_decode(&value)))
        })
        .collect()
}

/// URL-encode a string (application/x-www-form-urlencoded).
pub fn url_encode(s: &str) -> String {
    let mut out = String::new();
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            b' ' => out.push('+'),
            _ => {
                out.push('%');
                out.push(hex_char(byte >> 4));
                out.push(hex_char(byte & 0x0F));
            }
        }
    }
    out
}

fn hex_char(n: u8) -> char {
    match n {
        n if n < 10 => (b'0' + n) as char,
        n => (b'a' + n - 10) as char,
    }
}
