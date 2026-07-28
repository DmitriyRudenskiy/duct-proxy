//! SNI (Server Name Indication) extraction from TLS ClientHello.

/// Extract SNI hostname from raw TLS ClientHello bytes.
///
/// # Arguments
/// * `client_hello` - Raw bytes of a TLS ClientHello message
///
/// # Returns
/// Some(hostname) if SNI is present and valid, None otherwise
pub fn extract_sni(client_hello: &[u8]) -> Option<String> {
    // TLS record header: 5 bytes (type, version, length)
    // Handshake header: 4 bytes (type, version, length)
    // Then ClientHello content

    if client_hello.len() < 9 {
        return None;
    }

    // Skip TLS record header (5 bytes) and handshake header (4 bytes)
    let mut offset = 9;

    // Parse ClientHello
    // client_version: 2 bytes
    // random: 32 bytes
    // session_id_length: 1 byte
    // session_id: session_id_length bytes
    // cipher_suites_length: 2 bytes
    // cipher_suites: cipher_suites_length bytes
    // compression_methods_length: 1 byte
    // compression_methods: compression_methods_length bytes

    if offset + 2 > client_hello.len() {
        return None;
    }
    offset += 2; // client_version

    if offset + 32 > client_hello.len() {
        return None;
    }
    offset += 32; // random

    if offset + 1 > client_hello.len() {
        return None;
    }
    let session_id_length = client_hello[offset] as usize;
    offset += 1;

    if offset + session_id_length > client_hello.len() {
        return None;
    }
    offset += session_id_length;

    if offset + 2 > client_hello.len() {
        return None;
    }
    let cipher_suites_length =
        ((client_hello[offset] as usize) << 8) | (client_hello[offset + 1] as usize);
    offset += 2;

    if offset + cipher_suites_length > client_hello.len() {
        return None;
    }
    offset += cipher_suites_length;

    if offset + 1 > client_hello.len() {
        return None;
    }
    let compression_methods_length = client_hello[offset] as usize;
    offset += 1;

    if offset + compression_methods_length > client_hello.len() {
        return None;
    }
    offset += compression_methods_length;

    // Extensions
    if offset + 2 > client_hello.len() {
        return None;
    }
    let extensions_length =
        ((client_hello[offset] as usize) << 8) | (client_hello[offset + 1] as usize);
    offset += 2;

    if offset + extensions_length > client_hello.len() {
        return None;
    }

    // Parse extensions
    let mut ext_offset = offset;
    let ext_end = offset + extensions_length;

    while ext_offset + 4 <= ext_end {
        let ext_type =
            ((client_hello[ext_offset] as usize) << 8) | (client_hello[ext_offset + 1] as usize);
        let ext_length =
            ((client_hello[ext_offset + 2] as usize) << 8) | (client_hello[ext_offset + 3] as usize);
        ext_offset += 4;

        if ext_offset + ext_length > ext_end {
            return None;
        }

        if ext_type == 0x0000 {
            // Server Name Indication
            if let Some(hostname) = parse_sni_extension(&client_hello[ext_offset..ext_offset + ext_length]) {
                return Some(hostname);
            }
        }

        ext_offset += ext_length;
    }

    None
}

/// Parse SNI extension (type 0x00)
fn parse_sni_extension(data: &[u8]) -> Option<String> {
    if data.len() < 5 {
        return None;
    }

    let sni_list_length =
        ((data[0] as usize) << 8) | (data[1] as usize);

    if data.len() < 2 + sni_list_length {
        return None;
    }

    let mut pos = 2;
    let end = 2 + sni_list_length;

    while pos + 3 <= end {
        let name_type = data[pos];
        let name_length =
            ((data[pos + 1] as usize) << 8) | (data[pos + 2] as usize);
        pos += 3;

        if name_type == 0 && pos + name_length <= end {
            let hostname = std::str::from_utf8(&data[pos..pos + name_length]).ok()?;
            return validate_hostname(hostname);
        }

        pos += name_length;
    }

    None
}

