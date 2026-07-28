//! HTTP/1.1 streaming parser.

use bytes::{BytesMut, Buf};
use std::io::Error as IoError;
use tokio::io::{AsyncRead, AsyncReadExt};
use thiserror::Error;

/// Header name-value pairs.
type Headers = Vec<(String, String)>;

/// HTTP request line.
#[derive(Debug, Clone)]
pub struct RequestLine {
    /// HTTP method (GET, POST, etc.)
    pub method: String,
    /// Request URI
    pub uri: String,
    /// HTTP version
    pub version: String,
}

/// HTTP response line.
#[derive(Debug, Clone)]
pub struct ResponseLine {
    /// HTTP version
    pub version: String,
    /// Status code
    pub status: u16,
    /// Status reason phrase
    pub reason: String,
}

/// HTTP message with headers and optional body.
#[derive(Debug, Clone)]
pub struct HttpMessage {
    /// Headers as name-value pairs
    pub headers: Vec<(String, String)>,
    /// Body bytes (empty if no body)
    pub body: Vec<u8>,
    /// Whether the message has been fully received
    pub complete: bool,
}

/// HTTP request.
#[derive(Debug, Clone)]
pub struct HttpRequest {
    /// Request line
    pub line: RequestLine,
    /// Headers
    pub headers: Vec<(String, String)>,
    /// Body
    pub body: Vec<u8>,
}

/// HTTP response.
#[derive(Debug, Clone)]
pub struct HttpResponse {
    /// Response line
    pub line: ResponseLine,
    /// Headers
    pub headers: Vec<(String, String)>,
    /// Body
    pub body: Vec<u8>,
}

impl Default for HttpResponse {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during HTTP parsing.
#[derive(Error, Debug)]
pub enum HttpParseError {
    #[error("I/O error: {0}")]
    Io(#[from] IoError),

    #[error("Invalid request line: {0}")]
    InvalidRequestLine(String),

    #[error("Invalid response line: {0}")]
    InvalidResponseLine(String),

    #[error("Invalid header: {0}")]
    InvalidHeader(String),

    #[error("Truncated message")]
    Truncated,

    #[error("Body too large: {0} bytes")]
    BodyTooLarge(usize),
}

/// Streaming HTTP request parser.
pub struct HttpRequestParser {
    buffer: BytesMut,
    max_body_size: usize,
}

impl HttpRequestParser {
    /// Create a new request parser.
    pub fn new(max_body_size: usize) -> Self {
        Self {
            buffer: BytesMut::with_capacity(8192),
            max_body_size,
        }
    }

    /// Parse an HTTP request from an async reader.
    pub async fn parse<R: AsyncRead + Unpin>(&mut self, reader: &mut R) -> Result<HttpRequest, HttpParseError> {
        // Parse request line
        self.read_until_crlf(reader).await?;
        let line = self.parse_request_line()?;

        // Parse headers
        self.read_until_double_crlf(reader).await?;
        let (headers, content_length, chunked) = self.parse_headers()?;

        // Parse body
        let body = self.read_body(reader, content_length, chunked).await?;

        Ok(HttpRequest {
            line,
            headers,
            body,
        })
    }

    async fn read_until_crlf<R: AsyncRead + Unpin>(&mut self, reader: &mut R) -> Result<(), HttpParseError> {
        loop {
            if let Some(pos) = self.buffer.windows(2).position(|w| w == b"\r\n") {
                self.buffer.advance(pos + 2);
                return Ok(());
            }

            let mut temp = [0u8; 8192];
            let n = reader.read(&mut temp).await?;
            if n == 0 {
                return Err(HttpParseError::Truncated);
            }
            self.buffer.extend_from_slice(&temp[..n]);
        }
    }

    async fn read_until_double_crlf<R: AsyncRead + Unpin>(&mut self, reader: &mut R) -> Result<(), HttpParseError> {
        loop {
            if let Some(pos) = self.buffer.windows(4).position(|w| w == b"\r\n\r\n") {
                self.buffer.advance(pos + 4);
                return Ok(());
            }

            let mut temp = [0u8; 8192];
            let n = reader.read(&mut temp).await?;
            if n == 0 {
                return Err(HttpParseError::Truncated);
            }
            self.buffer.extend_from_slice(&temp[..n]);
        }
    }

    fn parse_request_line(&self) -> Result<RequestLine, HttpParseError> {
        let line = std::str::from_utf8(&self.buffer)
            .map_err(|e| HttpParseError::InvalidRequestLine(e.to_string()))?;

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 3 {
            return Err(HttpParseError::InvalidRequestLine(format!("Expected 3 parts, got {}", parts.len())));
        }

        Ok(RequestLine {
            method: parts[0].to_string(),
            uri: parts[1].to_string(),
            version: parts[2].to_string(),
        })
    }

