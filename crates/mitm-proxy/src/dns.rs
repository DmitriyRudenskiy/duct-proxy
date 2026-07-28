//! DNS protocol types: `Question`, `ResourceRecord`, `DNSMessage`, `DNSFlow`.
//!
//! Provides wire-format serialization (`packed`) and deserialization (`unpack`)
//! per RFC 1035, including domain name compression pointers.

use serde::{Deserialize, Serialize};

// ---- DNS record type constants (newtype wrappers for serde compat) ----

/// DNS record types.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DnsType(u16);

impl DnsType {
    pub const A: Self = Self(1);
    pub const NS: Self = Self(2);
    pub const CNAME: Self = Self(5);
    pub const SOA: Self = Self(6);
    pub const PTR: Self = Self(12);
    pub const MX: Self = Self(15);
    pub const TXT: Self = Self(16);
    pub const AAAA: Self = Self(28);
    pub const SRV: Self = Self(33);
    pub const HTTPS: Self = Self(65);

    pub fn from_u16(val: u16) -> Self {
        Self(val)
    }

    pub fn to_str(&self) -> &'static str {
        match self.0 {
            1 => "A",
            2 => "NS",
            5 => "CNAME",
            6 => "SOA",
            12 => "PTR",
            15 => "MX",
            16 => "TXT",
            28 => "AAAA",
            33 => "SRV",
            65 => "HTTPS",
            _ => "TYPE?",
        }
    }
}

impl Serialize for DnsType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DnsType {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let val = u16::deserialize(deserializer)?;
        Ok(Self(val))
    }
}

/// DNS class constants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DnsClass(u16);

impl DnsClass {
    pub const IN: Self = Self(1);
    pub const CH: Self = Self(3);
    pub const CS: Self = Self(4);
    pub const HS: Self = Self(4);

    pub fn from_u16(val: u16) -> Self {
        Self(val)
    }

    pub fn to_str(&self) -> &'static str {
        match self.0 {
            1 => "IN",
            3 => "CH",
            4 => "CS",
            _ => "CLASS?",
        }
    }
}

impl Serialize for DnsClass {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DnsClass {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let val = u16::deserialize(deserializer)?;
        Ok(Self(val))
    }
}

/// DNS response codes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rcode(u8);

impl Rcode {
    pub const NOERROR: Self = Self(0);
    pub const FORMERR: Self = Self(1);
    pub const SERVFAIL: Self = Self(2);
    pub const NXDOMAIN: Self = Self(3);
    pub const NOTIMP: Self = Self(4);
    pub const REFUSED: Self = Self(5);

    pub fn from_u8(val: u8) -> Self {
        Self(val)
    }

    pub fn to_str(&self) -> &'static str {
        match self.0 {
            0 => "NOERROR",
            1 => "FORMERR",
            2 => "SERVFAIL",
            3 => "NXDOMAIN",
            4 => "NOTIMP",
            5 => "REFUSED",
            _ => "RCODE?",
        }
    }
}

impl Serialize for Rcode {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Rcode {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let val = u8::deserialize(deserializer)?;
        Ok(Self(val))
    }
}

// ---- DNS message types ----

/// A DNS question section entry.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Question {
    /// Domain name being queried.
    pub name: String,
    /// Record type.
    pub type_: DnsType,
    /// Class.
    pub class_: DnsClass,
}

impl Question {
    /// Convert to JSON for the web UI.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "type": self.type_.to_str(),
            "class": self.class_.to_str(),
        })
    }
}

/// A DNS resource record.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceRecord {
    /// Owner name.
    pub name: String,
    /// Record type.
    pub type_: DnsType,
    /// Class.
    pub class_: DnsClass,
    /// Time to live in seconds.
    pub ttl: u32,
    /// Raw RDATA bytes.
    pub data: Vec<u8>,
}

impl ResourceRecord {
    /// Default TTL.
    pub const DEFAULT_TTL: u32 = 60;

