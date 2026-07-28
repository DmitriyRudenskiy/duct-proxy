//! WebSocket session types: `WebSocketOpcode`, `WebSocketMessage`, `WebSocketData`.

use serde::{Deserialize, Serialize};

/// WebSocket frame opcode (RFC 6455).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WebSocketOpcode {
    /// Text frame (opcode 1).
    TEXT = 1,
    /// Binary frame (opcode 2).
    BINARY = 2,
    /// Connection close (opcode 8).
    CLOSE = 8,
    /// Ping (opcode 9).
    PING = 9,
    /// Pong (opcode 10).
    PONG = 10,
}

impl WebSocketOpcode {
    /// Create from raw opcode value.
    pub fn from_u8(val: u8) -> Self {
        match val {
            1 => Self::TEXT,
            2 => Self::BINARY,
            8 => Self::CLOSE,
            9 => Self::PING,
            10 => Self::PONG,
            _ => panic!("unknown WebSocket opcode: {}", val),
        }
    }

    /// Raw opcode value.
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// A single WebSocket message.
///
/// Fragmented messages are reassembled into a single instance.
/// Content is always bytes (even for text frames) to avoid type confusion.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WebSocketMessage {
    /// `true` if sent by the client.
    pub from_client: bool,
    /// Message opcode.
    pub msg_type: WebSocketOpcode,
    /// Message payload (always bytes).
    pub content: Vec<u8>,
    /// Unix timestamp.
    pub timestamp: f64,
    /// `true` if the message was dropped (not forwarded).
    pub dropped: bool,
    /// `true` if the message was injected by a hook.
    pub injected: bool,
}

impl WebSocketMessage {
    /// Create a new WebSocketMessage.
    pub fn new(
        msg_type: WebSocketOpcode,
        from_client: bool,
        content: Vec<u8>,
    ) -> Self {
        Self {
            from_client,
            msg_type,
            content,
            timestamp: mitm_core::flow::current_timestamp(),
            dropped: false,
            injected: false,
        }
    }

    /// Returns `true` if this is a text message.
    pub fn is_text(&self) -> bool {
        self.msg_type == WebSocketOpcode::TEXT
    }

    /// Returns the message content as a UTF-8 string.
    ///
    /// Only valid for TEXT messages.
    pub fn text(&self) -> Result<String, &'static str> {
        if self.msg_type != WebSocketOpcode::TEXT {
            return Err("not a TEXT message");
        }
        String::from_utf8(self.content.clone()).map_err(|_| "invalid UTF-8 in text message")
    }

    /// Set content as text (encodes to UTF-8 bytes).
    pub fn set_text(&mut self, text: &str) -> Result<(), &'static str> {
        if self.msg_type != WebSocketOpcode::TEXT {
            return Err("not a TEXT message");
        }
        self.content = text.as_bytes().to_vec();
        Ok(())
    }

    /// Mark this message as dropped (not forwarded to the other peer).
    pub fn drop_msg(&mut self) {
        self.dropped = true;
    }

    /// Deprecated alias for `drop_msg()`.
    #[deprecated(note = "use drop_msg() instead")]
    pub fn kill(&mut self) {
        self.dropped = true;
    }
}

/// Container for all WebSocket data in a session.
///
/// Attached to `HTTPFlow` via the `websocket` field — only present for
/// WebSocket upgrade connections.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WebSocketData {
    /// All WebSocket messages in the session.
    pub messages: Vec<WebSocketMessage>,
    /// Who closed the connection: `true` if client, `false` if server, `None` if active.
    pub closed_by_client: Option<bool>,
    /// WebSocket close code (RFC 6455 §7.1.5).
    pub close_code: Option<u16>,
    /// WebSocket close reason text.
    pub close_reason: Option<String>,
    /// Unix timestamp of connection close.
    pub timestamp_end: Option<f64>,
}

impl WebSocketData {
    /// Create empty WebSocketData.
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_message_text() {
        let msg = WebSocketMessage::new(
            WebSocketOpcode::TEXT,
            true,
            b"hello".to_vec(),
        );
        assert!(msg.is_text());
        assert_eq!(msg.text().unwrap(), "hello");
    }

    #[test]
    fn test_websocket_message_binary_no_text() {
        let msg = WebSocketMessage::new(
            WebSocketOpcode::BINARY,
            false,
            vec![0xDE, 0xAD],
        );
        assert!(!msg.is_text());
        assert!(msg.text().is_err());
    }

    #[test]
    fn test_websocket_message_drop() {
        let mut msg = WebSocketMessage::new(
            WebSocketOpcode::TEXT,
            true,
            b"test".to_vec(),
        );
        assert!(!msg.dropped);
        msg.drop_msg();
        assert!(msg.dropped);
    }

    #[test]
    fn test_websocket_data_roundtrip() {
        let mut ws = WebSocketData::new();
        ws.messages.push(WebSocketMessage::new(
            WebSocketOpcode::TEXT,
            true,
            b"hello".to_vec(),
        ));
        ws.closed_by_client = Some(true);
        ws.close_code = Some(1000);
        ws.close_reason = Some("normal closure".to_string());

        let json = serde_json::to_string(&ws).unwrap();
        let deserialized: WebSocketData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.messages.len(), 1);
        assert_eq!(deserialized.closed_by_client, Some(true));
        assert_eq!(deserialized.close_code, Some(1000));
    }

    #[test]
    fn test_opcode_values() {
        assert_eq!(WebSocketOpcode::TEXT.as_u8(), 1);
        assert_eq!(WebSocketOpcode::BINARY.as_u8(), 2);
        assert_eq!(WebSocketOpcode::CLOSE.as_u8(), 8);
        assert_eq!(WebSocketOpcode::PING.as_u8(), 9);
        assert_eq!(WebSocketOpcode::PONG.as_u8(), 10);
    }
}
