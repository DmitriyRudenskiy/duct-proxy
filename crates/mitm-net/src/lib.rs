//! mitm-net: network protocol types for the mitmproxy-rs data model.
//!
//! This crate provides HTTP message types used by the proxy layer.

pub mod http;
pub mod url;
pub mod cookie;
pub mod form;
pub mod chunked;
pub mod parser;
pub mod http2;

// Re-exports.
pub use http::{Message, MessageData, Request, Response, StreamMode};
pub use url::{UrlComponents, UrlParser, UrlParseError};
pub use cookie::{Cookie, CookieParser, CookieParseError, SameSite};
pub use form::{FormFields, FormField, FormParser, FormParseError};
pub use chunked::{decode as decode_chunked, encode as encode_chunked, ChunkedError};
pub use parser::{HttpRequest, HttpResponse, HttpRequestParser, HttpParseError, RequestLine, ResponseLine};
pub use http2::{Http2Error, has_http2_prior_knowledge, detect_http2};