    /// Parse as IPv4 address (A record).
    pub fn ipv4_address(&self) -> Option<std::net::Ipv4Addr> {
        if self.type_.0 == 1 && self.data.len() == 4 {
            let octets: [u8; 4] = self.data[..4].try_into().ok()?;
            Some(std::net::Ipv4Addr::from(octets))
        } else {
            None
        }
    }

    /// Set as IPv4 address.
    pub fn set_ipv4_address(&mut self, ip: std::net::Ipv4Addr) {
        self.type_ = DnsType::A;
        self.data = ip.octets().to_vec();
    }

    /// Parse as IPv6 address (AAAA record).
    pub fn ipv6_address(&self) -> Option<std::net::Ipv6Addr> {
        if self.type_.0 == 28 && self.data.len() == 16 {
            let octets: [u8; 16] = self.data[..16].try_into().ok()?;
            Some(std::net::Ipv6Addr::from(octets))
        } else {
            None
        }
    }

    /// Set as IPv6 address.
    pub fn set_ipv6_address(&mut self, ip: std::net::Ipv6Addr) {
        self.type_ = DnsType::AAAA;
        self.data = ip.octets().to_vec();
    }

    /// Parse as domain name (CNAME, PTR, NS records).
    pub fn domain_name(&self) -> Option<String> {
        unpack_domain_name_raw(&self.data).ok()
    }

    /// Set as domain name.
    pub fn set_domain_name(&mut self, name: &str) {
        self.data = pack_domain_name(name);
    }

    /// Parse as text (TXT record).
    pub fn text(&self) -> Option<String> {
        if self.type_.0 == 16 {
            String::from_utf8(self.data.clone()).ok()
        } else {
            None
        }
    }

    /// Set as text.
    pub fn set_text(&mut self, text: &str) {
        self.type_ = DnsType::TXT;
        self.data = text.as_bytes().to_vec();
    }

    // ---- Constructors ----

    /// Create an A record.
    pub fn a(name: &str, ip: std::net::Ipv4Addr) -> Self {
        Self {
            name: name.to_string(),
            type_: DnsType::A,
            class_: DnsClass::IN,
            ttl: Self::DEFAULT_TTL,
            data: ip.octets().to_vec(),
        }
    }

    /// Create an AAAA record.
    pub fn aaaa(name: &str, ip: std::net::Ipv6Addr) -> Self {
        Self {
            name: name.to_string(),
            type_: DnsType::AAAA,
            class_: DnsClass::IN,
            ttl: Self::DEFAULT_TTL,
            data: ip.octets().to_vec(),
        }
    }

    /// Create a CNAME record.
    pub fn cname(alias: &str, canonical: &str) -> Self {
        Self {
            name: alias.to_string(),
            type_: DnsType::CNAME,
            class_: DnsClass::IN,
            ttl: Self::DEFAULT_TTL,
            data: pack_domain_name(canonical),
        }
    }

    /// Create a PTR record.
    pub fn ptr(inaddr: &str, ptr: &str) -> Self {
        Self {
            name: inaddr.to_string(),
            type_: DnsType::PTR,
            class_: DnsClass::IN,
            ttl: Self::DEFAULT_TTL,
            data: pack_domain_name(ptr),
        }
    }

    /// Create a TXT record.
    pub fn txt(name: &str, text: &str) -> Self {
        Self {
            name: name.to_string(),
            type_: DnsType::TXT,
            class_: DnsClass::IN,
            ttl: Self::DEFAULT_TTL,
            data: text.as_bytes().to_vec(),
        }
    }

    /// Convert to JSON for the web UI.
    pub fn to_json(&self) -> serde_json::Value {
        let data_str = match self.type_.0 {
            1 => self
                .ipv4_address()
                .map(|ip| ip.to_string())
                .unwrap_or_else(|| format!("0x{}", hex_encode(&self.data))),
            28 => self
                .ipv6_address()
                .map(|ip| ip.to_string())
                .unwrap_or_else(|| format!("0x{}", hex_encode(&self.data))),
            5 | 12 | 2 => {
                self.domain_name().unwrap_or_else(|| format!("0x{}", hex_encode(&self.data)))
            }
            16 => self.text().unwrap_or_else(|| format!("0x{}", hex_encode(&self.data))),
            _ => format!("0x{}", hex_encode(&self.data)),
        };
        serde_json::json!({
            "name": self.name,
            "type": self.type_.to_str(),
            "class": self.class_.to_str(),
            "ttl": self.ttl,
            "data": data_str,
        })
    }
}

