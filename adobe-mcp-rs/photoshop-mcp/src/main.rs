//! Adobe Photoshop MCP Server
//! 
//! Model Context Protocol server for Adobe Photoshop automation via WebSocket proxy.

mod client;
mod tools;

use clap::Parser;
use serde_json::json;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{error, info};
use adobe_common::{McpRequest, McpResponse, error_codes};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// WebSocket proxy URL
    #[arg(long, env = "PHOTOSHOP_PROXY_URL", default_value = "ws://localhost:3001")]
    proxy_url: String,

    /// Command timeout in milliseconds
    #[arg(long, env = "PHOTOSHOP_TIMEOUT", default_value = "30000")]
    timeout: u64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // CRITICAL: Log to stderr to avoid corrupting JSON-RPC stdout stream
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let args = Args::parse();
    info!("Starting photoshop-mcp with proxy: {}", args.proxy_url);

    // Initialize WebSocket client
    let client = Arc::new(client::PhotoshopClient::new(&args.proxy_url, args.timeout).await?);
    info!("Connected to proxy at {}", args.proxy_url);

    // Start JSON-RPC loop over stdio
    info!("Listening on stdin for MCP requests...");

    let stdin = tokio::io::stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();

    loop {
        tokio::select! {
            line_result = lines.next_line() => {
                match line_result {
                    Ok(Some(line)) => {
                        let line = line.trim();
                        if line.is_empty() {
                            continue;
                        }

                        let request: Result<McpRequest, _> = serde_json::from_str(line);
                        match request {
                            Ok(req) => {
                                let _id = req.id.clone();
                                let response = handle_request(req, &client).await;
                                let response_json = serde_json::to_string(&response)?;
                                println!("{}", response_json);
                            }
                            Err(e) => {
                                error!("Failed to parse JSON-RPC: {}", e);
                                let err_resp = McpResponse::error(
                                    json!(null),
                                    error_codes::PARSE_ERROR,
                                    format!("Parse error: {}", e)
                                );
                                println!("{}", serde_json::to_string(&err_resp)?);
                            }
                        }
                    }
                    Ok(None) => break, // EOF
                    Err(e) => {
                        error!("Error reading stdin: {}", e);
                        break;
                    }
                }
            }
            _ = tokio::signal::ctrl_c() => {
                info!("Received Ctrl+C, shutting down...");
                break;
            }
        }
    }

    info!("Photoshop MCP server shutting down. Goodbye.");
    Ok(())
}

async fn handle_request(
    req: McpRequest,
    client: &Arc<client::PhotoshopClient>,
) -> McpResponse {
    let id = req.id.clone();

    match req.method.as_str() {
        "ping" => McpResponse::success(id.unwrap_or(json!(null)), json!({"status": "ok"})),

        "initialize" => McpResponse::success(
            id.unwrap_or(json!(null)),
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": { "listChanged": false } },
                "serverInfo": {
                    "name": "photoshop-mcp",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }),
        ),

        "tools/list" => {
            let tools = tools::get_tool_definitions();
            McpResponse::success(id.unwrap_or(json!(null)), json!({"tools": tools}))
        }

        "tools/call" => {
            if let Some(params) = req.params {
                let name = params.get("name").and_then(|v| v.as_str());
                let args = params.get("arguments").cloned().unwrap_or(json!({}));

                if let Some(tool_name) = name {
                    match tools::handle_tool_call(client, tool_name, args).await {
                        Ok(result) => McpResponse::success(
                            id.unwrap_or(json!(null)),
                            json!({
                                "content": [{ "type": "text", "text": result }],
                                "isError": false
                            }),
                        ),
                        Err(e) => McpResponse::success(
                            id.unwrap_or(json!(null)),
                            json!({
                                "content": [{ "type": "text", "text": format!("Error: {}", e) }],
                                "isError": true
                            }),
                        ),
                    }
                } else {
                    McpResponse::error(
                        id.unwrap_or(json!(null)),
                        error_codes::INVALID_PARAMS,
                        "Missing tool name",
                    )
                }
            } else {
                McpResponse::error(
                    id.unwrap_or(json!(null)),
                    error_codes::INVALID_PARAMS,
                    "Missing params",
                )
            }
        }

        _ => McpResponse::error(
            id.unwrap_or(json!(null)),
            error_codes::METHOD_NOT_FOUND,
            "Method not found",
        ),
    }
}

