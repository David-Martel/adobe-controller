//! JSON-RPC 2.0 protocol types for MCP

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    #[allow(dead_code)]
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
    pub id: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    pub const PARSE_ERROR: i32 = -32700;
    #[allow(dead_code)]
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    #[allow(dead_code)]
    pub const INTERNAL_ERROR: i32 = -32603;

    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    #[allow(dead_code)]
    pub fn parse_error(detail: impl Into<String>) -> Self {
        Self::new(Self::PARSE_ERROR, format!("Parse error: {}", detail.into()))
    }

    #[allow(dead_code)]
    pub fn invalid_request(detail: impl Into<String>) -> Self {
        Self::new(
            Self::INVALID_REQUEST,
            format!("Invalid Request: {}", detail.into()),
        )
    }

    pub fn method_not_found() -> Self {
        Self::new(Self::METHOD_NOT_FOUND, "Method not found")
    }

    pub fn invalid_params(detail: impl Into<String>) -> Self {
        Self::new(
            Self::INVALID_PARAMS,
            format!("Invalid params: {}", detail.into()),
        )
    }

    #[allow(dead_code)]
    pub fn internal_error(detail: impl Into<String>) -> Self {
        Self::new(
            Self::INTERNAL_ERROR,
            format!("Internal error: {}", detail.into()),
        )
    }
}

impl JsonRpcResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<Value>, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}