/// A complete DNS message (query or response).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DNSMessage {
    /// Query identifier.
    pub id: u16,
    /// `true` for queries, `false` for responses.
    pub query: bool,
    /// Operation code.
    pub op_code: u8,
    /// Authoritative answer flag.
    pub authoritative_answer: bool,
    /// Truncation flag.
    pub truncation: bool,
    /// Recursion desired flag.
    pub recursion_desired: bool,
    /// Recursion available flag.
    pub recursion_available: bool,
    /// Reserved bits.
    pub reserved: u8,
    /// Response code.
    pub response_code: Rcode,
    /// Question section.
    pub questions: Vec<Question>,
    /// Answer section.
    pub answers: Vec<ResourceRecord>,
    /// Authority section.
    pub authorities: Vec<ResourceRecord>,
    /// Additional section.
    pub additionals: Vec<ResourceRecord>,
    /// Timestamp.
    pub timestamp: Option<f64>,
}

impl DNSMessage {
    /// Create a success response from a query.
    pub fn succeed(&self, answers: Vec<ResourceRecord>) -> Self {
        Self {
            id: self.id,
            query: false,
            op_code: self.op_code,
            authoritative_answer: false,
            truncation: false,
            recursion_desired: self.recursion_desired,
            recursion_available: true,
            reserved: 0,
            response_code: Rcode::NOERROR,
            questions: self.questions.clone(),
            answers,
            authorities: Vec::new(),
            additionals: Vec::new(),
            timestamp: Some(mitm_core::flow::current_timestamp()),
        }
    }

    /// Create an error response.
    pub fn fail(&self, rcode: Rcode) -> Self {
        if rcode.0 == 0 {
            panic!("fail() requires an error rcode");
        }
        Self {
            id: self.id,
            query: false,
            op_code: self.op_code,
            authoritative_answer: false,
            truncation: false,
            recursion_desired: self.recursion_desired,
            recursion_available: false,
            reserved: 0,
            response_code: rcode,
            questions: self.questions.clone(),
            answers: Vec::new(),
            authorities: Vec::new(),
            additionals: Vec::new(),
            timestamp: Some(mitm_core::flow::current_timestamp()),
        }
    }

    /// Create a copy with a new random ID.
    pub fn copy(&self) -> Self {
        Self {
            id: rand_u16(),
            ..self.clone()
        }
    }

    /// Returns the single question if present, else None.
    pub fn question(&self) -> Option<&Question> {
        if self.questions.len() == 1 {
            Some(&self.questions[0])
        } else {
            None
        }
    }

    /// Total data size of all resource record sections.
    pub fn size(&self) -> usize {
        self.answers
            .iter()
            .chain(self.authorities.iter())
            .chain(self.additionals.iter())
            .map(|rr| rr.data.len())
            .sum()
    }

    // ---- Wire format: serialize ----

    /// Serialize to DNS wire format bytes.
    pub fn packed(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(12 + self.questions.len() * 5);

        // Header (12 bytes)
        let flags = self.build_flags();
        buf.extend_from_slice(&self.id.to_be_bytes());
        buf.extend_from_slice(&flags.to_be_bytes());
        buf.extend_from_slice(&(self.questions.len() as u16).to_be_bytes());
        buf.extend_from_slice(&(self.answers.len() as u16).to_be_bytes());
        buf.extend_from_slice(&(self.authorities.len() as u16).to_be_bytes());
        buf.extend_from_slice(&(self.additionals.len() as u16).to_be_bytes());

        // Questions
        for q in &self.questions {
            buf.extend_from_slice(&pack_domain_name(&q.name));
            buf.extend_from_slice(&(q.type_.0).to_be_bytes());
            buf.extend_from_slice(&(q.class_.0).to_be_bytes());
        }

        // Resource records
        for rr in self.answers.iter().chain(self.authorities.iter()).chain(self.additionals.iter())
        {
            buf.extend_from_slice(&pack_domain_name(&rr.name));
            buf.extend_from_slice(&(rr.type_.0).to_be_bytes());
            buf.extend_from_slice(&(rr.class_.0).to_be_bytes());
            buf.extend_from_slice(&rr.ttl.to_be_bytes());
            buf.extend_from_slice(&(rr.data.len() as u16).to_be_bytes());
            buf.extend_from_slice(&rr.data);
        }

        buf
    }

