// MIT License
//
// Copyright (c) 2025 Mike Chambers
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::{Json, Response},
    routing::get,
    Router,
};
use clap::Parser;
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    net::SocketAddr,
    process::Command,
    sync::Arc,
    time::Instant,
};
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use adobe_common::socket_io::{decode_event, encode_event, ENGINE_PING, ENGINE_PONG, is_connect, is_disconnect};

#[derive(Parser, Debug)]
#[command(name = "adobe-proxy")]
#[command(about = "WebSocket proxy server for Adobe MCP applications", long_about = None)]
struct Args {
    /// Host to bind to
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Port to listen on
    #[arg(long, default_value = "3001")]
    port: u16,

    /// Attempt to auto-launch Adobe apps when no clients are connected
    #[arg(long, env = "ADOBE_PROXY_AUTO_LAUNCH", default_value_t = false)]
    auto_launch: bool,

    /// Auto-launch wait time in milliseconds before returning failure
    #[arg(long, env = "ADOBE_PROXY_AUTO_LAUNCH_TIMEOUT_MS", default_value_t = 20000)]
    auto_launch_timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RegisterMessage {
    application: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CommandPacket {
    application: String,
    command: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CommandPacketWithSender {
    #[serde(rename = "senderId")]
    sender_id: String,
    application: String,
    command: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CommandPacketResponse {
    packet: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RegistrationResponse {
    #[serde(rename = "type")]
    response_type: String,
    status: String,
    message: String,
}

#[derive(Debug, Clone)]
struct ClientInfo {
    #[allow(dead_code)] // Reserved for future use (metrics, logging)
    id: String,
    application: Option<String>,
    tx: broadcast::Sender<SocketIoMessage>,
}

#[derive(Debug, Clone)]
enum SocketIoMessage {
    Text(String),
    #[allow(dead_code)] // Reserved for graceful shutdown implementation
    Close,
}

#[derive(Clone)]
struct AppState {
    clients: Arc<DashMap<String, ClientInfo>>,
    application_clients: Arc<DashMap<String, Vec<String>>>,
    start_time: Instant,
    auto_launch: bool,
    auto_launch_timeout: Duration,
}

impl AppState {
    fn new(auto_launch: bool, auto_launch_timeout: Duration) -> Self {
        Self {
            clients: Arc::new(DashMap::new()),
            application_clients: Arc::new(DashMap::new()),
            start_time: Instant::now(),
            auto_launch,
            auto_launch_timeout,
        }
    }

    fn register_client(&self, client_id: String, application: String) {
        // Update client info
        if let Some(mut client) = self.clients.get_mut(&client_id) {
            client.application = Some(application.clone());
        }

        // Add to application clients
        self.application_clients
            .entry(application.clone())
            .and_modify(|clients| {
                if !clients.contains(&client_id) {
                    clients.push(client_id.clone());
                }
            })
            .or_insert_with(|| vec![client_id.clone()]);

        info!(
            "Client {} registered for application: {}",
            client_id, application
        );
    }

    fn unregister_client(&self, client_id: &str) {
        // Remove from clients
        if let Some((_, client_info)) = self.clients.remove(client_id) {
            // Remove from application clients
            if let Some(app) = &client_info.application {
                if let Some(mut clients) = self.application_clients.get_mut(app) {
                    clients.retain(|id| id != client_id);
                    if clients.is_empty() {
                        drop(clients);
                        self.application_clients.remove(app);
                    }
                }
            }
        }
        info!("Client {} disconnected and cleaned up", client_id);
    }

    fn send_to_application(&self, packet: &CommandPacketWithSender) -> bool {
        let application = &packet.application;

        if let Some(clients) = self.application_clients.get(application) {
            let client_count = clients.len();
            info!(
                "Sending to {} clients for application: {}",
                client_count, application
            );

            let event_data = json!({
                "senderId": packet.sender_id,
                "application": packet.application,
                "command": packet.command,
            });

            let socket_io_msg = encode_event("command_packet", event_data);

            for client_id in clients.iter() {
                if let Some(client) = self.clients.get(client_id) {
                    let _ = client.tx.send(SocketIoMessage::Text(socket_io_msg.clone()));
                }
            }

            return true;
        }

        warn!("No clients registered for application: {}", application);
        false
    }

    fn send_to_client(&self, client_id: &str, event: &str, data: Value) -> bool {
        if let Some(client) = self.clients.get(client_id) {
            let socket_io_msg = encode_event(event, data);
            let _ = client.tx.send(SocketIoMessage::Text(socket_io_msg));
            true
        } else {
            warn!("Client {} not found", client_id);
            false
        }
    }

    fn get_status(&self) -> StatusResponse {
        let mut clients_map = HashMap::new();

        for entry in self.application_clients.iter() {
            clients_map.insert(entry.key().clone(), entry.value().len());
        }

        StatusResponse {
            status: "running".to_string(),
            port: 3001, // Will be updated by the handler
            clients: clients_map,
            uptime: self.start_time.elapsed().as_secs(),
        }
    }

    fn has_application_client(&self, application: &str) -> bool {
        self.application_clients
            .get(application)
            .map(|clients| !clients.is_empty())
            .unwrap_or(false)
    }
}

#[derive(Debug, Serialize)]
struct StatusResponse {
    status: String,
    port: u16,
    clients: HashMap<String, usize>,
    uptime: u64,
}

// Socket.IO protocol encoding/decoding helpers are centralized in adobe-common::socket_io.

async fn handle_socket(
    socket: WebSocket,
    state: AppState,
    client_id: String,
) -> Result<(), anyhow::Error> {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = broadcast::channel::<SocketIoMessage>(100);

    // Store client info
    state.clients.insert(
        client_id.clone(),
        ClientInfo {
            id: client_id.clone(),
            application: None,
            tx: tx.clone(),
        },
    );

    info!("User connected: {}", client_id);

    // Engine.IO open + Socket.IO connect (required by socket.io clients)
    let connect_msg = format!("0{}", json!({"sid": client_id, "upgrades": [], "pingInterval": 25000, "pingTimeout": 20000}));
    sender.send(Message::Text(connect_msg)).await?;
    sender.send(Message::Text("40".to_string())).await?;

    // Spawn task to send outgoing messages
    let client_id_clone = client_id.clone();
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            match msg {
                SocketIoMessage::Text(text) => {
                    if sender.send(Message::Text(text)).await.is_err() {
                        break;
                    }
                }
                SocketIoMessage::Close => {
                    let _ = sender.send(Message::Close(None)).await;
                    break;
                }
            }
        }
        debug!("Send task completed for client: {}", client_id_clone);
    });

    // Handle incoming messages
    let state_clone = state.clone();
    let client_id_clone = client_id.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    debug!("Received from {}: {}", client_id_clone, text);

                    if text == ENGINE_PING {
                        let _ = tx.send(SocketIoMessage::Text(ENGINE_PONG.to_string()));
                        continue;
                    }

                    if is_connect(&text) || is_disconnect(&text) {
                        debug!("Socket.IO connection control: {}", text);
                        continue;
                    }

                    // Decode Socket.IO event
                    if let Some((event, data)) = decode_event(&text) {
                        handle_event(
                            &state_clone,
                            &client_id_clone,
                            &tx,
                            event.as_str(),
                            data,
                        )
                        .await;
                    } else {
                        warn!("Failed to parse Socket.IO message: {}", text);
                    }
                }
                Message::Close(_) => {
                    debug!("Client {} sent close message", client_id_clone);
                    break;
                }
                Message::Ping(_) => {
                    let _ = tx.send(SocketIoMessage::Text(ENGINE_PONG.to_string()));
                }
                _ => {}
            }
        }
        debug!("Receive task completed for client: {}", client_id_clone);
    });

    // Wait for either task to complete
    tokio::select! {
        _ = &mut send_task => {
            recv_task.abort();
        }
        _ = &mut recv_task => {
            send_task.abort();
        }
    }

    // Cleanup
    state.unregister_client(&client_id);

    Ok(())
}

