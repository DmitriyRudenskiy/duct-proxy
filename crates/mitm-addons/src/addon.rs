//! Addon trait definition with lifecycle hooks.

use mitm_core::FlowBase;
use thiserror::Error;

/// Errors that can occur during addon execution.
#[derive(Error, Debug)]
pub enum AddonError {
    #[error("Addon execution failed: {0}")]
    Execution(String),

    #[error("Addon timeout after {0:?}")]
    Timeout(std::time::Duration),

    #[error("Flow not found")]
    FlowNotFound,
}

/// Trait for mitm-proxy addons.
///
/// Addons implement lifecycle hooks to modify or observe HTTP, TCP, UDP, and DNS flows.
/// All methods have default no-op implementations - only override the hooks you need.
#[async_trait::async_trait]
pub trait Addon: Send + Sync {
    /// Called before request headers are sent to upstream.
    async fn requestheaders(&mut self, _flow: &mut FlowBase) -> Result<(), AddonError> {
        Ok(())
    }

    /// Called after request body is complete.
    async fn request(&mut self, _flow: &mut FlowBase) -> Result<(), AddonError> {
        Ok(())
    }

    /// Called before response headers are sent to client.
    async fn responseheaders(&mut self, _flow: &mut FlowBase) -> Result<(), AddonError> {
        Ok(())
    }

    /// Called after response body is complete.
    async fn response(&mut self, _flow: &mut FlowBase) -> Result<(), AddonError> {
        Ok(())
    }

    /// Called when an error occurs during flow processing.
    async fn error(&mut self, _error: &AddonError) -> Result<(), AddonError> {
        Ok(())
    }

    /// Called for each TCP message in a TCP flow.
    async fn tcp_message(&mut self, _flow: &mut FlowBase, _message: &[u8]) -> Result<(), AddonError> {
        Ok(())
    }

    /// Called for each UDP message in a UDP flow.
    async fn udp_message(&mut self, _flow: &mut FlowBase, _message: &[u8]) -> Result<(), AddonError> {
        Ok(())
    }

    /// Called for DNS query messages.
    async fn dns_request(&mut self, _flow: &mut FlowBase, _message: &[u8]) -> Result<(), AddonError> {
        Ok(())
    }

    /// Called for DNS response messages.
    async fn dns_response(&mut self, _flow: &mut FlowBase, _message: &[u8]) -> Result<(), AddonError> {
        Ok(())
    }
}
