//! WebSocket client for communicating with Adobe proxy server

use adobe_common::{AdobeApplication, Command, CommandPacket, CommandResponse, ResponseStatus};
use adobe_common::socket_io::{decode_event, encode_event, ENGINE_PING, ENGINE_PONG};
use anyhow::{anyhow, Result};
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tracing::{debug, info};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// WebSocket client for Acrobat commands
pub struct AcrobatClient {
    ws: Arc<Mutex<WsStream>>,
    timeout_ms: u64,
}

impl AcrobatClient {
    /// Create new client and connect to proxy
    pub async fn new(proxy_url: &str, timeout_ms: u64) -> Result<Self> {
        info!("Connecting to proxy at {}", proxy_url);

        let (ws_stream, _) = connect_async(proxy_url)
            .await
            .map_err(|e| anyhow!("Failed to connect to proxy: {}", e))?;

        info!("WebSocket connection established");

        let client = Self {
            ws: Arc::new(Mutex::new(ws_stream)),
            timeout_ms,
        };

        {
            let mut ws = client.ws.lock().await;
            ws.send(tokio_tungstenite::tungstenite::Message::Text("40".to_string()))
                .await
                .map_err(|e| anyhow!("Failed to send Socket.IO connect: {}", e))?;
        }

        Ok(client)
    }

    /// Send command to Acrobat and wait for response
    pub async fn send_command(
        &self,
        action: impl Into<String>,
        options: Value,
    ) -> Result<CommandResponse> {
        let command = Command::new(action, options);
        let packet = CommandPacket::new(AdobeApplication::Acrobat, command);

        debug!("Sending command: {:?}", packet);

        let payload = serde_json::json!({
            "type": packet.packet_type,
            "application": packet.application,
            "command": packet.command,
        });

        let message = encode_event("command_packet", payload);
        let mut ws = self.ws.lock().await;

        ws.send(tokio_tungstenite::tungstenite::Message::Text(message))
            .await
            .map_err(|e| anyhow!("Failed to send message: {}", e))?;

        let timeout_duration = Duration::from_millis(self.timeout_ms);

        let response = timeout(timeout_duration, async {
            loop {
                let msg = ws.next().await.ok_or_else(|| anyhow!("WebSocket closed"))?;
                let msg = msg.map_err(|e| anyhow!("WebSocket error: {}", e))?;

                match msg {
                    tokio_tungstenite::tungstenite::Message::Text(text) => {
                        if text == ENGINE_PING {
                            ws.send(tokio_tungstenite::tungstenite::Message::Text(ENGINE_PONG.to_string()))
                                .await
                                .map_err(|e| anyhow!("Failed to send pong: {}", e))?;
                            continue;
                        }

                        if let Some((event, data)) = decode_event(&text) {
                            if event == "packet_response" {
                                let response: CommandResponse = serde_json::from_value(data)
                                    .map_err(|e| anyhow!("Failed to parse response: {}", e))?;
                                return Ok(response);
                            }
                            continue;
                        }

                        if text.starts_with('{') {
                            let response: CommandResponse = serde_json::from_str(&text)
                                .map_err(|e| anyhow!("Failed to parse response: {}", e))?;
                            return Ok(response);
                        }
                    }
                    tokio_tungstenite::tungstenite::Message::Close(_) => {
                        return Err(anyhow!("WebSocket connection closed"));
                    }
                    tokio_tungstenite::tungstenite::Message::Ping(_) => {
                        ws.send(tokio_tungstenite::tungstenite::Message::Text(ENGINE_PONG.to_string()))
                            .await
                            .map_err(|e| anyhow!("Failed to send pong: {}", e))?;
                    }
                    _ => {}
                }
            }
        })
        .await
        .map_err(|_| anyhow!("Command timeout after {}ms", self.timeout_ms))??;

        if response.status == ResponseStatus::Success {
            Ok(response)
        } else {
            Err(anyhow!(
                "Command failed: {}",
                response.message.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    /// Get response data as JSON value
    pub fn extract_response(response: &CommandResponse) -> Option<&Value> {
        response.response.as_ref()
    }

    /// Get document info from response
    #[allow(dead_code)]
    pub fn extract_document(response: &CommandResponse) -> Option<&Value> {
        response.document.as_ref()
    }
}