async fn handle_event(
    state: &AppState,
    client_id: &str,
    tx: &broadcast::Sender<SocketIoMessage>,
    event: &str,
    data: Value,
) {
    debug!("Event '{}' from {}: {:?}", event, client_id, data);

    match event {
        "register" => {
            if let Ok(register_msg) = serde_json::from_value::<RegisterMessage>(data) {
                state.register_client(client_id.to_string(), register_msg.application.clone());

                let response = RegistrationResponse {
                    response_type: "registration".to_string(),
                    status: "success".to_string(),
                    message: format!("Registered for {}", register_msg.application),
                };

                let msg = encode_event("registration_response", json!(response));
                let _ = tx.send(SocketIoMessage::Text(msg));
            }
        }
        "command_packet" => {
            if let Ok(cmd_packet) = serde_json::from_value::<CommandPacket>(data) {
                info!(
                    "Command from {} for application {}: {:?}",
                    client_id, cmd_packet.application, cmd_packet.command
                );

                let packet_with_sender = CommandPacketWithSender {
                    sender_id: client_id.to_string(),
                    application: cmd_packet.application,
                    command: cmd_packet.command,
                };

                if !state.send_to_application(&packet_with_sender) {
                    let mut auto_launch_note = None;

                    if state.auto_launch {
                        if try_launch_application(&packet_with_sender.application) {
                            if wait_for_application(
                                state,
                                &packet_with_sender.application,
                                state.auto_launch_timeout,
                            )
                            .await
                            {
                                if state.send_to_application(&packet_with_sender) {
                                    return;
                                }
                            }

                            auto_launch_note = Some(format!(
                                "Auto-launch attempted, no client registered within {}ms",
                                state.auto_launch_timeout.as_millis()
                            ));
                        } else {
                            auto_launch_note = Some(format!(
                                "Auto-launch enabled but no executable found for application: {}",
                                packet_with_sender.application
                            ));
                        }
                    }

                    let mut message = format!(
                        "No clients registered for application: {}",
                        packet_with_sender.application
                    );
                    if let Some(note) = auto_launch_note {
                        message = format!("{}. {}", message, note);
                    }

                    let response = json!({
                        "senderId": client_id,
                        "status": "FAILURE",
                        "message": message
                    });
                    state.send_to_client(client_id, "packet_response", response);
                }
            }
        }
        "command_packet_response" => {
            if let Ok(response) = serde_json::from_value::<CommandPacketResponse>(data) {
                if let Some(sender_id) = response.packet.get("senderId").and_then(|v| v.as_str()) {
                    let sender_id = sender_id.to_string();
                    info!("Sending response to client {}", sender_id);
                    state.send_to_client(&sender_id, "packet_response", response.packet);
                } else {
                    warn!("No sender ID in command_packet_response");
                }
            }
        }
        _ => {
            warn!("Unknown event: {}", event);
        }
    }
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    let client_id = Uuid::new_v4().to_string();

    ws.on_upgrade(move |socket| async move {
        if let Err(e) = handle_socket(socket, state, client_id).await {
            error!("WebSocket error: {}", e);
        }
    })
}

