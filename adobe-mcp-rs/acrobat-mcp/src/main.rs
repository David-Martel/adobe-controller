//! Adobe Acrobat MCP Server
//!
//! Model Context Protocol server for Adobe Acrobat automation via WebSocket proxy.

mod client;
mod mcp;
mod tools;

use clap::Parser;
use serde_json::json;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{error, info};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// WebSocket proxy URL
    #[arg(long, env = "ACROBAT_PROXY_URL", default_value = "ws://localhost:3001")]
    proxy_url: String,

    /// Command timeout in milliseconds
    #[arg(long, env = "ACROBAT_TIMEOUT", default_value = "30000")]
    timeout: u64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // CRITICAL: Log to stderr to avoid corrupting JSON-RPC stdout stream
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let args = Args::parse();
    info!("Starting acrobat-mcp with proxy: {}", args.proxy_url);

    // Initialize WebSocket client
    let client = Arc::new(client::AcrobatClient::new(&args.proxy_url, args.timeout).await?);
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

                        let request: Result<mcp::protocol::JsonRpcRequest, _> = serde_json::from_str(line);
                        match request {
                            Ok(req) => {
                                let _id = req.id.clone();
                                let response = handle_request(req, &client).await;
                                let response_json = serde_json::to_string(&response)?;
                                println!("{}", response_json);
                            }
                            Err(e) => {
                                error!("Failed to parse JSON-RPC: {}", e);
                                let err_resp = mcp::protocol::JsonRpcResponse {
                                    jsonrpc: "2.0".into(),
                                    id: None,
                                    result: None,
                                    error: Some(mcp::protocol::JsonRpcError::new(
                                        mcp::protocol::JsonRpcError::PARSE_ERROR,
                                        e.to_string()
                                    )),
                                };
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

    info!("Acrobat MCP server shutting down. Goodbye.");
    Ok(())
}

async fn handle_request(
    req: mcp::protocol::JsonRpcRequest,
    client: &Arc<client::AcrobatClient>,
) -> mcp::protocol::JsonRpcResponse {
    let id = req.id.clone();

    match req.method.as_str() {
        "ping" => mcp::protocol::JsonRpcResponse::success(id, json!({"status": "ok"})),

        "initialize" => mcp::protocol::JsonRpcResponse::success(
            id,
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": { "listChanged": false } },
                "serverInfo": {
                    "name": "acrobat-mcp",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }),
        ),

        "tools/list" => {
            let tools = tools::get_tool_definitions();
            mcp::protocol::JsonRpcResponse::success(id, json!({"tools": tools}))
        }

        "tools/call" => {
            if let Some(params) = req.params {
                let name = params.get("name").and_then(|v| v.as_str());
                let args = params.get("arguments").cloned().unwrap_or(json!({}));

                if let Some(tool_name) = name {
                    match tools::handle_tool_call(client, tool_name, args).await {
                        Ok(result) => mcp::protocol::JsonRpcResponse::success(
                            id,
                            json!({
                                "content": [{ "type": "text", "text": result }],
                                "isError": false
                            }),
                        ),
                        Err(e) => mcp::protocol::JsonRpcResponse::success(
                            id,
                            json!({
                                "content": [{ "type": "text", "text": format!("Error: {}", e) }],
                                "isError": true
                            }),
                        ),
                    }
                } else {
                    mcp::protocol::JsonRpcResponse::error(
                        id,
                        mcp::protocol::JsonRpcError::invalid_params("Missing tool name"),
                    )
                }
            } else {
                mcp::protocol::JsonRpcResponse::error(
                    id,
                    mcp::protocol::JsonRpcError::invalid_params("Missing params"),
                )
            }
        }

        _ => mcp::protocol::JsonRpcResponse::error(
            id,
            mcp::protocol::JsonRpcError::method_not_found(),
        ),
    }
}
