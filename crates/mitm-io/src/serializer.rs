//! FlowSerializer trait and JSON implementation.

use mitm_proxy::HTTPFlow;
use thiserror::Error;

/// Errors that can occur during serialization.
#[derive(Error, Debug)]
pub enum SerializationError {
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Serialization failed: {0}")]
    Serialization(String),
}

/// Trait for serializing and deserializing flows.
#[async_trait::async_trait]
pub trait FlowSerializer: Send + Sync {
    /// Serialize a flow to a string (typically JSON).
    async fn serialize(&self, flow: &HTTPFlow) -> Result<String, SerializationError>;

    /// Deserialize a flow from a string (typically JSON).
    async fn deserialize(&self, data: &str) -> Result<HTTPFlow, SerializationError>;
}

/// JSON flow serializer implementation.
#[derive(Clone)]
pub struct JsonFlowSerializer;

impl JsonFlowSerializer {
    /// Create a new JsonFlowSerializer.
    pub fn new() -> Self {
        Self
    }
}

impl Default for JsonFlowSerializer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl FlowSerializer for JsonFlowSerializer {
    async fn serialize(&self, flow: &HTTPFlow) -> Result<String, SerializationError> {
        let json = serde_json::to_string(flow)?;
        Ok(json)
    }

    async fn deserialize(&self, data: &str) -> Result<HTTPFlow, SerializationError> {
        let flow: HTTPFlow = serde_json::from_str(data)?;
        Ok(flow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mitm_core::connection::{Client, Server};
    use mitm_core::flow::FlowBase;
    use mitm_net::http::{Request, Response};

    fn test_flow() -> HTTPFlow {
        HTTPFlow {
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
    async fn test_json_serialize() {
        let serializer = JsonFlowSerializer::new();
        let flow = test_flow();
        let json = serializer.serialize(&flow).await.unwrap();
        assert!(!json.is_empty());
        assert!(json.contains("127.0.0.1"));
    }

    #[tokio::test]
    async fn test_json_deserialize() {
        let serializer = JsonFlowSerializer::new();
        let flow = test_flow();
        let json = serializer.serialize(&flow).await.unwrap();
        let deserialized = serializer.deserialize(&json).await.unwrap();
        assert_eq!(deserialized.base.client_conn.connection.peername, flow.base.client_conn.connection.peername);
    }

    #[tokio::test]
    async fn test_json_roundtrip() {
        let serializer = JsonFlowSerializer::new();
        let flow = test_flow();
        let json = serializer.serialize(&flow).await.unwrap();
        let deserialized = serializer.deserialize(&json).await.unwrap();
        // Verify basic fields are preserved
        assert_eq!(
            deserialized.request.data.http_version,
            flow.request.data.http_version
        );
    }
}