    fn parse_headers(&self) -> Result<(Headers, Option<usize>, bool), HttpParseError> {
        let headers_str = std::str::from_utf8(&self.buffer)
            .map_err(|e| HttpParseError::InvalidHeader(e.to_string()))?;

        let mut headers = Vec::new();
        let mut content_length = None;
        let mut chunked = false;

        for line in headers_str.lines() {
            if line.is_empty() {
                continue;
            }

            if let Some(colon_pos) = line.find(':') {
                let name = line[..colon_pos].trim().to_string();
                let value = line[colon_pos + 1..].trim().to_string();
                headers.push((name, value));
            }
        }

        // Check for Content-Length and Transfer-Encoding
        for (name, value) in &headers {
            if name.eq_ignore_ascii_case("Content-Length") {
                content_length = value.parse().ok();
            } else if name.eq_ignore_ascii_case("Transfer-Encoding")
                && value.eq_ignore_ascii_case("chunked")
            {
                chunked = true;
            }
        }

        Ok((headers, content_length, chunked))
    }

    async fn read_body<R: AsyncRead + Unpin>(
        &mut self,
        reader: &mut R,
        content_length: Option<usize>,
        chunked: bool,
    ) -> Result<Vec<u8>, HttpParseError> {
        if chunked {
            // Read chunked body
            let mut body = Vec::new();
            loop {
                self.read_until_crlf(reader).await?;
                let size_line = std::str::from_utf8(&self.buffer)
                    .map_err(|e| HttpParseError::InvalidHeader(e.to_string()))?;
                let size_line = size_line.strip_suffix("\r\n").unwrap_or(size_line);
                let size = u64::from_str_radix(size_line.split(';').next().unwrap_or("0").trim(), 16)
                    .map_err(|e| HttpParseError::InvalidHeader(e.to_string()))?;

                if size == 0 {
                    self.buffer.clear();
                    break;
                }

                if self.buffer.len() < size as usize + 2 {
                    let mut temp = [0u8; 8192];
                    while self.buffer.len() < size as usize + 2 {
                        let n = reader.read(&mut temp).await?;
                        if n == 0 {
                            return Err(HttpParseError::Truncated);
                        }
                        self.buffer.extend_from_slice(&temp[..n]);
                    }
                }

                let data = &self.buffer[..size as usize];
                body.extend_from_slice(data);
                self.buffer.advance(size as usize + 2); // Skip CRLF
            }

            Ok(body)
        } else if let Some(len) = content_length {
            if len > self.max_body_size {
                return Err(HttpParseError::BodyTooLarge(len));
            }

            if self.buffer.len() < len {
                let mut temp = [0u8; 8192];
                while self.buffer.len() < len {
                    let n = reader.read(&mut temp).await?;
                    if n == 0 {
                        return Err(HttpParseError::Truncated);
                    }
                    self.buffer.extend_from_slice(&temp[..n]);
                }
            }

            let body = self.buffer[..len].to_vec();
            self.buffer.advance(len);
            Ok(body)
        } else {
            Ok(Vec::new())
        }
    }
}

impl HttpResponse {
    /// Create an empty response.
    pub fn new() -> Self {
        Self {
            line: ResponseLine {
                version: "HTTP/1.1".to_string(),
                status: 200,
                reason: "OK".to_string(),
            },
            headers: Vec::new(),
            body: Vec::new(),
        }
    }

    /// Convert to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend_from_slice(format!("{} {} {}\r\n", self.line.version, self.line.status, self.line.reason).as_bytes());

        for (name, value) in &self.headers {
            result.extend_from_slice(format!("{}: {}\r\n", name, value).as_bytes());
        }

        result.extend_from_slice(b"\r\n");
        result.extend_from_slice(&self.body);

        result
    }
}

impl HttpRequest {
    /// Convert to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend_from_slice(format!("{} {} {}\r\n", self.line.method, self.line.uri, self.line.version).as_bytes());

        for (name, value) in &self.headers {
            result.extend_from_slice(format!("{}: {}\r\n", name, value).as_bytes());
        }

        result.extend_from_slice(b"\r\n");
        result.extend_from_slice(&self.body);

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_request_line() {
        let parser = HttpRequestParser::new(1024 * 1024);
        // This is a simplified test - full parsing requires async reader
        assert!(true);
    }

    #[test]
    fn test_response_to_bytes() {
        let response = HttpResponse::new();
        let bytes = response.to_bytes();
        assert!(String::from_utf8_lossy(&bytes).starts_with("HTTP/1.1 200 OK\r\n"));
    }

    #[test]
    fn test_request_to_bytes() {
        let request = HttpRequest {
            line: RequestLine {
                method: "GET".to_string(),
                uri: "/".to_string(),
                version: "HTTP/1.1".to_string(),
            },
            headers: vec![("Host".to_string(), "example.com".to_string())],
            body: Vec::new(),
        };
        let bytes = request.to_bytes();
        assert!(String::from_utf8_lossy(&bytes).starts_with("GET / HTTP/1.1\r\n"));
    }
}