    fn build_flags(&self) -> u16 {
        let mut flags: u16 = 0;
        if !self.query {
            flags |= 1 << 15;
        }
        flags |= (self.op_code as u16) << 11;
        if self.authoritative_answer {
            flags |= 1 << 10;
        }
        if self.truncation {
            flags |= 1 << 9;
        }
        if self.recursion_desired {
            flags |= 1 << 8;
        }
        if self.recursion_available {
            flags |= 1 << 7;
        }
        flags |= (self.reserved as u16) << 4;
        flags |= self.response_code.0 as u16;
        flags
    }

    // ---- Wire format: deserialize ----

    /// Parse a DNS message from wire format bytes.
    pub fn unpack(data: &[u8]) -> Result<Self, DnsError> {
        if data.len() < 12 {
            return Err(DnsError::TooShort);
        }

        let id = u16::from_be_bytes([data[0], data[1]]);
        let flags = u16::from_be_bytes([data[2], data[3]]);

        let query = flags & (1 << 15) == 0;
        let op_code = ((flags >> 11) & 0b1111) as u8;
        let authoritative_answer = flags & (1 << 10) != 0;
        let truncation = flags & (1 << 9) != 0;
        let recursion_desired = flags & (1 << 8) != 0;
        let recursion_available = flags & (1 << 7) != 0;
        let reserved = ((flags >> 4) & 0b111) as u8;
        let response_code = (flags & 0b1111) as u8;

        let qdcount = u16::from_be_bytes([data[4], data[5]]) as usize;
        let ancount = u16::from_be_bytes([data[6], data[7]]) as usize;
        let nscount = u16::from_be_bytes([data[8], data[9]]) as usize;
        let arcount = u16::from_be_bytes([data[10], data[11]]) as usize;

        let mut pos = 12;
        let mut questions = Vec::with_capacity(qdcount);
        let mut answers = Vec::with_capacity(ancount);
        let mut authorities = Vec::with_capacity(nscount);
        let mut additionals = Vec::with_capacity(arcount);

        // Parse questions
        for _ in 0..qdcount {
            let (name, new_pos) = parse_domain_name(data, pos)?;
            pos = new_pos;
            if data.len() - pos < 4 {
                return Err(DnsError::Truncated);
            }
            let type_ = DnsType::from_u16(u16::from_be_bytes([data[pos], data[pos + 1]]));
            let class_ = DnsClass::from_u16(u16::from_be_bytes([data[pos + 2], data[pos + 3]]));
            pos += 4;
            questions.push(Question { name, type_, class_ });
        }

        // Parse resource records
        let parse_rr = |data: &[u8], pos: &mut usize| -> Result<ResourceRecord, DnsError> {
            let (name, new_pos) = parse_domain_name(data, *pos)?;
            *pos = new_pos;
            if data.len() - *pos < 6 {
                return Err(DnsError::Truncated);
            }
            let type_ = DnsType::from_u16(u16::from_be_bytes([data[*pos], data[*pos + 1]]));
            let class_ = DnsClass::from_u16(u16::from_be_bytes([data[*pos + 2], data[*pos + 3]]));
            let ttl = u32::from_be_bytes([data[*pos + 4], data[*pos + 5], data[*pos + 6], data[*pos + 7]]);
            *pos += 8;
            let rdlength = u16::from_be_bytes([data[*pos], data[*pos + 1]]) as usize;
            *pos += 2;
            if data.len() - *pos < rdlength {
                return Err(DnsError::Truncated);
            }
            let rdata = data[*pos..*pos + rdlength].to_vec();
            *pos += rdlength;
            Ok(ResourceRecord { name, type_, class_, ttl, data: rdata })
        };

        for _ in 0..ancount {
            answers.push(parse_rr(data, &mut pos)?);
        }
        for _ in 0..nscount {
            authorities.push(parse_rr(data, &mut pos)?);
        }
        for _ in 0..arcount {
            additionals.push(parse_rr(data, &mut pos)?);
        }

        Ok(Self {
            id,
            query,
            op_code,
            authoritative_answer,
            truncation,
            recursion_desired,
            recursion_available,
            reserved,
            response_code: Rcode::from_u8(response_code),
            questions,
            answers,
            authorities,
            additionals,
            timestamp: None,
        })
    }

