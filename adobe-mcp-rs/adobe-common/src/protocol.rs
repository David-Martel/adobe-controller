//! WebSocket protocol messages for Adobe MCP proxy communication

use serde::{Deserialize, Serialize};
use crate::types::AdobeApplication;

/// Command sent from MCP server to proxy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPacket {
    /// Type of packet (always "command")
    #[serde(rename = "type")]
    pub packet_type: String,
    /// Target application
    pub application: String,
    /// The command to execute
    pub command: Command,
}

impl CommandPacket {
    pub fn new(application: AdobeApplication, command: Command) -> Self {
        Self {
            packet_type: "command".to_string(),
            application: application.as_str().to_string(),
            command,
        }
    }
}

/// A command to be executed in an Adobe application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    /// Action name (e.g., "createDocument", "addText")
    pub action: String,
    /// Action parameters
    #[serde(default)]
    pub options: serde_json::Value,
}

impl Command {
    pub fn new(action: impl Into<String>, options: serde_json::Value) -> Self {
        Self {
            action: action.into(),
            options,
        }
    }
}

/// Response from application via proxy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResponse {
    /// Sender ID for routing
    #[serde(rename = "senderId")]
    pub sender_id: String,
    /// Status: "SUCCESS" or "FAILURE"
    pub status: ResponseStatus,
    /// Response data (on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<serde_json::Value>,
    /// Error message (on failure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Document info (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ResponseStatus {
    Success,
    Failure,
}

/// Registration message from application to proxy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterMessage {
    pub application: String,
}

/// Registration response from proxy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationResponse {
    #[serde(rename = "type")]
    pub response_type: String,
    pub status: String,
    pub message: String,
}

/// Internal packet with routing info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutedPacket {
    #[serde(rename = "senderId")]
    pub sender_id: String,
    pub application: String,
    pub command: Command,
}

/// MCP JSON-RPC request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl McpRequest {
    pub fn new(id: Option<impl Into<serde_json::Value>>, method: impl Into<String>, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: id.map(|i| i.into()),
            method: method.into(),
            params,
        }
    }
}

/// MCP JSON-RPC response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpError>,
}

impl McpResponse {
    pub fn success(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: serde_json::Value, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(McpError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// MCP error codes
pub mod error_codes {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
    pub const APPLICATION_NOT_CONNECTED: i32 = -32000;
    pub const COMMAND_TIMEOUT: i32 = -32001;
    pub const COMMAND_FAILED: i32 = -32002;
}
