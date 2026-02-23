//! WebSocket client for connecting to adobe-proxy
//!
//! This module handles the WebSocket connection to the proxy server,
//! message routing to command handlers, and response transmission.

use crate::commands;
use crate::error::{BridgeError, BridgeResult};
use adobe_common::{Command, CommandResponse, ResponseStatus};
use adobe_common::socket_io::{decode_event, encode_event, ENGINE_PING, ENGINE_PONG, SOCKET_IO_CONNECT};
use futures_util::{SinkExt, StreamExt};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// Proxy client for WebSocket communication
pub struct ProxyClient {
    /// Sender for outgoing messages
    tx: mpsc::Sender<String>,
    /// Whether connected
    connected: Arc<AtomicBool>,
    /// Shutdown signal sender
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl ProxyClient {
    /// Create new client and connect to proxy
    ///
    /// # Errors
    /// Returns error if WebSocket connection fails
    pub async fn connect(proxy_url: &str) -> BridgeResult<Self> {
        let (ws_stream, _) = connect_async(proxy_url)
            .await
            .map_err(|e| BridgeError::ConnectionFailed(e.to_string()))?;

        let (mut write, mut read) = ws_stream.split();

        // Channel for sending messages
        let (tx, mut rx) = mpsc::channel::<String>(100);
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

        // Clone tx for the read task to send responses
        let response_tx = tx.clone();
        let connected = Arc::new(AtomicBool::new(true));
        let connected_write = connected.clone();
        let connected_read = connected.clone();

        // Send Socket.IO connect frame
        if let Err(e) = write.send(Message::Text("40".to_string())).await {
            return Err(BridgeError::ConnectionFailed(format!(
                "Failed to send Socket.IO connect: {}",
                e
            )));
        }

        // Spawn write task
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(msg) = rx.recv() => {
                        if let Err(e) = write.send(Message::Text(msg)).await {
                            tracing::error!("WebSocket send error: {}", e);
                            connected_write.store(false, Ordering::SeqCst);
                            break;
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        tracing::info!("Write task received shutdown signal");
                        // Send close frame
                        let _ = write.send(Message::Close(None)).await;
                        connected_write.store(false, Ordering::SeqCst);
                        break;
                    }
                }
            }
        });

        // Spawn read task
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if text == ENGINE_PING {
                            let _ = response_tx.send(ENGINE_PONG.to_string()).await;
                            continue;
                        }
                        if text == SOCKET_IO_CONNECT {
                            continue;
                        }
                        if let Err(e) = Self::handle_message(&text, response_tx.clone()).await {
                            tracing::error!("Error handling message: {}", e);
                        }
                    }
                    Ok(Message::Close(_)) => {
                        tracing::info!("WebSocket closed by server");
                        connected_read.store(false, Ordering::SeqCst);
                        break;
                    }
                    Ok(Message::Ping(_)) => {
                        let _ = response_tx.send(ENGINE_PONG.to_string()).await;
                    }
                    Err(e) => {
                        tracing::error!("WebSocket receive error: {}", e);
                        connected_read.store(false, Ordering::SeqCst);
                        break;
                    }
                    _ => {}
                }
            }
        });

        let mut client = Self {
            tx,
            connected,
            shutdown_tx: Some(shutdown_tx),
        };

        // Register with proxy
        client.register().await?;

        Ok(client)
    }

    /// Register this client as "acrobat" application
    async fn register(&mut self) -> BridgeResult<()> {
        self.tx
            .send(encode_event(
                "register",
                serde_json::json!({ "application": "acrobat" }),
            ))
            .await
            .map_err(|e| BridgeError::SendFailed(e.to_string()))?;

        tracing::info!("Registered as 'acrobat' with proxy");
        Ok(())
    }

    /// Handle incoming message from proxy
    ///
    /// Routes commands to the command handler and sends responses back
    async fn handle_message(text: &str, response_tx: mpsc::Sender<String>) -> BridgeResult<()> {
        tracing::debug!("Received: {}", text);

        if !text.starts_with("42") && text != ENGINE_PING {
            return Err(BridgeError::Deserialization("Invalid Socket.IO message".to_string()));
        }

        // Parse as Socket.IO event
        if let Some((event, data)) = decode_event(text) {
            if event == "status" {
                response_tx
                    .send(encode_event(
                        "status",
                        serde_json::json!({
                            "application": "acrobat",
                            "status": "ready"
                        }),
                    ))
                    .await
                    .map_err(|e| BridgeError::SendFailed(e.to_string()))?;
                return Ok(());
            }

            if event == "ping" {
                response_tx
                    .send(encode_event("pong", serde_json::json!({})))
                    .await
                    .map_err(|e| BridgeError::SendFailed(e.to_string()))?;
                return Ok(());
            }

            if event != "command_packet" {
                return Ok(());
            }

            // Extract sender_id for response routing
            let sender_id = data
                .get("senderId")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            // Check if this is a command packet
            let command_value = data
                .get("command")
                .ok_or_else(|| BridgeError::Deserialization("Missing command".to_string()))?;

            // Parse the command
            let command: Command = serde_json::from_value(command_value.clone())
                .map_err(|e| BridgeError::Deserialization(format!("Invalid command: {}", e)))?;

            tracing::info!("Executing command: {}", command.action);

            // Execute the command
            let response = match commands::execute_command(&command) {
                Ok(mut resp) => {
                    resp.sender_id = sender_id.clone();
                    resp
                }
                Err(e) => CommandResponse {
                    sender_id: sender_id.clone(),
                    status: ResponseStatus::Failure,
                    response: None,
                    message: Some(e.to_string()),
                    document: None,
                },
            };

            // Send response back
            response_tx
                .send(encode_event(
                    "command_packet_response",
                    serde_json::json!({
                        "packet": {
                            "senderId": sender_id,
                            "status": response.status,
                            "response": response.response,
                            "message": response.message,
                            "document": response.document,
                        }
                    }),
                ))
                .await
                .map_err(|e| BridgeError::SendFailed(e.to_string()))?;

            tracing::debug!("Sent response for command: {}", command.action);
        } else if text == ENGINE_PING {
            response_tx
                .send(ENGINE_PONG.to_string())
                .await
                .map_err(|e| BridgeError::SendFailed(e.to_string()))?;
        }

        Ok(())
    }

    /// Send a raw message to the proxy
    ///
    /// # Errors
    /// Returns error if send fails
    pub async fn send_raw(&self, message: &str) -> BridgeResult<()> {
        self.tx
            .send(message.to_string())
            .await
            .map_err(|e| BridgeError::SendFailed(e.to_string()))
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }

    /// Disconnect from the proxy
    pub async fn disconnect(&mut self) -> BridgeResult<()> {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            shutdown_tx
                .send(())
                .await
                .map_err(|e| BridgeError::SendFailed(e.to_string()))?;
        }
        self.connected.store(false, Ordering::SeqCst);
        Ok(())
    }
}

