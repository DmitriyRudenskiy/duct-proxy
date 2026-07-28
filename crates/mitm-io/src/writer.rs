//! FlowWriter for appending flows to gzip-compressed JSONL files.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use flate2::write::GzEncoder;
use flate2::Compression;
use thiserror::Error;

use crate::serializer::{FlowSerializer, SerializationError};

/// Errors that can occur during writing.
#[derive(Error, Debug)]
pub enum WriteError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] SerializationError),
}

/// Writer for appending flows to a gzip-compressed JSONL file.
pub struct FlowWriter<S: FlowSerializer> {
    inner: Option<BufWriter<GzEncoder<File>>>,
    serializer: S,
}

impl<S: FlowSerializer> FlowWriter<S> {
    /// Create a new FlowWriter that writes to the given path.
    pub fn new(path: &Path, serializer: S) -> Result<Self, WriteError> {
        let file = File::create(path)?;
        let encoder = GzEncoder::new(file, Compression::default());
        let writer = BufWriter::new(encoder);
        Ok(Self {
            inner: Some(writer),
            serializer,
        })
    }

    /// Write a flow to the file.
    pub async fn write(&mut self, flow: &mitm_proxy::HTTPFlow) -> Result<(), WriteError> {
        let json = self.serializer.serialize(flow).await?;
        if let Some(ref mut writer) = self.inner {
            writeln!(writer, "{}", json)?;
            writer.flush()?;
        }
        Ok(())
    }

    /// Close the writer, finalizing the gzip stream.
    pub fn close(&mut self) -> Result<(), WriteError> {
        if let Some(mut writer) = self.inner.take() {
            writer.flush()?;
            // GzEncoder will be dropped here, finalizing the gzip stream
        }
        Ok(())
    }
}

impl<S: FlowSerializer> Drop for FlowWriter<S> {
    fn drop(&mut self) {
        // Close is called implicitly when inner is dropped
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mitm_core::connection::{Client, Server};
    use mitm_core::flow::FlowBase;
    use mitm_net::http::{Request, Response};
    use tempfile::NamedTempFile;

    fn test_flow() -> mitm_proxy::HTTPFlow {
        mitm_proxy::HTTPFlow {
            base: FlowBase::new(
                Client::new(
                    ("127.0.0.1".to_string(), 12345),
                    ("127.0.0.1".to_string(), 80),
                    "regular",
                ),
                Server::new(),
                true,
            ),
            request: Request::new(),
            response: Some(Response::new()),
            websocket: None,
        }
    }

    #[tokio::test]
    async fn test_flow_writer_create() {
        let tmp_file = NamedTempFile::new().unwrap();
        let serializer = JsonFlowSerializer::new();
        let mut writer = FlowWriter::new(tmp_file.path(), serializer).unwrap();
        writer.close().unwrap();
    }

    #[tokio::test]
    async fn test_flow_writer_write() {
        let tmp_file = NamedTempFile::new().unwrap();
        let serializer = JsonFlowSerializer::new();
        let mut writer = FlowWriter::new(tmp_file.path(), serializer).unwrap();
        let flow = test_flow();
        writer.write(&flow).await.unwrap();
        writer.close().unwrap();
        // Verify file exists and has content
        let metadata = std::fs::metadata(tmp_file.path()).unwrap();
        assert!(metadata.len() > 0);
    }

    #[tokio::test]
    async fn test_flow_writer_gzip_compression() {
        let tmp_file = NamedTempFile::new().unwrap();
        let serializer = JsonFlowSerializer::new();
        let mut writer = FlowWriter::new(tmp_file.path(), serializer).unwrap();
        // Write multiple flows
        for _ in 0..10 {
            let flow = test_flow();
            writer.write(&flow).await.unwrap();
        }
        writer.close().unwrap();
        // Verify file is gzip compressed (check magic bytes)
        let bytes = std::fs::read(tmp_file.path()).unwrap();
        assert_eq!(bytes[0], 0x1f); // gzip magic number
        assert_eq!(bytes[1], 0x8b); // gzip magic number
    }
}
