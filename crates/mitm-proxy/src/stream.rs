//! TCP and UDP stream message types.

use serde::{Deserialize, Serialize};

/// A single TCP "message" (chunk of the byte stream).
///
/// Note: TCP is stream-based, not message-based. Message boundaries are
/// conceptual chunks, not protocol-enforced.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TCPMessage {
    /// `true` if sent by the client, `false` if sent by the server.
    pub from_client: bool,
    /// The message payload bytes.
    pub content: Vec<u8>,
    /// Unix timestamp of when the message was sent/received.
    pub timestamp: f64,
}

impl TCPMessage {
    /// Create a new TCPMessage.
    pub fn new(from_client: bool, content: Vec<u8>) -> Self {
        Self {
            from_client,
            content,
            timestamp: mitm_core::flow::current_timestamp(),
        }
    }
}

/// A single UDP datagram.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UDPMessage {
    /// `true` if sent by the client, `false` if sent by the server.
    pub from_client: bool,
    /// The datagram payload bytes.
    pub content: Vec<u8>,
    /// Unix timestamp of when the message was sent/received.
    pub timestamp: f64,
}

impl UDPMessage {
    /// Create a new UDPMessage.
    pub fn new(from_client: bool, content: Vec<u8>) -> Self {
        Self {
            from_client,
            content,
            timestamp: mitm_core::flow::current_timestamp(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_message_creation() {
        let msg = TCPMessage::new(true, b"hello".to_vec());
        assert!(msg.from_client);
        assert_eq!(msg.content, b"hello");
        assert!(msg.timestamp > 0.0);
    }

    #[test]
    fn test_udp_message_creation() {
        let msg = UDPMessage::new(false, b"response".to_vec());
        assert!(!msg.from_client);
        assert_eq!(msg.content, b"response");
    }

    #[test]
    fn test_tcp_message_serialization() {
        let msg = TCPMessage::new(true, b"data".to_vec());
        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: TCPMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.from_client, msg.from_client);
        assert_eq!(deserialized.content, msg.content);
    }
}
