## ADDED Requirements

### Requirement: DNS message structure

The system SHALL define a `DNSMessage` struct representing a complete DNS message:

- `id: u16` — query identifier
- `query: bool` — `true` for queries, `false` for responses
- `op_code: u8` — operation code (0=QUERY, 1=IQUERY, 2=STATUS, 4=UPDATE, 5=RESERVED)
- `authoritative_answer: bool` — whether the responding name server is authoritative
- `truncation: bool` — whether the message was truncated
- `recursion_desired: bool` — whether recursion was requested
- `recursion_available: bool` — whether recursion is available
- `reserved: u8` — reserved bits (must be zero)
- `response_code: u8` — response code (0=NOERROR, 1=FORMERR, 2=SERVFAIL, 3=NXDOMAIN, 4=NOTIMP, 5=REFUSED)
- `questions: Vec<Question>` — question section
- `answers: Vec<ResourceRecord>` — answer section
- `authorities: Vec<ResourceRecord>` — authority section
- `additionals: Vec<ResourceRecord>` — additional section
- `timestamp: Option<f64>` — when the message was sent/received

DNS messages SHALL provide factory methods:

- `succeed(answers: Vec<ResourceRecord>) -> DNSMessage` — create a success response
- `fail(response_code: u8) -> DNSMessage` — create an error response
- `copy() -> DNSMessage` — create a copy with a new random ID

#### Scenario: DNS message has correct header flags

- **WHEN** a DNSMessage is constructed as a query
- **THEN** `query = true` and `response_code = 0`

#### Scenario: Succeed creates valid response

- **WHEN** `DNSMessage.succeed(answers)` is called on a query
- **THEN** the resulting message has `query = false`, `response_code = 0`, and the provided answers

#### Scenario: Fail creates error response

- **WHEN** `DNSMessage.fail(NXDOMAIN)` is called
- **THEN** the resulting message has `query = false` and the error response code

#### Scenario: Copy generates new ID

- **WHEN** `msg.copy()` is called
- **THEN** the copy has a different random ID but identical other fields

### Requirement: DNS question

The system SHALL define a `Question` struct:

- `name: String` — domain name being queried (e.g., "example.com")
- `type_: u16` — record type (1=A, 28=AAAA, 5=CNAME, 16=TXT, etc.)
- `class_: u16` — class (1=IN, 3=CH, 4=CS)

`DNSMessage.question` SHALL return `Some(&Question)` if there is exactly one question, else `None`.

#### Scenario: Single question shorthand

- **WHEN** a DNS message has exactly one question
- **THEN** `msg.question` returns `Some(question)`

#### Scenario: Multiple questions returns None

- **WHEN** a DNS message has two or more questions
- **THEN** `msg.question` returns `None`

### Requirement: Resource record with typed accessors

The system SHALL define a `ResourceRecord` struct:

- `name: String` — owner name
- `type_: u16` — record type
- `class_: u16` — class
- `ttl: u32` — time to live in seconds
- `data: Vec<u8>` — raw RDATA bytes

The system SHALL provide typed accessor properties on `ResourceRecord`:

- `ipv4_address: Ipv4Addr` — parse A record data as IPv4
- `ipv6_address: Ipv6Addr` — parse AAAA record data as IPv6
- `domain_name: String` — parse CNAME/PTR/NS record as domain name
- `text: String` — parse TXT record as UTF-8 string

Setting these accessors MUST update `data` with the correctly encoded bytes.

The system SHALL provide constructor methods:

- `ResourceRecord::A(name, ip) -> ResourceRecord`
- `ResourceRecord::AAAA(name, ip) -> ResourceRecord`
- `ResourceRecord::CNAME(alias, canonical) -> ResourceRecord`
- `ResourceRecord::PTR(inaddr, ptr) -> ResourceRecord`
- `ResourceRecord::TXT(name, text) -> ResourceRecord`
- `ResourceRecord::HTTPS(name, record) -> ResourceRecord`

#### Scenario: A record provides IPv4 address

- **WHEN** a ResourceRecord of type A has `data = 0xC0A80001`
- **THEN** `record.ipv4_address` returns `192.168.0.1`

#### Scenario: CNAME record provides domain name

- **WHEN** a ResourceRecord of type CNAME is accessed
- **THEN** `record.domain_name` returns the canonical domain string

#### Scenario: TXT record provides text

- **WHEN** a ResourceRecord of type TXT has UTF-8 data
- **THEN** `record.text` returns the decoded string

#### Scenario: Constructor creates properly encoded record

- **WHEN** `ResourceRecord::A("example.com", Ipv4Addr::new(93, 184, 216, 34))` is called
- **THEN** the record has type A, class IN, and packed IPv4 bytes in data

### Requirement: DNS wire format encode/decode

The system SHALL provide wire format serialization and deserialization for DNS messages:

- `DNSMessage::unpack(buffer: &[u8]) -> DNSMessage` — parse from wire bytes
- `DNSMessage::packed: Vec<u8>` — serialize to wire bytes

The implementation MUST handle:

- Domain name compression pointers (as per RFC 1035)
- All standard record types (A, AAAA, CNAME, PTR, TXT, NS, SOA, MX, SRV, HTTPS, etc.)
- Proper error reporting for malformed messages

#### Scenario: Pack and unpack round-trip

- **WHEN** a DNSMessage is serialized to bytes and deserialized back
- **THEN** all fields (questions, answers, flags, rcode) are preserved

#### Scenario: Malformed message fails with error

- **WHEN** `DNSMessage::unpack` is called on truncated bytes
- **THEN** it returns an error (does not panic)

### Requirement: DNS flow

The system SHALL define a `DNSFlow` struct containing:

- `base: FlowBase` — shared flow state
- `request: DNSMessage` — the DNS query
- `response: Option<DNSMessage>` — the DNS response (None if unanswered)

#### Scenario: DNS flow without response

- **WHEN** a DNS query is sent but no response is received
- **THEN** `flow.response` is `None` and `flow.error` may be set

#### Scenario: DNS flow with response

- **WHEN** a DNS query receives a response
- **THEN** `flow.response` is `Some(response_message)`

#### Scenario: DNSFlow is a Flow variant

- **WHEN** a DNS query is intercepted
- **THEN** it is represented as `Flow::Dns(DNSFlow)`

### Requirement: DNS JSON serialization

All DNS types SHALL be serializable to JSON for the web UI:

- `DNSMessage.to_json() -> serde_json::Value` — full JSON representation
- `Question.to_json()` — question as JSON
- `ResourceRecord.to_json()` — record as JSON with type-appropriate data representation

JSON output MUST be consumable by a TypeScript frontend (`web/src/flow.ts`).

#### Scenario: DNS message serializes to JSON

- **WHEN** a DNSMessage is converted to JSON
- **THEN** the output includes id, flags, questions, answers with type-appropriate data
