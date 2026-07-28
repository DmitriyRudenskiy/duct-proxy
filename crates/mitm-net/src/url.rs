//! URL parsing module.

use thiserror::Error;

/// Parsed URL components.
#[derive(Debug, Clone)]
pub struct UrlComponents {
    /// URL scheme (http, https, etc.)
    pub scheme: String,
    /// Hostname or IP address
    pub host: String,
    /// Port number (None for default ports)
    pub port: Option<u16>,
    /// URL path
    pub path: String,
    /// Query string (without leading ?)
    pub query: Option<String>,
    /// Fragment (without leading #)
    pub fragment: Option<String>,
}

/// Errors that can occur during URL parsing.
#[derive(Error, Debug)]
pub enum UrlParseError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Missing scheme")]
    MissingScheme,

    #[error("Missing host")]
    MissingHost,

    #[error("Invalid port: {0}")]
    InvalidPort(String),
}

/// URL parser.
pub struct UrlParser;

impl UrlParser {
    /// Parse a URL string into components.
    pub fn parse(input: &str) -> Result<UrlComponents, UrlParseError> {
        let mut scheme = None;
        let mut rest = input;

        // Parse scheme
        if let Some(colon_pos) = rest.find("://") {
            scheme = Some(&rest[..colon_pos]);
            rest = &rest[colon_pos + 3..];
        } else {
            // Try to parse scheme without ://
            if let Some(colon_pos) = rest.find(':') {
                let potential_scheme = &rest[..colon_pos];
                if potential_scheme.chars().all(|c| c.is_alphanumeric() || c == '+' || c == '-' || c == '.') {
                    scheme = Some(potential_scheme);
                    rest = &rest[colon_pos + 1..];
                }
            }
        }

        let scheme = scheme.unwrap_or("http").to_string();

        // Parse authority (host:port)
        let (authority, path_query_fragment) = if let Some(slash_pos) = rest.find('/') {
            (&rest[..slash_pos], &rest[slash_pos..])
        } else {
            (rest, "")
        };

        let (host, port) = if let Some(bracket_end) = authority.find(']') {
            // IPv6 host
            let host = &authority[..bracket_end + 1];
            let port = if authority.len() > bracket_end + 1 && authority.as_bytes()[bracket_end + 1] == b':' {
                Some(authority[bracket_end + 2..].parse::<u16>().map_err(|e| UrlParseError::InvalidPort(e.to_string()))?)
            } else {
                None
            };
            (host.to_string(), port)
        } else if let Some(colon_pos) = authority.rfind(':') {
            // Regular host:port
            let host = &authority[..colon_pos];
            let port = Some(authority[colon_pos + 1..].parse::<u16>().map_err(|e| UrlParseError::InvalidPort(e.to_string()))?);
            (host.to_string(), port)
        } else {
            (authority.to_string(), None)
        };

        if host.is_empty() {
            return Err(UrlParseError::MissingHost);
        }

        // Parse path, query, fragment
        let (path, query, fragment) = if let Some(hash_pos) = path_query_fragment.find('#') {
            let before_hash = &path_query_fragment[..hash_pos];
            let frag = &path_query_fragment[hash_pos + 1..];
            if let Some(query_pos) = before_hash.find('?') {
                (&before_hash[..query_pos], Some(before_hash[query_pos + 1..].to_string()), Some(frag.to_string()))
            } else {
                (before_hash, None, Some(frag.to_string()))
            }
        } else if let Some(query_pos) = path_query_fragment.find('?') {
            (&path_query_fragment[..query_pos], Some(path_query_fragment[query_pos + 1..].to_string()), None)
        } else {
            (path_query_fragment, None, None)
        };

        Ok(UrlComponents {
            scheme,
            host,
            port,
            path: if path.is_empty() { "/".to_string() } else { path.to_string() },
            query,
            fragment,
        })
    }

    /// Reconstruct a URL from components.
    pub fn reconstruct(components: &UrlComponents) -> String {
        let mut url = String::new();
        url.push_str(&components.scheme);
        url.push_str("://");
        url.push_str(&components.host);
        if let Some(port) = components.port {
            url.push(':');
            url.push_str(&port.to_string());
        }
        url.push_str(&components.path);
        if let Some(query) = &components.query {
            url.push('?');
            url.push_str(query);
        }
        if let Some(fragment) = &components.fragment {
            url.push('#');
            url.push_str(fragment);
        }
        url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_url() {
        let url = "https://example.com:8443/path?q=1#frag";
        let components = UrlParser::parse(url).unwrap();
        assert_eq!(components.scheme, "https");
        assert_eq!(components.host, "example.com");
        assert_eq!(components.port, Some(8443));
        assert_eq!(components.path, "/path");
        assert_eq!(components.query, Some("q=1".to_string()));
        assert_eq!(components.fragment, Some("frag".to_string()));
    }

    #[test]
    fn test_parse_default_port() {
        let url = "http://example.com/path";
        let components = UrlParser::parse(url).unwrap();
        assert_eq!(components.scheme, "http");
        assert_eq!(components.host, "example.com");
        assert_eq!(components.port, None);
        assert_eq!(components.path, "/path");
    }

    #[test]
    fn test_parse_ipv6() {
        let url = "http://[::1]:8080/path";
        let components = UrlParser::parse(url).unwrap();
        assert_eq!(components.host, "[::1]");
        assert_eq!(components.port, Some(8080));
    }

    #[test]
    fn test_reconstruct_url() {
        let components = UrlComponents {
            scheme: "https".to_string(),
            host: "example.com".to_string(),
            port: Some(443),
            path: "/path".to_string(),
            query: Some("q=1".to_string()),
            fragment: Some("frag".to_string()),
        };
        let url = UrlParser::reconstruct(&components);
        assert_eq!(url, "https://example.com:443/path?q=1#frag");
    }
}
