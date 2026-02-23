//! Photoshop tool definitions and handlers

use crate::client::PhotoshopClient;
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::sync::Arc;

/// Get all tool definitions for MCP tools/list
pub fn get_tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "create_document",
            "description": "Create a new Photoshop document",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "document_name": {
                        "type": "string",
                        "description": "Name for the new document"
                    },
                    "width": {
                        "type": "integer",
                        "description": "Width in pixels",
                        "default": 1920
                    },
                    "height": {
                        "type": "integer",
                        "description": "Height in pixels",
                        "default": 1080
                    },
                    "resolution": {
                        "type": "integer",
                        "description": "Resolution in pixels/inch",
                        "default": 72
                    },
                    "fill_color": {
                        "type": "object",
                        "properties": {
                            "red": { "type": "integer" },
                            "green": { "type": "integer" },
                            "blue": { "type": "integer" }
                        },
                        "description": "Background fill color"
                    },
                    "color_mode": {
                        "type": "string",
                        "description": "Color mode (RGB, CMYK, etc)",
                        "default": "RGB"
                    }
                },
                "required": ["document_name", "width", "height"]
            }
        }),
        json!({
            "name": "save_document",
            "description": "Save the current document",
            "inputSchema": {
                "type": "object",
                "properties": {
                },
            }
        }),
         json!({
            "name": "save_document_as",
            "description": "Save the current document to a specific path",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Absolute path to save file"
                    },
                    "file_type": {
                         "type": "string",
                         "description": "File format (PSD, PNG, JPG)",
                         "default": "PSD"
                    }
                },
                "required": ["file_path"]
            }
        }),
        json!({
            "name": "get_document_info",
            "description": "Get information about the current document",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }),
        json!({
            "name": "get_layers",
            "description": "Get hierarchy of layers in the document",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }),
        json!({
            "name": "create_pixel_layer",
            "description": "Create a new pixel layer",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "layer_name": {
                        "type": "string",
                        "description": "Name of the new layer"
                    },
                    "opacity": {
                        "type": "integer",
                        "description": "Opacity (0-100)",
                        "default": 100
                    },
                    "blend_mode": {
                        "type": "string",
                        "description": "Blend mode",
                        "default": "NORMAL"
                    }
                },
                "required": ["layer_name"]
            }
        }),
        json!({
            "name": "generate_image",
            "description": "Generate an image using Adobe Firefly",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "layer_name": {
                        "type": "string",
                        "description": "Name for the layer to contain the generated image"
                    },
                    "prompt": {
                        "type": "string",
                        "description": "Prompt describing the image"
                    },
                    "content_type": {
                        "type": "string",
                        "description": "Content type (photo, art, none)",
                        "default": "none"
                    }
                },
                "required": ["layer_name", "prompt"]
            }
        }),
    ]
}

/// Handle tool call and route to appropriate function
pub async fn handle_tool_call(
    client: &Arc<PhotoshopClient>,
    tool_name: &str,
    args: Value,
) -> Result<String> {
    match tool_name {
        "create_document" => create_document(client, args).await,
        "save_document" => save_document(client, args).await,
        "save_document_as" => save_document_as(client, args).await,
        "get_document_info" => get_document_info(client, args).await,
        "get_layers" => get_layers(client, args).await,
        "create_pixel_layer" => create_pixel_layer(client, args).await,
        "generate_image" => generate_image(client, args).await,
        _ => Err(anyhow!("Unknown tool: {}", tool_name)),
    }
}

// Tool implementations

