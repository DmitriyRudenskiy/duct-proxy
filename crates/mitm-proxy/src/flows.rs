//! Concrete flow types: `Flow` enum and its variants.
//!
//! The `Flow` enum is the top-level abstraction. Every intercepted network
//! transaction is one of `Http`, `Tcp`, `Udp`, or `Dns`.

use serde::{Deserialize, Serialize};

use mitm_core::flow::FlowBase;
use mitm_core::connection::{Client, Server};
use mitm_net::http::{Request, Response};

/// HTTP flow: request + optional response, optional WebSocket data.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HTTPFlow {
    #[serde(flatten)]
    pub base: FlowBase,
    pub request: Request,
    pub response: Option<Response>,
    pub websocket: Option<super::websocket::WebSocketData>,
}

/// TCP flow: ordered list of stream messages.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TCPFlow {
    #[serde(flatten)]
    pub base: FlowBase,
    pub messages: Vec<super::stream::TCPMessage>,
}

/// UDP flow: ordered list of datagram messages.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UDPFlow {
    #[serde(flatten)]
    pub base: FlowBase,
    pub messages: Vec<super::stream::UDPMessage>,
}

/// DNS flow: request + optional response.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DNSFlow {
    #[serde(flatten)]
    pub base: FlowBase,
    pub request: super::dns::DNSMessage,
    pub response: Option<super::dns::DNSMessage>,
}

/// The top-level flow enum.
///
/// Every intercepted network transaction is represented as one variant.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[allow(clippy::large_enum_variant)]
pub enum Flow {
    /// HTTP request/response transaction.
    #[serde(rename = "http")]
    Http(HTTPFlow),
    /// Raw TCP stream session.
    #[serde(rename = "tcp")]
    Tcp(TCPFlow),
    /// UDP datagram session.
    #[serde(rename = "udp")]
    Udp(UDPFlow),
    /// DNS query/response.
    #[serde(rename = "dns")]
    Dns(DNSFlow),
}

impl Flow {
    /// Returns the shared base (helper to avoid repeated match).
    fn base_ref(&self) -> &FlowBase {
        match self {
            Flow::Http(f) => &f.base,
            Flow::Tcp(f) => &f.base,
            Flow::Udp(f) => &f.base,
            Flow::Dns(f) => &f.base,
        }
    }

    fn base_mut_ref(&mut self) -> &mut FlowBase {
        match self {
            Flow::Http(f) => &mut f.base,
            Flow::Tcp(f) => &mut f.base,
            Flow::Udp(f) => &mut f.base,
            Flow::Dns(f) => &mut f.base,
        }
    }

    /// Returns `true` if this flow can be killed.
    pub fn killable(&self) -> bool {
        let b = self.base_ref();
        b.live && !b.is_killed()
    }

