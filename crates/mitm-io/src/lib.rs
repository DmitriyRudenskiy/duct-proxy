//! mitm-io: I/O utilities for mitmproxy-rs.
//!
//! This crate provides:
//! - **FlowSerializer**: Serialize/deserialize HTTPFlow to/from JSON
//! - **FlowWriter**: Append flows to gzip-compressed JSONL files (.jsonl.gz)
//! - **FlowReader**: Read flows from gzip-compressed JSONL files
//! - **HarExporter**: Export flows to HAR 1.2 format (optional)

pub mod serializer;
pub mod writer;
pub mod reader;
pub mod har;

// Re-exports.
pub use serializer::{FlowSerializer, JsonFlowSerializer};
pub use writer::FlowWriter;
pub use reader::FlowReader;
pub use har::HarExporter;