    /// Convert to JSON for the web UI.
    pub fn to_json(&self) -> serde_json::Value {
        let mut json = serde_json::json!({
            "id": self.id,
            "query": self.query,
            "op_code": self.op_code,
            "authoritative_answer": self.authoritative_answer,
            "truncation": self.truncation,
            "recursion_desired": self.recursion_desired,
            "recursion_available": self.recursion_available,
            "response_code": self.response_code.to_str(),
            "questions": self.questions.iter().map(|q| q.to_json()).collect::<Vec<_>>(),
            "answers": self.answers.iter().map(|rr| rr.to_json()).collect::<Vec<_>>(),
            "authorities": self.authorities.iter().map(|rr| rr.to_json()).collect::<Vec<_>>(),
            "additionals": self.additionals.iter().map(|rr| rr.to_json()).collect::<Vec<_>>(),
            "size": self.size(),
        });
        if let Some(ts) = self.timestamp {
            json["timestamp"] = serde_json::Value::Number(serde_json::Number::from_f64(ts).unwrap_or(serde_json::Number::from(0)));
        }
        json
    }
}

/// DNS flow: a DNS query/response pair.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DNSFlow {
    #[serde(flatten)]
    pub base: mitm_core::flow::FlowBase,
    pub request: DNSMessage,
    pub response: Option<DNSMessage>,
}

// ---- Wire format helpers ----

/// Pack a domain name into wire format.
fn pack_domain_name(name: &str) -> Vec<u8> {
    let mut result = Vec::new();
    for label in name.split('.') {
        if label.is_empty() {
            continue;
        }
        let label_bytes = label.as_bytes();
        result.push(label_bytes.len() as u8);
        result.extend_from_slice(label_bytes);
    }
    result.push(0); // root label
    result
}

/// Parse a domain name from wire format, handling compression pointers.
fn parse_domain_name(data: &[u8], mut pos: usize) -> Result<(String, usize), DnsError> {
        let mut jumped = false;
        let mut labels = Vec::new();

    for _ in 0..64 {
        if pos >= data.len() {
            return Err(DnsError::Truncated);
        }
        let len = data[pos] as usize;

        if len == 0 {
            if !jumped {
                pos += 1;
            }
            break;
        }

        if len & 0xC0 == 0xC0 {
            // Compression pointer
            if pos + 1 >= data.len() {
                return Err(DnsError::Truncated);
            }
            let pointer = ((len & 0x3F) << 8) | data[pos + 1] as usize;
            if !jumped {
                jumped = true;
            }
            pos = pointer;
            continue;
        }

        pos += 1;
        if pos + len > data.len() {
            return Err(DnsError::Truncated);
        }
        let label = String::from_utf8_lossy(&data[pos..pos + len]).into_owned();
        labels.push(label);
        pos += len;
    }

    let name = labels.join(".");

    Ok((name, pos))
}

/// Unpack raw RDATA as a domain name (for CNAME/PTR/NS records).
fn unpack_domain_name_raw(data: &[u8]) -> Result<String, DnsError> {
    let (name, _) = parse_domain_name(data, 0)?;
    Ok(name)
}

/// Simple hex encoding for display.
fn hex_encode(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Generate a random u16 for DNS ID.
fn rand_u16() -> u16 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    (nanos ^ (nanos >> 16)) as u16
}

