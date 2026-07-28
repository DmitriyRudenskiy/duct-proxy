//! Form parsing module.

use thiserror::Error;

/// A parsed form field.
#[derive(Debug, Clone, PartialEq)]
pub enum FormField {
    /// Text field with name and value
    Text { name: String, value: String },
    /// File upload with name, filename, content type, and data
    File {
        name: String,
        filename: String,
        content_type: String,
        data: Vec<u8>,
    },
}

/// Parsed form data.
#[derive(Debug, Clone, Default)]
pub struct FormFields {
    /// Form fields
    pub fields: Vec<FormField>,
}

/// Errors that can occur during form parsing.
#[derive(Error, Debug)]
pub enum FormParseError {
    #[error("Invalid form data: {0}")]
    InvalidData(String),

    #[error("Missing boundary")]
    MissingBoundary,

    #[error("Invalid multipart: {0}")]
    InvalidMultipart(String),
}

/// Form parser.
pub struct FormParser;

impl FormParser {
    /// Parse URL-encoded form data.
    pub fn parse_url_encoded(body: &[u8]) -> Result<FormFields, FormParseError> {
        let body_str = std::str::from_utf8(body)
            .map_err(|e| FormParseError::InvalidData(e.to_string()))?;

        let mut fields = Vec::new();

        for pair in body_str.split('&') {
            if pair.is_empty() {
                continue;
            }

            let (name, value) = if let Some(eq_pos) = pair.find('=') {
                let name = &pair[..eq_pos];
                let value = &pair[eq_pos + 1..];
                (name, value)
            } else {
                (pair, "")
            };

            // URL-decode
            let name = url_decode(name);
            let value = url_decode(value);

            fields.push(FormField::Text { name, value });
        }

        Ok(FormFields { fields })
    }

    /// Parse multipart/form-data.
    pub fn parse_multipart(body: &[u8], boundary: &str) -> Result<FormFields, FormParseError> {
        if boundary.is_empty() {
            return Err(FormParseError::MissingBoundary);
        }

        let body_str = std::str::from_utf8(body)
            .map_err(|e| FormParseError::InvalidData(e.to_string()))?;

        let boundary_marker = format!("--{}", boundary);
        let fields = Self::parse_multipart_parts(body_str, &boundary_marker)?;

        Ok(FormFields { fields })
    }

    fn parse_multipart_parts(body: &str, boundary: &str) -> Result<Vec<FormField>, FormParseError> {
        let mut fields = Vec::new();
        let mut remaining = body;

        while remaining.starts_with(boundary) {
            // Skip boundary and CRLF
            remaining = &remaining[boundary.len()..];
            if remaining.starts_with("\r\n") {
                remaining = &remaining[2..];
            } else if remaining.starts_with('\n') {
                remaining = &remaining[1..];
            }

            // Check for end boundary
            if remaining.starts_with("--") {
                break;
            }

            // Parse part headers
            let (headers, _body_start) = if let Some(double_crlf) = remaining.find("\r\n\r\n") {
                (&remaining[..double_crlf], &remaining[double_crlf + 4..])
            } else if let Some(double_lf) = remaining.find("\n\n") {
                (&remaining[..double_lf], &remaining[double_lf + 2..])
            } else {
                break;
            };

            // Parse Content-Disposition header
            let mut name = None;
            let mut filename = None;
            let mut content_type = String::new();

            for header_line in headers.split('\n') {
                let header_line = header_line.trim().trim_end_matches('\r');
                if header_line.to_lowercase().starts_with("content-disposition:") {
                    let value = &header_line["Content-Disposition:".len()..].trim();
                    for param in value.split(';') {
                        let param = param.trim();
                        if let Some(eq_pos) = param.find('=') {
                            let key = param[..eq_pos].trim().to_lowercase();
                            let val = param[eq_pos + 1..].trim().trim_matches('"');
                            match key.as_str() {
                                "name" => name = Some(val.to_string()),
                                "filename" => filename = Some(val.to_string()),
                                _ => {}
                            }
                        }
                    }
                } else if header_line.to_lowercase().starts_with("content-type:") {
                    content_type = header_line["Content-Type:".len()..].trim().to_string();
                }
            }

            // Find end of part (next boundary or end)
            let part_end = remaining.find(&format!("\r\n{}", boundary))
                .or_else(|| remaining.find(&format!("\n{}", boundary)))
                .unwrap_or(remaining.len());

            let part_body = &remaining[..part_end];
            let part_body = part_body.strip_suffix("\r\n").or_else(|| part_body.strip_suffix('\n')).unwrap_or(part_body);

            remaining = &remaining[part_end + boundary.len()..];
            if remaining.starts_with("\r\n") {
                remaining = &remaining[2..];
            } else if remaining.starts_with('\n') {
                remaining = &remaining[1..];
            }

            let name = name.ok_or(FormParseError::InvalidMultipart("Missing name in Content-Disposition".to_string()))?;

            let field = if let Some(filename) = filename {
                FormField::File {
                    name,
                    filename,
                    content_type,
                    data: part_body.as_bytes().to_vec(),
                }
            } else {
                FormField::Text {
                    name,
                    value: part_body.to_string(),
                }
            };

            fields.push(field);
        }

        Ok(fields)
    }

