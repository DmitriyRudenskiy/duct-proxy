//! HAR 1.2 export for mitmproxy-rs.

use mitm_proxy::HTTPFlow;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during HAR export.
#[derive(Error, Debug)]
pub enum HarExportError {
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Export failed: {0}")]
    Export(String),
}

/// HAR 1.2 Log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarEntry {
    #[serde(rename = "startedDateTime")]
    pub started_date_time: String,
    pub request: HarRequest,
    pub response: HarResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timings: Option<HarTimings>,
}

/// HAR 1.2 request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarRequest {
    pub method: String,
    pub url: String,
    #[serde(rename = "httpVersion")]
    pub http_version: String,
    #[serde(rename = "headers", default)]
    pub headers: Vec<HarHeader>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_string: Option<Vec<HarNameValuePair>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_size: Option<u64>,
}

/// HAR 1.2 response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarResponse {
    pub status: u16,
    #[serde(rename = "statusText")]
    pub status_text: String,
    #[serde(rename = "httpVersion")]
    pub http_version: String,
    #[serde(rename = "headers", default)]
    pub headers: Vec<HarHeader>,
    #[serde(rename = "content")]
    pub content: HarContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_size: Option<u64>,
}

/// HAR 1.2 content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarContent {
    pub size: u64,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

/// HAR 1.2 header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarHeader {
    pub name: String,
    pub value: String,
}

/// HAR 1.2 name-value pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarNameValuePair {
    pub name: String,
    pub value: String,
}

/// HAR 1.2 timings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarTimings {
    pub send: i64,
    pub wait: i64,
    pub receive: i64,
}

/// HAR 1.2 log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarLog {
    pub version: String,
    pub creator: HarCreator,
    #[serde(rename = "pages", default)]
    pub pages: Vec<HarPage>,
    #[serde(rename = "entries", default)]
    pub entries: Vec<HarEntry>,
}

/// HAR 1.2 creator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarCreator {
    pub name: String,
    pub version: String,
}

/// HAR 1.2 page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarPage {
    pub started_date_time: String,
    pub title: String,
    pub page_ref: String,
}

/// Exporter for converting flows to HAR 1.2 format.
pub struct HarExporter {
    log: HarLog,
}

impl HarExporter {
    /// Create a new HarExporter.
    pub fn new() -> Self {
        Self {
            log: HarLog {
                version: "1.2".to_string(),
                creator: HarCreator {
                    name: "mitmproxy-rs".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                },
                pages: Vec::new(),
                entries: Vec::new(),
            },
        }
    }

    /// Add a flow entry to the HAR log.
    pub fn add_entry(&mut self, flow: &HTTPFlow) {
        let scheme = String::from_utf8_lossy(&flow.request.scheme);
        let host = flow.request.host.clone();
        let path = String::from_utf8_lossy(&flow.request.path);
        let method = String::from_utf8_lossy(&flow.request.method);
        
        let url = format!("{}://{}{}", scheme, host, path);
        
        let entry = HarEntry {
            started_date_time: flow.base.timestamp_created.to_string(),
            request: HarRequest {
                method: method.to_string(),
                url: url.clone(),
                http_version: String::from_utf8_lossy(&flow.request.data.http_version).to_string(),
                headers: flow.request.data.headers.iter()
                    .map(|(k, v)| HarHeader {
                        name: String::from_utf8_lossy(k).to_string(),
                        value: String::from_utf8_lossy(v).to_string(),
                    })
                    .collect(),
                query_string: None,
                body_size: flow.request.data.content.as_ref().map(|c| c.len() as u64),
            },
            response: flow.response.as_ref().map(|r| HarResponse {
                status: r.status_code,
                status_text: String::from_utf8_lossy(&r.reason).to_string(),
                http_version: String::from_utf8_lossy(&r.data.http_version).to_string(),
                headers: r.data.headers.iter()
                    .map(|(k, v)| HarHeader {
                        name: String::from_utf8_lossy(k).to_string(),
                        value: String::from_utf8_lossy(v).to_string(),
                    })
                    .collect(),
                content: HarContent {
                    size: r.data.content.as_ref().map(|c| c.len() as u64).unwrap_or(0),
                    mime_type: r.data.headers.get("content-type").unwrap_or_default(),
                    text: r.data.content.as_ref().map(|c| String::from_utf8_lossy(c).to_string()),
                },
                redirect_url: None,
                body_size: r.data.content.as_ref().map(|c| c.len() as u64),
            }).unwrap_or(HarResponse {
                status: 0,
                status_text: "N/A".to_string(),
                http_version: "HTTP/1.1".to_string(),
                headers: Vec::new(),
                content: HarContent {
                    size: 0,
                    mime_type: String::new(),
                    text: None,
                },
                redirect_url: None,
                body_size: None,
            }),
            timings: None,
        };
        self.log.entries.push(entry);
    }

    /// Export the HAR log to a JSON string.
    pub fn export(&self) -> Result<String, HarExportError> {
        let json = serde_json::to_string_pretty(&self.log)?;
        Ok(json)
    }
}

impl Default for HarExporter {
    fn default() -> Self {
        Self::new()
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

    #[test]
    fn test_har_exporter_create() {
        let exporter = HarExporter::new();
        assert_eq!(exporter.log.version, "1.2");
        assert_eq!(exporter.log.entries.len(), 0);
    }

    #[test]
    fn test_har_exporter_add_entry() {
        let mut exporter = HarExporter::new();
        let flow = test_flow();
        exporter.add_entry(&flow);
        assert_eq!(exporter.log.entries.len(), 1);
    }

    #[test]
    fn test_har_exporter_export() {
        let mut exporter = HarExporter::new();
        let flow = test_flow();
        exporter.add_entry(&flow);
        let json = exporter.export().unwrap();
        assert!(json.contains("mitmproxy-rs"));
        assert!(json.contains("1.2"));
    }
}
