//! Cookie parsing module (RFC 6265).

use thiserror::Error;

/// A parsed cookie.
#[derive(Debug, Clone)]
pub struct Cookie {
    /// Cookie name
    pub name: String,
    /// Cookie value
    pub value: String,
    /// Path attribute
    pub path: Option<String>,
    /// Domain attribute
    pub domain: Option<String>,
    /// Expiration time (Unix timestamp)
    pub expires: Option<i64>,
    /// Max-Age in seconds
    pub max_age: Option<i64>,
    /// HttpOnly flag
    pub http_only: bool,
    /// Secure flag
    pub secure: bool,
    /// SameSite attribute
    pub same_site: Option<SameSite>,
}

/// SameSite attribute values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

impl std::fmt::Display for SameSite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SameSite::Strict => write!(f, "Strict"),
            SameSite::Lax => write!(f, "Lax"),
            SameSite::None => write!(f, "None"),
        }
    }
}

/// Errors that can occur during cookie parsing.
#[derive(Error, Debug)]
pub enum CookieParseError {
    #[error("Invalid cookie: {0}")]
    InvalidCookie(String),
}

/// Cookie parser.
pub struct CookieParser;

impl CookieParser {
    /// Parse a Set-Cookie header value.
    pub fn parse_set_cookie(header: &str) -> Result<Cookie, CookieParseError> {
        let parts: Vec<&str> = header.split(';').collect();
        let name_value = parts[0].trim();

        let (name, value) = if let Some(eq_pos) = name_value.find('=') {
            (&name_value[..eq_pos], &name_value[eq_pos + 1..])
        } else {
            return Err(CookieParseError::InvalidCookie("Missing = in cookie".to_string()));
        };

        let mut cookie = Cookie {
            name: name.trim().to_string(),
            value: value.trim().to_string(),
            path: None,
            domain: None,
            expires: None,
            max_age: None,
            http_only: false,
            secure: false,
            same_site: None,
        };

        // Parse attributes
        for attr in parts.iter().skip(1) {
            let attr = attr.trim();
            if attr.is_empty() {
                continue;
            }

            if let Some(eq_pos) = attr.find('=') {
                let key = attr[..eq_pos].trim().to_lowercase();
                let val = attr[eq_pos + 1..].trim();

                match key.as_str() {
                    "path" => cookie.path = Some(val.to_string()),
                    "domain" => cookie.domain = Some(val.to_string()),
                    "expires" => {
                        // Parse expires date (simplified)
                        cookie.expires = Some(0); // TODO: proper date parsing
                    }
                    "max-age" => {
                        cookie.max_age = val.parse().ok();
                    }
                    "httponly" => cookie.http_only = true,
                    "secure" => cookie.secure = true,
                    "samesite" => {
                        cookie.same_site = match val.to_lowercase().as_str() {
                            "strict" => Some(SameSite::Strict),
                            "lax" => Some(SameSite::Lax),
                            "none" => Some(SameSite::None),
                            _ => None,
                        };
                    }
                    _ => {} // Ignore unknown attributes
                }
            } else if attr.to_lowercase() == "httponly" {
                cookie.http_only = true;
            } else if attr.to_lowercase() == "secure" {
                cookie.secure = true;
            }
        }

        Ok(cookie)
    }

    /// Parse a Cookie header value (multiple name=value pairs).
    pub fn parse_cookie(header: &str) -> Vec<Cookie> {
        let mut cookies = Vec::new();

        for pair in header.split(';') {
            let pair = pair.trim();
            if pair.is_empty() {
                continue;
            }

            if let Some(eq_pos) = pair.find('=') {
                let name = pair[..eq_pos].trim().to_string();
                let value = pair[eq_pos + 1..].trim().to_string();
                cookies.push(Cookie {
                    name,
                    value,
                    path: None,
                    domain: None,
                    expires: None,
                    max_age: None,
                    http_only: false,
                    secure: false,
                    same_site: None,
                });
            }
        }

        cookies
    }

    /// Convert a cookie to a Set-Cookie header value.
    pub fn to_set_cookie_header(cookie: &Cookie) -> String {
        let mut header = format!("{}={}", cookie.name, cookie.value);

        if let Some(path) = &cookie.path {
            header.push_str(&format!("; Path={}", path));
        }

        if let Some(domain) = &cookie.domain {
            header.push_str(&format!("; Domain={}", domain));
        }

        if cookie.http_only {
            header.push_str("; HttpOnly");
        }

        if cookie.secure {
            header.push_str("; Secure");
        }

        if let Some(same_site) = cookie.same_site {
            header.push_str(&format!("; SameSite={}", same_site));
        }

        header
    }

    /// Check if a cookie matches a given host and path.
    pub fn matches(cookie: &Cookie, host: &str, path: &str) -> bool {
        // Check domain match
        if let Some(ref domain) = cookie.domain {
            let cookie_domain = domain.strip_prefix('.').unwrap_or(domain);
            if !host.ends_with(cookie_domain) && host != cookie_domain {
                return false;
            }
        }

        // Check path match (prefix)
        if let Some(ref path_attr) = cookie.path
            && !path.starts_with(path_attr)
        {
            return false;
        }

        // Check expiration
        if let Some(expires) = cookie.expires {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            if now > expires {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_cookie() {
        let cookie = CookieParser::parse_set_cookie("name=value").unwrap();
        assert_eq!(cookie.name, "name");
        assert_eq!(cookie.value, "value");
        assert_eq!(cookie.path, None);
        assert_eq!(cookie.http_only, false);
    }

    #[test]
    fn test_parse_cookie_with_attributes() {
        let cookie = CookieParser::parse_set_cookie("name=value; Path=/; HttpOnly; Secure").unwrap();
        assert_eq!(cookie.name, "name");
        assert_eq!(cookie.value, "value");
        assert_eq!(cookie.path, Some("/".to_string()));
        assert_eq!(cookie.http_only, true);
        assert_eq!(cookie.secure, true);
    }

    #[test]
    fn test_parse_cookie_header() {
        let cookies = CookieParser::parse_cookie("name1=value1; name2=value2");
        assert_eq!(cookies.len(), 2);
        assert_eq!(cookies[0].name, "name1");
        assert_eq!(cookies[1].name, "name2");
    }

    #[test]
    fn test_to_set_cookie_header() {
        let cookie = Cookie {
            name: "name".to_string(),
            value: "value".to_string(),
            path: Some("/".to_string()),
            domain: None,
            expires: None,
            max_age: None,
            http_only: true,
            secure: true,
            same_site: Some(SameSite::Strict),
        };
        let header = CookieParser::to_set_cookie_header(&cookie);
        assert_eq!(header, "name=value; Path=/; HttpOnly; Secure; SameSite=Strict");
    }

    #[test]
    fn test_cookie_matching() {
        let cookie = Cookie {
            name: "test".to_string(),
            value: "val".to_string(),
            path: Some("/".to_string()),
            domain: Some(".example.com".to_string()),
            expires: None,
            max_age: None,
            http_only: false,
            secure: false,
            same_site: None,
        };
        assert!(CookieParser::matches(&cookie, "www.example.com", "/path"));
        assert!(!CookieParser::matches(&cookie, "other.com", "/path"));
    }
}