    /// Convert form fields to URL-encoded format.
    pub fn to_url_encoded(fields: &[FormField]) -> Vec<u8> {
        let mut result = Vec::new();

        for (i, field) in fields.iter().enumerate() {
            if i > 0 {
                result.extend_from_slice(b"&");
            }

            if let FormField::Text { name, value } = field {
                result.extend_from_slice(&url_encode(name));
                result.extend_from_slice(b"=");
                result.extend_from_slice(&url_encode(value));
            }
            // Skip files for URL encoding
        }

        result
    }

    /// Convert form fields to multipart format.
    pub fn to_multipart(fields: &[FormField], boundary: &str) -> Vec<u8> {
        let mut result = Vec::new();

        for field in fields {
            result.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());

            match field {
                FormField::Text { name, value } => {
                    result.extend_from_slice(b"Content-Disposition: form-data; name=\"");
                    result.extend_from_slice(name.as_bytes());
                    result.extend_from_slice(b"\"\r\n\r\n");
                    result.extend_from_slice(value.as_bytes());
                    result.extend_from_slice(b"\r\n");
                }
                FormField::File { name, filename, content_type, data } => {
                    result.extend_from_slice(b"Content-Disposition: form-data; name=\"");
                    result.extend_from_slice(name.as_bytes());
                    result.extend_from_slice(b"\"; filename=\"");
                    result.extend_from_slice(filename.as_bytes());
                    result.extend_from_slice(b"\"\r\n");
                    result.extend_from_slice(format!("Content-Type: {}\r\n", content_type).as_bytes());
                    result.extend_from_slice(b"\r\n");
                    result.extend_from_slice(data);
                    result.extend_from_slice(b"\r\n");
                }
            }
        }

        result.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());

        result
    }
}

/// URL-encode a string.
fn url_encode(input: &str) -> Vec<u8> {
    let mut result = Vec::new();
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte);
            }
            b' ' => {
                result.extend_from_slice(b"%20");
            }
            _ => {
                result.push(b'%');
                result.push(hex_byte(byte >> 4));
                result.push(hex_byte(byte & 0x0f));
            }
        }
    }
    result
}

/// URL-decode a string.
fn url_decode(input: &str) -> String {
    let mut result = Vec::new();
    let bytes = input.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hi = hex_val(bytes[i + 1]);
            let lo = hex_val(bytes[i + 2]);
            if let (Some(hi), Some(lo)) = (hi, lo) {
                result.push((hi << 4) | lo);
                i += 3;
                continue;
            }
        } else if bytes[i] == b'+' {
            result.push(b' ');
            i += 1;
            continue;
        }
        result.push(bytes[i]);
        i += 1;
    }

    String::from_utf8_lossy(&result).to_string()
}

fn hex_byte(b: u8) -> u8 {
    if b < 10 { b'0' + b } else { b'a' + b - 10 }
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url_encoded() {
        let form = FormParser::parse_url_encoded(b"name=John&age=30").unwrap();
        assert_eq!(form.fields.len(), 2);
        assert_eq!(form.fields[0], FormField::Text { name: "name".to_string(), value: "John".to_string() });
        assert_eq!(form.fields[1], FormField::Text { name: "age".to_string(), value: "30".to_string() });
    }

    #[test]
    fn test_parse_url_encoded_special_chars() {
        let form = FormParser::parse_url_encoded(b"name=John%20Doe").unwrap();
        if let FormField::Text { value, .. } = &form.fields[0] {
            assert_eq!(value, "John Doe");
        } else {
            panic!("Expected Text field");
        }
    }

    #[test]
    fn test_to_url_encoded() {
        let fields = vec![
            FormField::Text { name: "name".to_string(), value: "John".to_string() },
            FormField::Text { name: "age".to_string(), value: "30".to_string() },
        ];
        let encoded = FormParser::to_url_encoded(&fields);
        assert_eq!(String::from_utf8(encoded).unwrap(), "name=John&age=30");
    }
}
