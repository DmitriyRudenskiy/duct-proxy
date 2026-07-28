//! Chunked transfer-encoding module.

use thiserror::Error;

/// Trailer name-value pairs.
type Trailers = Vec<(String, String)>;

/// Errors that can occur during chunked encoding/decoding.
#[derive(Error, Debug)]
pub enum ChunkedError {
    #[error("Invalid chunk size: {0}")]
    InvalidChunkSize(String),

    #[error("Invalid chunk extension: {0}")]
    InvalidExtension(String),

    #[error("Truncated chunked data")]
    Truncated,

    #[error("Invalid chunked format: {0}")]
    InvalidFormat(String),
}

/// Decode chunked transfer-encoded data.
///
/// Returns the decoded body and any trailers.
pub fn decode(input: &[u8]) -> Result<(Vec<u8>, Trailers), ChunkedError> {
    let mut body = Vec::new();
    let mut trailers = Vec::new();
    let mut pos = 0;

    while pos < input.len() {
        // Find end of chunk size line
        let line_end = input[pos..]
            .iter()
            .position(|&b| b == b'\n')
            .ok_or(ChunkedError::Truncated)?;

        // Extract chunk size line (without CRLF)
        let mut chunk_size_line = &input[pos..pos + line_end];
        if chunk_size_line.ends_with(b"\r") {
            chunk_size_line = &chunk_size_line[..chunk_size_line.len() - 1];
        }

        let chunk_size_str = std::str::from_utf8(chunk_size_line).map_err(|e| ChunkedError::InvalidFormat(e.to_string()))?;

        // Parse chunk size (may have chunk extensions after semicolon)
        let chunk_size_str = if let Some(semi_pos) = chunk_size_str.find(';') {
            &chunk_size_str[..semi_pos]
        } else {
            chunk_size_str
        };

        let chunk_size = u64::from_str_radix(chunk_size_str.trim(), 16)
            .map_err(|e| ChunkedError::InvalidChunkSize(e.to_string()))?;

        // Move past the chunk size line and its CRLF
        // line_end is the position of \n, so we move to line_end + 1 (past \n)
        pos += line_end + 1;
        // If there was a \r before the \n, it's already been skipped by stripping it from chunk_size_line
        // But we need to make sure we're past the CRLF
        if pos < input.len() && input[pos] == b'\r' {
            pos += 1;
        }
        if pos < input.len() && input[pos] == b'\n' {
            pos += 1;
        }

        // Check for last chunk
        if chunk_size == 0 {
            // Read trailers until empty line
            while pos < input.len() {
                let trailer_end = input[pos..]
                    .iter()
                    .position(|&b| b == b'\n')
                    .ok_or(ChunkedError::Truncated)?;

                let mut trailer_line = &input[pos..pos + trailer_end];
                if trailer_line.ends_with(b"\r") {
                    trailer_line = &trailer_line[..trailer_line.len() - 1];
                }

                let trailer_str = std::str::from_utf8(trailer_line).map_err(|e| ChunkedError::InvalidFormat(e.to_string()))?;
                pos += trailer_end + 1;
                if pos < input.len() && input[pos] == b'\r' {
                    pos += 1;
                }

                if trailer_str.is_empty() {
                    // End of trailers
                    break;
                }

                if let Some(eq_pos) = trailer_str.find(':') {
                    let name = trailer_str[..eq_pos].trim().to_string();
                    let value = trailer_str[eq_pos + 1..].trim().to_string();
                    trailers.push((name, value));
                }
            }

            break;
        }

        // Read chunk data
        if pos + chunk_size as usize > input.len() {
            return Err(ChunkedError::Truncated);
        }

        body.extend_from_slice(&input[pos..pos + chunk_size as usize]);
        pos += chunk_size as usize;

        // Skip CRLF after chunk data
        if pos < input.len() && input[pos] == b'\r' {
            pos += 1;
        }
        if pos < input.len() && input[pos] == b'\n' {
            pos += 1;
        }
    }

    Ok((body, trailers))
}

/// Encode data as chunked transfer-encoded.
pub fn encode(body: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();

    if body.is_empty() {
        result.extend_from_slice(b"0\r\n\r\n");
        return result;
    }

    // Split body into chunks (max 8KB per chunk for safety)
    const MAX_CHUNK_SIZE: usize = 8192;
    let mut pos = 0;

    while pos < body.len() {
        let end = std::cmp::min(pos + MAX_CHUNK_SIZE, body.len());
        let chunk_size = end - pos;

        // Write chunk size in hex
        result.extend_from_slice(format!("{:x}\r\n", chunk_size).as_bytes());
        result.extend_from_slice(&body[pos..end]);
        result.extend_from_slice(b"\r\n");

        pos = end;
    }

    // Write last chunk
    result.extend_from_slice(b"0\r\n\r\n");

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_simple() {
        let input = b"5\r\nHello\r\n0\r\n\r\n";
        let (body, trailers) = decode(input).unwrap();
        assert_eq!(body, b"Hello");
        assert!(trailers.is_empty());
    }

    #[test]
    fn test_decode_multiple_chunks() {
        let input = b"5\r\nHello\r
5\r\nWorld\r
0\r\n\r\n";
        let (body, _) = decode(input).unwrap();
        assert_eq!(body, b"HelloWorld");
    }

    #[test]
    fn test_decode_empty() {
        let input = b"0\r\n\r\n";
        let (body, _) = decode(input).unwrap();
        assert_eq!(body, b"");
    }

    #[test]
    fn test_encode_simple() {
        let body = b"Hello";
        let encoded = encode(body);
        assert_eq!(encoded, b"5\r\nHello\r\n0\r\n\r\n");
    }

    #[test]
    fn test_encode_empty() {
        let body = b"";
        let encoded = encode(body);
        assert_eq!(encoded, b"0\r\n\r\n");
    }

    #[test]
    fn test_roundtrip() {
        let original = b"Hello, World!";
        let encoded = encode(original);
        let (decoded, _) = decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }
}