async fn create_document(client: &Arc<PhotoshopClient>, args: Value) -> Result<String> {
    let name = args
        .get("document_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing required field: document_name"))?;
    
    let width = args.get("width").and_then(|v| v.as_i64()).unwrap_or(1920);
    let height = args.get("height").and_then(|v| v.as_i64()).unwrap_or(1080);
    let resolution = args.get("resolution").and_then(|v| v.as_i64()).unwrap_or(72);
    let color_mode = args.get("color_mode").and_then(|v| v.as_str()).unwrap_or("RGB");
    let fill_color = args.get("fill_color").cloned().unwrap_or(json!({"red": 255, "green": 255, "blue": 255}));

    let options = json!({
        "name": name,
        "width": width,
        "height": height,
        "resolution": resolution,
        "fillColor": fill_color,
        "colorMode": color_mode
    });

    let _response = client.send_command("createDocument", options).await?;
    Ok(format!("Created document: {} ({}x{})", name, width, height))
}

async fn save_document(client: &Arc<PhotoshopClient>, _args: Value) -> Result<String> {
    let _response = client.send_command("saveDocument", json!({})).await?;
    Ok("Document saved".to_string())
}

async fn save_document_as(client: &Arc<PhotoshopClient>, args: Value) -> Result<String> {
    let file_path = args.get("file_path").and_then(|v| v.as_str()).ok_or_else(|| anyhow!("Missing file_path"))?;
    let file_type = args.get("file_type").and_then(|v| v.as_str()).unwrap_or("PSD");

    let options = json!({
        "filePath": file_path,
        "fileType": file_type
    });

    let _response = client.send_command("saveDocumentAs", options).await?;
    Ok(format!("Document saved to: {}", file_path))
}

async fn get_document_info(client: &Arc<PhotoshopClient>, _args: Value) -> Result<String> {
    let response = client.send_command("getDocumentInfo", json!({})).await?;
    
    if let Some(data) = PhotoshopClient::extract_response(&response) {
        Ok(format!("Document info:\n{}", serde_json::to_string_pretty(data)?))
    } else {
        Ok("No document info returned".to_string())
    }
}

async fn get_layers(client: &Arc<PhotoshopClient>, _args: Value) -> Result<String> {
    let response = client.send_command("getLayers", json!({})).await?;

    if let Some(data) = PhotoshopClient::extract_response(&response) {
        Ok(format!("Layers:\n{}", serde_json::to_string_pretty(data)?))
    } else {
        Ok("No layers info returned".to_string())
    }
}

async fn create_pixel_layer(client: &Arc<PhotoshopClient>, args: Value) -> Result<String> {
    let layer_name = args.get("layer_name").and_then(|v| v.as_str()).ok_or_else(|| anyhow!("Missing layer_name"))?;
    let opacity = args.get("opacity").and_then(|v| v.as_i64()).unwrap_or(100);
    let blend_mode = args.get("blend_mode").and_then(|v| v.as_str()).unwrap_or("NORMAL");

    let options = json!({
        "layerName": layer_name,
        "opacity": opacity,
        "blendMode": blend_mode,
        "fillNeutral": false
    });

    let _response = client.send_command("createPixelLayer", options).await?;
    Ok(format!("Created pixel layer: {}", layer_name))
}

async fn generate_image(client: &Arc<PhotoshopClient>, args: Value) -> Result<String> {
    let layer_name = args.get("layer_name").and_then(|v| v.as_str()).ok_or_else(|| anyhow!("Missing layer_name"))?;
    let prompt = args.get("prompt").and_then(|v| v.as_str()).ok_or_else(|| anyhow!("Missing prompt"))?;
    let content_type = args.get("content_type").and_then(|v| v.as_str()).unwrap_or("none");

    let options = json!({
        "layerName": layer_name,
        "prompt": prompt,
        "contentType": content_type
    });

    let _response = client.send_command("generateImage", options).await?;
    Ok(format!("Generated image '{}' with prompt: {}", layer_name, prompt))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_definitions() {
        let tools = get_tool_definitions();
        assert!(tools.len() > 0);
        
        for tool in tools {
            assert!(tool.get("name").is_some());
            assert!(tool.get("description").is_some());
            assert!(tool.get("inputSchema").is_some());
        }
    }
}
