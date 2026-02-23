//! Error types for Acrobat Bridge
//!
//! Provides structured error handling without panics.

use std::fmt;

/// Result type for bridge operations
pub type BridgeResult<T> = Result<T, BridgeError>;

/// Bridge error types
#[derive(Debug)]
pub enum BridgeError {
    /// WebSocket connection failed
    ConnectionFailed(String),
    /// WebSocket send failed
    SendFailed(String),
    /// WebSocket receive failed
    ReceiveFailed(String),
    /// Command execution failed
    CommandFailed(String),
    /// JavaScript execution failed
    JsExecutionFailed(String),
    /// Invalid command format
    InvalidCommand(String),
    /// Timeout waiting for response
    Timeout(String),
    /// Plugin not initialized
    NotInitialized,
    /// Plugin already initialized
    AlreadyInitialized,
    /// Serialization error
    Serialization(String),
    /// Deserialization error
    Deserialization(String),
    /// Invalid state
    InvalidState(String),
    /// IO error
    Io(String),
}

impl fmt::Display for BridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            Self::SendFailed(msg) => write!(f, "Send failed: {}", msg),
            Self::ReceiveFailed(msg) => write!(f, "Receive failed: {}", msg),
            Self::CommandFailed(msg) => write!(f, "Command failed: {}", msg),
            Self::JsExecutionFailed(msg) => write!(f, "JS execution failed: {}", msg),
            Self::InvalidCommand(msg) => write!(f, "Invalid command: {}", msg),
            Self::Timeout(msg) => write!(f, "Timeout: {}", msg),
            Self::NotInitialized => write!(f, "Plugin not initialized"),
            Self::AlreadyInitialized => write!(f, "Plugin already initialized"),
            Self::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            Self::Deserialization(msg) => write!(f, "Deserialization error: {}", msg),
            Self::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
            Self::Io(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for BridgeError {}

impl From<serde_json::Error> for BridgeError {
    fn from(err: serde_json::Error) -> Self {
        BridgeError::Serialization(err.to_string())
    }
}

impl From<std::io::Error> for BridgeError {
    fn from(err: std::io::Error) -> Self {
        BridgeError::Io(err.to_string())
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for BridgeError {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        BridgeError::ConnectionFailed(err.to_string())
    }
}

impl From<tokio::sync::mpsc::error::SendError<String>> for BridgeError {
    fn from(err: tokio::sync::mpsc::error::SendError<String>) -> Self {
        BridgeError::SendFailed(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = BridgeError::ConnectionFailed("test".to_string());
        assert!(err.to_string().contains("Connection failed"));
        assert!(err.to_string().contains("test"));
    }

    #[test]
    fn test_error_from_serde() {
        let json_err: Result<(), serde_json::Error> = serde_json::from_str("invalid");
        let bridge_err: BridgeError = json_err.unwrap_err().into();
        assert!(matches!(bridge_err, BridgeError::Serialization(_)));
    }

    #[test]
    fn test_all_error_variants_display() {
        let errors = vec![
            BridgeError::ConnectionFailed("test".into()),
            BridgeError::SendFailed("test".into()),
            BridgeError::ReceiveFailed("test".into()),
            BridgeError::CommandFailed("test".into()),
            BridgeError::JsExecutionFailed("test".into()),
            BridgeError::InvalidCommand("test".into()),
            BridgeError::Timeout("test".into()),
            BridgeError::NotInitialized,
            BridgeError::AlreadyInitialized,
            BridgeError::Serialization("test".into()),
            BridgeError::Deserialization("test".into()),
            BridgeError::InvalidState("test".into()),
            BridgeError::Io("test".into()),
        ];

        for err in errors {
            // Should not panic
            let _ = format!("{}", err);
            let _ = format!("{:?}", err);
        }
    }
}