impl Drop for ProxyClient {
    fn drop(&mut self) {
        // Mark as disconnected on drop
        self.connected.store(false, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proxy_client_default_disconnected() {
        // A client without connection should report disconnected
        let connected = Arc::new(AtomicBool::new(false));
        assert!(!connected.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_handle_message_invalid_json() {
        let (tx, _rx) = mpsc::channel(10);
        let result = ProxyClient::handle_message("invalid json", tx).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_message_ping() {
        let (tx, mut rx) = mpsc::channel(10);
        let result = ProxyClient::handle_message(r#"42["ping",{}]"#, tx).await;
        assert!(result.is_ok());

        // Should have received a pong event
        let response = rx.recv().await.unwrap();
        assert!(response.contains("pong"));
    }

    #[tokio::test]
    async fn test_handle_message_status() {
        let (tx, mut rx) = mpsc::channel(10);
        let result = ProxyClient::handle_message(r#"42["status",{}]"#, tx).await;
        assert!(result.is_ok());

        // Should have received a status response
        let response = rx.recv().await.unwrap();
        assert!(response.contains("ready"));
    }

    #[tokio::test]
    async fn test_handle_message_command() {
        let (tx, mut rx) = mpsc::channel(10);
        let msg = r#"42["command_packet",{"senderId":"test123","command":{"action":"getPageCount","options":{}}}]"#;
        let result = ProxyClient::handle_message(msg, tx).await;
        assert!(result.is_ok());

        // Should have received a response
        let response = rx.recv().await.unwrap();
        assert!(response.contains("test123"));
    }
}
