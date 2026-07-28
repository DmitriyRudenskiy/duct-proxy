## ADDED Requirements

### Requirement: Headers type — order-preserving case-insensitive MultiDict

The system SHALL define a `Headers` type that:

- Preserves insertion order of header fields
- Performs case-insensitive lookups (HTTP header names are case-insensitive per RFC 7230)
- Supports multiple values per key (e.g., multiple `Set-Cookie` headers)
- Stores all keys and values as `Vec<u8>` (bytes) internally

Internal representation MAY use `Vec<(Vec<u8>, Vec<u8>)>` with a `HashMap<String, Vec<usize>>` lookup index (lowercase key → indices).

#### Scenario: Case-insensitive header lookup

- **WHEN** `headers["host"]` is accessed
- **THEN** it finds the header regardless of original casing (e.g., "Host", "HOST", "host")

#### Scenario: Header order is preserved

- **WHEN** headers are inserted in order and iterated
- **THEN** they are returned in insertion order

#### Scenario: Multiple values for same key

- **WHEN** two `Set-Cookie` headers are added
- **THEN** `get_all("set-cookie")` returns both values as separate strings

#### Scenario: Setting a header replaces all values

- **WHEN** `headers["Accept"] = "text/html"` is called after existing values
- **THEN** all previous `Accept` header values are removed and replaced

#### Scenario: Bytes serialization produces HTTP header block

- **WHEN** `bytes(headers)` is called (or equivalent)
- **THEN** it produces `name: value\r\n` lines separated by `\r\n`, terminated with a final `\r\n`

#### Scenario: Header fields are bytes

- **WHEN** any value is set on Headers
- **THEN** it is stored as `Vec<u8>` (strings are converted via UTF-8)

### Requirement: Message base type

The system SHALL define a `Message` struct (base for Request and Response) with:

- `data: MessageData` — the underlying data struct
- `stream: StreamMode` — controls body streaming behavior:
  - `StreamMode::Buffered` — buffer entire body before forwarding
  - `StreamMode::Passthrough` — forward body immediately without buffering
  - `StreamMode::Transform(Box<dyn FnMut(&[u8]) -> Vec<u8>>)` — apply transformation function per chunk

`MessageData` contains:

- `http_version: Vec<u8>` — e.g., `b"HTTP/1.1"`
- `headers: Headers` — HTTP headers
- `content: Option<Vec<u8>>` — raw (potentially compressed) body bytes
- `trailers: Option<Headers>` — HTTP trailers
- `timestamp_start: f64` — when headers were received
- `timestamp_end: Option<f64>` — when last byte was received

#### Scenario: Message exposes http_version as string

- **WHEN** `message.http_version` is accessed
- **THEN** it returns a decoded `String` like `"HTTP/1.1"`

#### Scenario: Message content defaults to None

- **WHEN** a new Message is created without a body
- **THEN** `content` is `None`

### Requirement: Request type

The system SHALL define a `Request` struct (extends Message) with:

- `data: RequestData` — additional request-specific fields:
  - `host: String` — target server hostname
  - `port: u16` — target port
  - `method: Vec<u8>` — HTTP method (e.g., `b"GET"`)
  - `scheme: Vec<u8>` — URL scheme (e.g., `b"https"`)
  - `authority: Vec<u8>` — HTTP/2 `:authority` pseudo-header or absolute-form target
  - `path: Vec<u8>` — request path including query (e.g., `b"/api/v1?key=val"`)

Computed properties:

- `url: String` — full URL constructed from scheme+host+port+path
- `host_header: Option<String>` — HTTP/1 `Host` header or HTTP/2 `:authority`
- `pretty_host: String` — host derived from `Host` header (preferred over request line)
- `pretty_url: String` — full URL using `pretty_host`
- `query: QueryParams` — mutable view on URL query string
- `cookies: CookieParams` — mutable view on `Cookie` header
- `path_components: &[String]` — URL path segments
- `first_line_format: FirstLineFormat` — "authority", "absolute", or "relative"

#### Scenario: Setting url updates components

- **WHEN** `request.url = "https://example.com:8443/path?q=1"` is set
- **THEN** `scheme`, `host`, `port`, and `path` are all updated accordingly