    /// Kill this flow. The current request/response will not be forwarded.
    pub fn kill(&mut self) -> Result<(), &'static str> {
        self.base_mut_ref().do_kill()
    }

    /// Intercept this flow — pauses processing. Idempotent.
    pub fn intercept(&mut self) {
        self.base_mut_ref().do_intercept();
    }

    /// Resume this flow. Idempotent.
    pub fn resume(&mut self) {
        self.base_mut_ref().do_resume();
    }

    /// Returns `true` if this flow is currently intercepted.
    pub fn is_intercepted(&self) -> bool {
        self.base_ref().is_intercepted()
    }

    /// Returns `true` if this flow has been killed.
    pub fn is_killed(&self) -> bool {
        self.base_ref().is_killed()
    }

    /// Deep copy with new ID and `live = false`.
    pub fn copy(&self) -> Self {
        match self {
            Flow::Http(f) => Flow::Http(HTTPFlow {
                base: f.base.do_copy(),
                request: f.request.clone(),
                response: f.response.clone(),
                websocket: f.websocket.clone(),
            }),
            Flow::Tcp(f) => Flow::Tcp(TCPFlow {
                base: f.base.do_copy(),
                messages: f.messages.clone(),
            }),
            Flow::Udp(f) => Flow::Udp(UDPFlow {
                base: f.base.do_copy(),
                messages: f.messages.clone(),
            }),
            Flow::Dns(f) => Flow::Dns(DNSFlow {
                base: f.base.do_copy(),
                request: f.request.clone(),
                response: f.response.clone(),
            }),
        }
    }

    /// Access the shared base.
    pub fn base(&self) -> &FlowBase {
        self.base_ref()
    }

    /// Mutable access to the shared base.
    pub fn base_mut(&mut self) -> &mut FlowBase {
        self.base_mut_ref()
    }

    /// Create an HTTPFlow.
    pub fn http(
        client: Client,
        server: Server,
        request: Request,
    ) -> Self {
        Self::Http(HTTPFlow {
            base: FlowBase::new(client, server, true),
            request,
            response: None,
            websocket: None,
        })
    }

    /// Create a TCPFlow.
    pub fn tcp(
        client: Client,
        server: Server,
    ) -> Self {
        Self::Tcp(TCPFlow {
            base: FlowBase::new(client, server, true),
            messages: Vec::new(),
        })
    }

    /// Create a UDPFlow.
    pub fn udp(
        client: Client,
        server: Server,
    ) -> Self {
        Self::Udp(UDPFlow {
            base: FlowBase::new(client, server, true),
            messages: Vec::new(),
        })
    }

    /// Create a DNSFlow.
    pub fn dns(
        client: Client,
        server: Server,
        request: super::dns::DNSMessage,
    ) -> Self {
        Self::Dns(DNSFlow {
            base: FlowBase::new(client, server, true),
            request,
            response: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mitm_core::flow::{default_client, default_server};
    use mitm_net::http::Request;

    #[test]
    fn test_create_http_flow() {
        let client = default_client();
        let server = default_server();
        let request = Request::make("GET", "http://example.com/", None, &[]);
        let flow = Flow::http(client, server, request);
        assert!(flow.base().live);
        assert!(!flow.base().intercepted);
        assert!(flow.killable());
    }

    #[test]
    fn test_flow_intercept_resume() {
        let client = default_client();
        let server = default_server();
        let request = Request::make("GET", "http://example.com/", None, &[]);
        let mut flow = Flow::http(client, server, request);
        flow.intercept();
        assert!(flow.is_intercepted());
        flow.intercept(); // idempotent
        flow.resume();
        assert!(!flow.is_intercepted());
    }

    #[test]
    fn test_flow_kill() {
        let client = default_client();
        let server = default_server();
        let request = Request::make("GET", "http://example.com/", None, &[]);
        let mut flow = Flow::http(client, server, request);
        flow.kill().unwrap();
        assert!(!flow.killable());
        assert!(flow.is_killed());
        assert!(!flow.base().live);
    }

    #[test]
    fn test_flow_copy() {
        let client = default_client();
        let server = default_server();
        let request = Request::make("GET", "http://example.com/", None, &[]);
        let flow = Flow::http(client, server, request);
        let id = flow.base().id.clone();
        let copy = flow.copy();
        assert_ne!(copy.base().id, id);
        assert!(!copy.base().live);
    }

    #[test]
    fn test_flow_serialization_all_variants() {
        let client = default_client();
        let server = default_server();

        // HTTP
        let http = Flow::http(
            client.clone(),
            server.clone(),
            Request::make("GET", "http://example.com/", None, &[]),
        );
        let json = serde_json::to_string(&http).unwrap();
        assert!(json.contains("\"type\":\"http\""));

        // TCP
        let tcp = Flow::tcp(client.clone(), server.clone());
        let json = serde_json::to_string(&tcp).unwrap();
        assert!(json.contains("\"type\":\"tcp\""));

        // UDP
        let udp = Flow::udp(client.clone(), server.clone());
        let json = serde_json::to_string(&udp).unwrap();
        assert!(json.contains("\"type\":\"udp\""));

        // DNS
        use crate::dns::{DNSMessage, DnsClass, DnsType, Question, Rcode};
        let dns_req = DNSMessage {
            id: 1,
            query: true,
            op_code: 0,
            authoritative_answer: false,
            truncation: false,
            recursion_desired: true,
            recursion_available: false,
            reserved: 0,
            response_code: Rcode::NOERROR,
            questions: vec![Question {
                name: "example.com".to_string(),
                type_: DnsType::A,
                class_: DnsClass::IN,
            }],
            answers: Vec::new(),
            authorities: Vec::new(),
            additionals: Vec::new(),
            timestamp: None,
        };
        let dns = Flow::dns(client, server, dns_req);
        let json = serde_json::to_string(&dns).unwrap();
        assert!(json.contains("\"type\":\"dns\""));
    }

    #[test]
    fn test_flow_deserialization_roundtrip() {
        let client = default_client();
        let server = default_server();
        let request = Request::make("POST", "http://example.com/api", Some(b"body"), &[
            ("Content-Type".to_string(), "application/json".to_string()),
        ]);
        let flow = Flow::http(client, server, request);
        let json = serde_json::to_string(&flow).unwrap();
        let deserialized: Flow = serde_json::from_str(&json).unwrap();

        match deserialized {
            Flow::Http(f) => {
                assert_eq!(f.request.method(), "POST");
                assert_eq!(f.request.host(), "example.com");
            }
            _ => panic!("Expected Http variant, got {:?}", deserialized),
        }
    }
}
