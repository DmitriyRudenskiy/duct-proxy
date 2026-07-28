//! Flow base types: `FlowError`, `FlowBase`, and shared lifecycle helpers.
//!
//! The concrete `Flow` enum (Http/Tcp/Udp/Dns variants) lives in `mitm-proxy`
//! since it depends on types from other crates. This module provides the
//! shared state and the error type that all variants embed.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::connection::{Client, Server};

/// An error affecting a flow (connection interrupt, timeout, protocol error).
///
/// Distinct from an HTTP protocol error response (e.g., a 500 status code),
/// which is represented by a normal `Response` object.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FlowError {
    /// Human-readable error description.
    pub msg: String,
    /// Unix timestamp of when the error occurred.
    pub timestamp: f64,
}

impl FlowError {
    /// The error message used when a flow is explicitly killed.
    pub const KILLED_MESSAGE: &'static str = "Connection killed.";

    pub fn killed() -> Self {
        Self {
            msg: Self::KILLED_MESSAGE.to_string(),
            timestamp: current_timestamp(),
        }
    }

    pub fn new(msg: &str) -> Self {
        Self {
            msg: msg.to_string(),
            timestamp: current_timestamp(),
        }
    }
}

impl std::fmt::Display for FlowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

/// Shared base state for all flow types.
///
/// Each concrete flow variant (HTTPFlow, TCPFlow, etc.) embeds this as `base`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FlowBase {
    /// Unique identifier (UUID4).
    pub id: String,
    /// The client connection.
    pub client_conn: Client,
    /// The server connection.
    pub server_conn: Server,
    /// Connection or protocol error.
    pub error: Option<FlowError>,
    /// Whether the flow is currently paused (intercepted).
    pub intercepted: bool,
    /// User-set marker annotation (e.g., emoji).
    pub marked: String,
    /// Replay direction: `"request"` or `"response"`, or `None`.
    pub is_replay: Option<String>,
    /// Whether the flow belongs to an active connection.
    pub live: bool,
    /// Unix timestamp of flow creation.
    pub timestamp_created: f64,
    /// Arbitrary user metadata.
    pub metadata: HashMap<String, serde_json::Value>,
    /// User comment.
    pub comment: String,
}

impl FlowBase {
    /// Create a new FlowBase.
    pub fn new(client_conn: Client, server_conn: Server, live: bool) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            client_conn,
            server_conn,
            error: None,
            intercepted: false,
            marked: String::new(),
            is_replay: None,
            live,
            timestamp_created: current_timestamp(),
            metadata: HashMap::new(),
            comment: String::new(),
        }
    }

    /// Returns `true` if this flow is currently intercepted.
    pub fn is_intercepted(&self) -> bool {
        self.intercepted
    }

    /// Returns `true` if the error is the kill message.
    pub fn is_killed(&self) -> bool {
        self.error
            .as_ref()
            .map(|e| e.msg == FlowError::KILLED_MESSAGE)
            .unwrap_or(false)
    }

    /// Kill this flow variant. Sets error, clears intercepted, sets live=false.
    pub fn do_kill(&mut self) -> Result<(), &'static str> {
        if self.live && !self.is_killed() {
            self.error = Some(FlowError::killed());
            self.intercepted = false;
            self.live = false;
            Ok(())
        } else {
            Err("Flow is not killable.")
        }
    }

    /// Intercept this flow. Idempotent.
    pub fn do_intercept(&mut self) {
        self.intercepted = true;
    }

    /// Resume this flow. Idempotent.
    pub fn do_resume(&mut self) {
        self.intercepted = false;
    }

    /// Deep clone with a new ID and `live = false`.
    pub fn do_copy(&self) -> Self {
        let mut cloned = self.clone();
        cloned.live = false;
        cloned.id = uuid::Uuid::new_v4().to_string();
        cloned
    }

    /// Update the ID.
    pub fn set_id(&mut self, id: String) {
        self.id = id;
    }

    /// Update the live flag.
    pub fn set_live(&mut self, live: bool) {
        self.live = live;
    }
}

/// Returns the current Unix timestamp in seconds.
pub fn current_timestamp() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

/// Create a default Client for constructing flows.
pub fn default_client() -> Client {
    Client::new(("127.0.0.1".to_string(), 0), ("0.0.0.0".to_string(), 0), "regular")
}

/// Create a default Server for constructing flows.
pub fn default_server() -> Server {
    Server::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flow_error_killed() {
        let err = FlowError::killed();
        assert_eq!(err.msg, FlowError::KILLED_MESSAGE);
        assert!(err.timestamp > 0.0);
    }

    #[test]
    fn test_flow_base_default() {
        let base = FlowBase::new(default_client(), default_server(), true);
        assert!(base.live);
        assert!(!base.intercepted);
        assert!(base.error.is_none());
        assert!(!base.is_killed());
    }

    #[test]
    fn test_flow_base_kill() {
        let mut base = FlowBase::new(default_client(), default_server(), true);
        base.do_kill().unwrap();
        assert!(base.is_killed());
        assert!(!base.live);
        assert!(!base.intercepted);
        // Cannot kill again
        assert!(base.do_kill().is_err());
    }

    #[test]
    fn test_flow_base_intercept_resume() {
        let mut base = FlowBase::new(default_client(), default_server(), true);
        base.do_intercept();
        assert!(base.is_intercepted());
        base.do_intercept(); // idempotent
        base.do_resume();
        assert!(!base.is_intercepted());
        base.do_resume(); // idempotent
    }

    #[test]
    fn test_flow_base_copy() {
        let base = FlowBase::new(default_client(), default_server(), true);
        let id = base.id.clone();
        let copy = base.do_copy();
        assert_ne!(copy.id, id);
        assert!(!copy.live);
    }

    #[test]
    fn test_flow_base_serialization() {
        let base = FlowBase::new(default_client(), default_server(), true);
        let json = serde_json::to_string(&base).unwrap();
        let deserialized: FlowBase = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, base.id);
        assert_eq!(deserialized.live, base.live);
    }
}
