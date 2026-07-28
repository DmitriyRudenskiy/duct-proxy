//! FlowReader for reading flows from gzip-compressed JSONL files.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use flate2::read::GzDecoder;
use thiserror::Error;

use crate::serializer::{FlowSerializer, SerializationError};

/// Errors that can occur during reading.
#[derive(Error, Debug)]
pub enum ReadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] SerializationError),

    #[error("End of stream")]
    EndOfStream,
}

/// Reader for reading flows from a gzip-compressed JSONL file.
pub struct FlowReader<S: FlowSerializer> {
    inner: Option<BufReader<GzDecoder<File>>>,
    serializer: S,
}

impl<S: FlowSerializer> FlowReader<S> {
    /// Create a new FlowReader that reads from the given path.
    pub fn new(path: &Path, serializer: S) -> Result<Self, ReadError> {
        let file = File::open(path)?;
        let decoder = GzDecoder::new(file);
        let reader = BufReader::new(decoder);
        Ok(Self {
            inner: Some(reader),
            serializer,
        })
    }

    /// Read the next flow from the file.
    pub async fn read_next(&mut self) -> Result<Option<mitm_proxy::HTTPFlow>, ReadError> {
        if let Some(ref mut reader) = self.inner {
            // Read lines, skipping empty ones
            loop {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => {
                        // End of stream
                        return Ok(None);
                    }
                    Ok(_) => {
                        let line = line.trim();
                        if line.is_empty() {
                            // Skip empty lines
                            continue;
                        }
                        let flow = self.serializer.deserialize(line).await?;
                        return Ok(Some(flow));
                    }
                    Err(e) => {
                        // Handle UnexpectedEof gracefully
                        if e.kind() == std::io::ErrorKind::UnexpectedEof {
                            return Ok(None);
                        }
                        return Err(ReadError::Io(e));
                    }
                }
            }
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::writer::FlowWriter;
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
    async fn test_flow_reader_read() {
        let tmp_file = NamedTempFile::new().unwrap();
        let serializer = JsonFlowSerializer::new();
        
        // Write a flow first
        {
            let mut writer = FlowWriter::new(tmp_file.path(), serializer.clone()).unwrap();
            let flow = test_flow();
            writer.write(&flow).await.unwrap();
            writer.close().unwrap();
        }
        
        // Now read it back
        let mut reader = FlowReader::new(tmp_file.path(), serializer).unwrap();
        let flow = reader.read_next().await.unwrap();
        assert!(flow.is_some());
        let flow = flow.unwrap();
        assert_eq!(
            flow.request.data.http_version,
            test_flow().request.data.http_version
        );
    }

    #[tokio::test]
    async fn test_flow_reader_empty_file() {
        let tmp_file = NamedTempFile::new().unwrap();
        let serializer = JsonFlowSerializer::new();
        let mut reader = FlowReader::new(tmp_file.path(), serializer).unwrap();
        let flow = reader.read_next().await.unwrap();
        assert!(flow.is_none());
    }

    #[tokio::test]
    async fn test_flow_reader_multiple_flows() {
        let tmp_file = NamedTempFile::new().unwrap();
        let serializer = JsonFlowSerializer::new();
        
        // Write multiple flows
        {
            let mut writer = FlowWriter::new(tmp_file.path(), serializer.clone()).unwrap();
            for _ in 0..3 {
                let flow = test_flow();
                writer.write(&flow).await.unwrap();
            }
            writer.close().unwrap();
        }
        
        // Read them back
        let mut reader = FlowReader::new(tmp_file.path(), serializer).unwrap();
        let mut count = 0;
        while let Some(_) = reader.read_next().await.unwrap() {
            count += 1;
        }
        assert_eq!(count, 3);
    }
}