/// DNS parsing errors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DnsError {
    TooShort,
    Truncated,
    InvalidCompression,
}

impl std::fmt::Display for DnsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooShort => write!(f, "DNS message too short"),
            Self::Truncated => write!(f, "DNS message truncated"),
            Self::InvalidCompression => write!(f, "invalid compression pointer"),
        }
    }
}

impl std::error::Error for DnsError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_query(name: &str, type_: DnsType) -> DNSMessage {
        DNSMessage {
            id: 12345,
            query: true,
            op_code: 0,
            authoritative_answer: false,
            truncation: false,
            recursion_desired: true,
            recursion_available: false,
            reserved: 0,
            response_code: Rcode::NOERROR,
            questions: vec![Question {
                name: name.to_string(),
                type_,
                class_: DnsClass::IN,
            }],
            answers: Vec::new(),
            authorities: Vec::new(),
            additionals: Vec::new(),
            timestamp: None,
        }
    }

    #[test]
    fn test_dns_wire_format_roundtrip() {
        let query = make_query("example.com", DnsType::A);
        let packed = query.packed();
        let parsed = DNSMessage::unpack(&packed).unwrap();
        assert_eq!(parsed.id, 12345);
        assert!(parsed.query);
        assert_eq!(parsed.questions[0].name, "example.com");
        assert_eq!(parsed.questions[0].type_, DnsType::A);
        assert_eq!(parsed.questions[0].class_, DnsClass::IN);
        assert_eq!(parsed.questions.len(), 1);
    }

    #[test]
    fn test_succeed_factory() {
        let query = make_query("example.com", DnsType::A);
        let answer = ResourceRecord::a("example.com", "93.184.216.34".parse().unwrap());
        let response = query.succeed(vec![answer]);
        assert!(!response.query);
        assert_eq!(response.response_code, Rcode::NOERROR);
        assert_eq!(response.answers.len(), 1);
        assert_eq!(response.answers[0].ipv4_address().unwrap().to_string(), "93.184.216.34");
    }

    #[test]
    fn test_fail_factory() {
        let query = make_query("missing.com", DnsType::A);
        let response = query.fail(Rcode::NXDOMAIN);
        assert!(!response.query);
        assert_eq!(response.response_code, Rcode::NXDOMAIN);
        assert!(response.answers.is_empty());
    }

    #[test]
    fn test_rr_convenience_accessors() {
        let mut rr = ResourceRecord::a("test.com", "10.0.0.1".parse().unwrap());
        assert_eq!(rr.ipv4_address().unwrap().to_string(), "10.0.0.1");

        let mut rr2 = ResourceRecord::cname("alias.com", "canonical.com");
        assert_eq!(rr2.domain_name().unwrap(), "canonical.com");

        let mut rr3 = ResourceRecord::txt("test.com", "v=spf1");
        assert_eq!(rr3.text().unwrap(), "v=spf1");
    }

    #[test]
    fn test_copy_generates_new_id() {
        let query = make_query("example.com", DnsType::A);
        let copy = query.copy();
        assert_ne!(copy.id, query.id);
        assert_eq!(copy.questions[0].name, "example.com");
    }

    #[test]
    fn test_question_shorthand() {
        let query = make_query("example.com", DnsType::A);
        assert!(query.question().is_some());
        assert_eq!(query.question().unwrap().name, "example.com");

        let multi = DNSMessage {
            questions: vec![
                Question { name: "a.com".into(), type_: DnsType::A, class_: DnsClass::IN },
                Question { name: "b.com".into(), type_: DnsType::A, class_: DnsClass::IN },
            ],
            ..query
        };
        assert!(multi.question().is_none());
    }

    #[test]
    fn test_unsupported_wire_format_returns_error() {
        assert!(DNSMessage::unpack(&[0u8; 5]).is_err());
    }

    #[test]
    fn test_dns_to_json() {
        let query = make_query("example.com", DnsType::A);
        let json = query.to_json();
        assert_eq!(json["id"], 12345);
        assert_eq!(json["query"], true);
        assert_eq!(json["questions"][0]["name"], "example.com");
    }
}
