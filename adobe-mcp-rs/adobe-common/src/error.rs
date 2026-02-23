//! Error types for Adobe MCP

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AdobeError {
    #[error("Unknown application: {0}")]
    UnknownApplication(String),

    #[error("Application not connected: {0}")]
    ApplicationNotConnected(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Command timeout after {0}ms")]
    CommandTimeout(u64),

    #[error("Command failed: {0}")]
    CommandFailed(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type AdobeResult<T> = Result<T, AdobeError>;