async fn status_handler(State(state): State<AppState>) -> Json<StatusResponse> {
    let mut status = state.get_status();
    // Update port from the actual server config if needed
    status.port = 3001; // This could be passed through state if needed
    Json(status)
}

fn try_launch_application(application: &str) -> bool {
    let candidates = match application {
        "acrobat" => vec![
            r"C:\Program Files\Adobe\Acrobat DC\Acrobat\Acrobat.exe",
            r"C:\Program Files (x86)\Adobe\Acrobat Reader DC\Reader\AcroRd32.exe",
        ],
        "photoshop" => vec![
            r"C:\Program Files\Adobe\Adobe Photoshop 2024\Photoshop.exe",
            r"C:\Program Files\Adobe\Adobe Photoshop 2025\Photoshop.exe",
        ],
        "illustrator" => vec![
            r"C:\Program Files\Adobe\Adobe Illustrator 2024\Support Files\Contents\Windows\Illustrator.exe",
            r"C:\Program Files\Adobe\Adobe Illustrator 2025\Support Files\Contents\Windows\Illustrator.exe",
        ],
        "indesign" => vec![
            r"C:\Program Files\Adobe\Adobe InDesign 2024\InDesign.exe",
            r"C:\Program Files\Adobe\Adobe InDesign 2025\InDesign.exe",
        ],
        "premiere" => vec![
            r"C:\Program Files\Adobe\Adobe Premiere Pro 2024\Adobe Premiere Pro.exe",
            r"C:\Program Files\Adobe\Adobe Premiere Pro 2025\Adobe Premiere Pro.exe",
        ],
        _ => vec![],
    };

    for exe in candidates {
        if std::path::Path::new(exe).exists() {
            if Command::new(exe).spawn().is_ok() {
                info!("Auto-launched {} via {}", application, exe);
                return true;
            }
        }
    }

    false
}

async fn wait_for_application(
    state: &AppState,
    application: &str,
    timeout: Duration,
) -> bool {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if state.has_application_client(application) {
            return true;
        }
        sleep(Duration::from_millis(500)).await;
    }
    false
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(false)
        .with_level(true)
        .init();

    let args = Args::parse();
    let state = AppState::new(
        args.auto_launch,
        Duration::from_millis(args.auto_launch_timeout_ms),
    );

    let app = Router::new()
        .route("/status", get(status_handler))
        .route("/socket.io/", get(websocket_handler))
        .with_state(state);

    let addr = SocketAddr::from((
        args.host.parse::<std::net::IpAddr>()?,
        args.port,
    ));

    info!("adobe-mcp Command proxy server running on ws://{}", addr);
    info!("Status endpoint: http://{}/status", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