/// Validate hostname (non-empty, valid characters)
fn validate_hostname(hostname: &str) -> Option<String> {
    if hostname.is_empty() {
        return None;
    }

    // Basic validation: alphanumeric, dots, hyphens
    if hostname
        .chars()
        .all(|c| c.is_alphanumeric() || c == '.' || c == '-')
    {
        Some(hostname.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_sni_valid() {
        // Construct a minimal ClientHello with SNI
        let mut client_hello = Vec::new();

        // TLS record header: type=0x16 (handshake), version=TLS 1.0, length=placeholder
        client_hello.push(0x16);
        client_hello.extend_from_slice(&[0x03, 0x01]);
        let record_length_start = client_hello.len();
        client_hello.extend_from_slice(&[0x00, 0x00]);

        // Handshake header: type=0x01 (client_hello), length=placeholder
        client_hello.push(0x01);
        let handshake_length_start = client_hello.len();
        client_hello.extend_from_slice(&[0x00, 0x00, 0x00]);

        // client_version: 2 bytes (TLS 1.0)
        client_hello.extend_from_slice(&[0x03, 0x01]);

        // random: 32 bytes
        client_hello.extend_from_slice(&[0u8; 32]);

        // session_id_length: 0
        client_hello.push(0x00);

        // cipher_suites: empty (0 length)
        client_hello.extend_from_slice(&[0x00, 0x00]);

        // compression_methods: empty (0 length)
        client_hello.push(0x00);

        // extensions length: placeholder
        let extensions_length_start = client_hello.len();
        client_hello.extend_from_slice(&[0x00, 0x00]);

        // SNI extension (type=0x0000, length=variable, data=SNI list)
        client_hello.extend_from_slice(&[0x00, 0x00]); // ext_type
        let sni_data = construct_sni_extension("example.com");
        client_hello.extend_from_slice(&[0x00, sni_data.len() as u8]); // ext_length (only data length)
        client_hello.extend_from_slice(&sni_data); // ext_data

        // Fix extensions length
        let extensions_length = client_hello.len() - extensions_length_start - 2;
        client_hello[extensions_length_start] = (extensions_length >> 8) as u8;
        client_hello[extensions_length_start + 1] = (extensions_length & 0xff) as u8;

        // Fix handshake length (content after handshake header)
        let handshake_length = client_hello.len() - handshake_length_start - 4;
        client_hello[handshake_length_start] = (handshake_length >> 16) as u8;
        client_hello[handshake_length_start + 1] = (handshake_length >> 8) as u8;
        client_hello[handshake_length_start + 2] = (handshake_length & 0xff) as u8;

        // Fix TLS record length
        let record_length = client_hello.len() - record_length_start - 2;
        client_hello[record_length_start] = (record_length >> 8) as u8;
        client_hello[record_length_start + 1] = (record_length & 0xff) as u8;

        let result = extract_sni(&client_hello);
        assert_eq!(result, Some("example.com".to_string()));
    }

    #[test]
    fn test_extract_sni_no_extension() {
        // Construct a ClientHello without extensions
        let mut client_hello = Vec::new();

        client_hello.push(0x01);
        client_hello.extend_from_slice(&[0x03, 0x01]);

        client_hello.push(0x01);
        client_hello.extend_from_slice(&[0x00, 0x00, 0x00]);

        client_hello.extend_from_slice(&[0x03, 0x01]);
        client_hello.extend_from_slice(&[0u8; 32]);
        client_hello.push(0x00);
        client_hello.extend_from_slice(&[0x00, 0x00]);
        client_hello.push(0x00);

        // extensions: empty
        client_hello.extend_from_slice(&[0x00, 0x00]);

        let result = extract_sni(&client_hello);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_sni_invalid() {
        // Invalid SNI with special characters
        let mut client_hello = Vec::new();

        client_hello.push(0x01);
        client_hello.extend_from_slice(&[0x03, 0x01]);

        client_hello.push(0x01);
        client_hello.extend_from_slice(&[0x00, 0x00, 0x00]);

        client_hello.extend_from_slice(&[0x03, 0x01]);
        client_hello.extend_from_slice(&[0u8; 32]);
        client_hello.push(0x00);
        client_hello.extend_from_slice(&[0x00, 0x00]);
        client_hello.push(0x00);

        let sni_data = construct_sni_extension("invalid<host>");
        client_hello.extend_from_slice(&[
            ((sni_data.len() >> 8) & 0xff) as u8,
            (sni_data.len() & 0xff) as u8,
        ]);
        client_hello.extend_from_slice(&sni_data);

        let result = extract_sni(&client_hello);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_sni_too_short() {
        let client_hello = vec![0x01, 0x03, 0x01];
        let result = extract_sni(&client_hello);
        assert_eq!(result, None);
    }

    fn construct_sni_extension(hostname: &str) -> Vec<u8> {
        let hostname_bytes = hostname.as_bytes();
        let mut data = Vec::new();

        // sni_list_length
        let list_len = (hostname_bytes.len() + 3) as u16;
        data.push((list_len >> 8) as u8);
        data.push((list_len & 0xff) as u8);

        // name_type: 0 (host_name)
        data.push(0x00);

        // name_length
        let name_len = hostname_bytes.len() as u16;
        data.push((name_len >> 8) as u8);
        data.push((name_len & 0xff) as u8);

        // hostname
        data.extend_from_slice(hostname_bytes);

        data
    }
}