#### Scenario: Setting host updates Host header

- **WHEN** `request.host = "newhost.com"` is set
- **THEN** the `Host` header and `authority` are updated to match

#### Scenario: Request method is case-normalized

- **WHEN** `request.method` is accessed
- **THEN** it returns an uppercase string (e.g., `"GET"`, not `"get"`)

#### Scenario: Request urlencoded_form parses POST body

- **WHEN** a POST request has `Content-Type: application/x-www-form-urlencoded`
- **THEN** `request.urlencoded_form` returns the parsed form fields as a MultiDictView

#### Scenario: Request cookies are parsed from header

- **WHEN** a request has `Cookie: name=value` header
- **THEN** `request.cookies` provides dictionary-like access to cookie name-value pairs

### Requirement: Response type

The system SHALL define a `Response` struct (extends Message) with:

- `data: ResponseData` — additional response-specific fields:
  - `status_code: u16` — HTTP status code (e.g., 200, 404, 500)
  - `reason: Vec<u8>` — reason phrase (e.g., `b"Not Found"`)

Computed properties:

- `cookies: ResponseCookies` — mutable view on `Set-Cookie` headers with attribute parsing

#### Scenario: Response status code is an integer

- **WHEN** `response.status_code` is accessed
- **THEN** it returns a `u16` value

#### Scenario: Response reason is ISO-8859-1 decoded

- **WHEN** `response.reason` is accessed as a String
- **THEN** the bytes are decoded using ISO-8859-1 encoding

#### Scenario: Response cookies include attributes

- **WHEN** a response has `Set-Cookie: id=abc; HttpOnly; Path=/`
- **THEN** `response.cookies["id"]` returns the value and attributes (HttpOnly, Path)

### Requirement: Message content management

The system SHALL provide content management methods on `Message`:

- `raw_content: Option<Vec<u8>>` — direct access to raw (compressed) body
- `content: Option<Vec<u8>>` — uncompressed body (decompressed based on `content-encoding`)
- `text: Option<String>` — decoded text body (UTF-8 or as specified by charset)
- `json<T: Deserialize>()` — parse body as JSON

Setting `content` SHALL:

- Store the value as `raw_content`
- Update `content-length` header automatically (unless `transfer-encoding` is present)
- Handle `content-encoding` header presence (identity, gzip, deflate, br, zstd)

Setting `text` SHALL:

- Encode the string to bytes
- Set `content-encoding` if needed based on content-type
- Fall back to UTF-8 and update `content-type` charset if inference fails

`decode()` SHALL:

- Decode the body based on current `content-encoding` header
- Remove the `content-encoding` header after decoding

`encode(encoding)` SHALL:

- Set `content-encoding` to the given encoding
- Re-encode `raw_content` with the new encoding

#### Scenario: Setting content updates content-length

- **WHEN** `message.content = b"hello"` is set
- **THEN** `headers["content-length"]` becomes `"5"`

#### Scenario: Content decoding respects content-encoding

- **WHEN** a response has `content-encoding: gzip` and compressed body
- **THEN** `message.content` returns the decompressed bytes

#### Scenario: Decode removes content-encoding header

- **WHEN** `message.decode()` is called on a gzipped response
- **THEN** `content-encoding` header is removed and body is decompressed

#### Scenario: JSON access on missing body raises

- **WHEN** `message.json::<serde_json::Value>()` is called on a message with no body
- **THEN** it returns an error

### Requirement: Message streaming

The system SHALL support three streaming modes via `Message.stream`:

- `Buffered` — entire body is buffered before forwarding (enables body transformation)
- `Passthrough` — body is forwarded immediately without buffering
- `Transform(f)` — body chunks pass through a user-provided transformation function

Streaming mode MUST be set in `requestheaders` or `responseheaders` hooks — setting it in `request` or `response` hooks is too late.

#### Scenario: Buffered mode captures entire body

- **WHEN** `stream = StreamMode::Buffered`
- **THEN** the proxy buffers the complete body before delivering to `request`/`response` hooks

#### Scenario: Passthrough mode does not buffer

- **WHEN** `stream = StreamMode::Passthrough`
- **THEN** body bytes are forwarded immediately as they arrive
