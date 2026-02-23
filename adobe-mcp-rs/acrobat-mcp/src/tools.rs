//! Acrobat tool definitions and handlers

use crate::client::AcrobatClient;
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::sync::Arc;

/// Get all tool definitions for MCP tools/list
pub fn get_tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "create_document",
            "description": "Create a new PDF document with specified size and page count",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Document name"
                    },
                    "page_size": {
                        "type": "string",
                        "description": "Page size preset",
                        "enum": ["LETTER", "LEGAL", "A4", "A3", "CUSTOM"],
                        "default": "LETTER"
                    },
                    "page_count": {
                        "type": "integer",
                        "description": "Number of pages to create",
                        "default": 1
                    },
                    "width": {
                        "type": "number",
                        "description": "Custom width in points (for CUSTOM page_size)"
                    },
                    "height": {
                        "type": "number",
                        "description": "Custom height in points (for CUSTOM page_size)"
                    }
                },
                "required": ["name"]
            }
        }),
        json!({
            "name": "open_document",
            "description": "Open an existing PDF document",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Absolute path to PDF file"
                    }
                },
                "required": ["file_path"]
            }
        }),
        json!({
            "name": "save_document",
            "description": "Save the current document",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Path to save the document"
                    },
                    "format": {
                        "type": "string",
                        "description": "Save format",
                        "enum": ["PDF", "PDF_A", "PDF_X"],
                        "default": "PDF"
                    }
                },
                "required": ["file_path"]
            }
        }),
        json!({
            "name": "close_document",
            "description": "Close the currently active document",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "save_changes": {
                        "type": "boolean",
                        "description": "Save changes before closing",
                        "default": false
                    }
                }
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
            "name": "add_text",
            "description": "Add text to a specific page",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "page": {
                        "type": "integer",
                        "description": "Page number (1-based)",
                        "default": 1
                    },
                    "text": {
                        "type": "string",
                        "description": "Text content to add"
                    },
                    "x": {
                        "type": "number",
                        "description": "X coordinate in points",
                        "default": 72
                    },
                    "y": {
                        "type": "number",
                        "description": "Y coordinate in points",
                        "default": 720
                    },
                    "font_size": {
                        "type": "number",
                        "description": "Font size in points",
                        "default": 12
                    },
                    "font_name": {
                        "type": "string",
                        "description": "Font name",
                        "default": "Helvetica"
                    }
                },
                "required": ["text"]
            }
        }),
        json!({
            "name": "extract_text",
            "description": "Extract text from specified page range",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "page_range": {
                        "type": "string",
                        "description": "Page range (e.g., '1-5', 'all')",
                        "default": "all"
                    }
                }
            }
        }),
        json!({
            "name": "export_as",
            "description": "Export document to different format",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "Output file path"
                    },
                    "format": {
                        "type": "string",
                        "description": "Export format",
                        "enum": ["PDF", "PNG", "JPEG", "TIFF", "DOCX", "PPTX"],
                        "default": "PDF"
                    },
                    "quality": {
                        "type": "integer",
                        "description": "Quality for image formats (1-100)",
                        "default": 90
                    }
                },
                "required": ["file_path", "format"]
            }
        }),
        json!({
            "name": "merge_documents",
            "description": "Merge multiple PDF documents into one",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file_paths": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Array of PDF file paths to merge"
                    },
                    "output_path": {
                        "type": "string",
                        "description": "Output file path for merged PDF"
                    }
                },
                "required": ["file_paths", "output_path"]
            }
        }),
        json!({
            "name": "split_document",
            "description": "Split document into multiple PDFs by page ranges",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "page_ranges": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Array of page ranges (e.g., ['1-3', '4-6'])"
                    },
                    "output_dir": {
                        "type": "string",
                        "description": "Output directory for split PDFs"
                    },
                    "name_pattern": {
                        "type": "string",
                        "description": "Filename pattern (e.g., 'part_{n}.pdf')",
                        "default": "split_{n}.pdf"
                    }
                },
                "required": ["page_ranges", "output_dir"]
            }
        }),
        json!({
            "name": "get_page_count",
            "description": "Get the number of pages in the current document",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        }),
        json!({
            "name": "delete_pages",
            "description": "Delete specified pages from the document",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "page_numbers": {
                        "type": "array",
                        "items": { "type": "integer" },
                        "description": "Array of page numbers to delete (1-based)"
                    }
                },
                "required": ["page_numbers"]
            }
        }),
        json!({
            "name": "rotate_pages",
            "description": "Rotate specified pages by angle",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "page_numbers": {
                        "type": "array",
                        "items": { "type": "integer" },
                        "description": "Array of page numbers to rotate (1-based)"
                    },
                    "angle": {
                        "type": "integer",
                        "description": "Rotation angle in degrees",
                        "enum": [90, 180, 270]
                    }
                },
                "required": ["page_numbers", "angle"]
            }
        }),
        json!({
            "name": "add_bookmark",
            "description": "Add a bookmark to a specific page",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "title": {
                        "type": "string",
                        "description": "Bookmark title"
                    },
                    "page": {
                        "type": "integer",
                        "description": "Target page number (1-based)"
                    },
                    "parent": {
                        "type": "string",
                        "description": "Parent bookmark title (optional)"
                    }
                },
                "required": ["title", "page"]
            }
        }),
        json!({
            "name": "set_metadata",
            "description": "Set document metadata (title, author, subject, keywords)",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "title": {
                        "type": "string",
                        "description": "Document title"
                    },
                    "author": {
                        "type": "string",
                        "description": "Document author"
                    },
                    "subject": {
                        "type": "string",
                        "description": "Document subject"
                    },
                    "keywords": {
                        "type": "string",
                        "description": "Document keywords"
                    }
                }
            }
        }),
    ]
}

/// Handle tool call and route to appropriate function
pub async fn handle_tool_call(
    client: &Arc<AcrobatClient>,
    tool_name: &str,
    args: Value,
) -> Result<String> {
    match tool_name {
        "create_document" => create_document(client, args).await,
        "open_document" => open_document(client, args).await,
        "save_document" => save_document(client, args).await,
        "close_document" => close_document(client, args).await,
        "get_document_info" => get_document_info(client, args).await,
        "add_text" => add_text(client, args).await,
        "extract_text" => extract_text(client, args).await,
        "export_as" => export_as(client, args).await,
        "merge_documents" => merge_documents(client, args).await,
        "split_document" => split_document(client, args).await,
        "get_page_count" => get_page_count(client, args).await,
        "delete_pages" => delete_pages(client, args).await,
        "rotate_pages" => rotate_pages(client, args).await,
        "add_bookmark" => add_bookmark(client, args).await,
        "set_metadata" => set_metadata(client, args).await,
        _ => Err(anyhow!("Unknown tool: {}", tool_name)),
    }
}

// Tool implementations

async fn create_document(client: &Arc<AcrobatClient>, args: Value) -> Result<String> {
    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing required field: name"))?;

    let options = json!({
        "name": name,
        "pageSize": args.get("page_size").and_then(|v| v.as_str()).unwrap_or("LETTER"),
        "pageCount": args.get("page_count").and_then(|v| v.as_i64()).unwrap_or(1),
        "width": args.get("width").and_then(|v| v.as_f64()),
        "height": args.get("height").and_then(|v| v.as_f64()),
    });

    let response = client.send_command("createDocument", options).await?;
    Ok(format!(
        "Created document: {}",
        serde_json::to_string_pretty(&response.document)?
    ))
}

async fn open_document(client: &Arc<AcrobatClient>, args: Value) -> Result<String> {
    let file_path = args
        .get("file_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing required field: file_path"))?;

    let options = json!({ "filePath": file_path });
    let response = client.send_command("openDocument", options).await?;

    Ok(format!(
        "Opened document: {}",
        serde_json::to_string_pretty(&response.document)?
    ))
}

async fn save_document(client: &Arc<AcrobatClient>, args: Value) -> Result<String> {
    let file_path = args
        .get("file_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing required field: file_path"))?;

    let options = json!({
        "filePath": file_path,
        "format": args.get("format").and_then(|v| v.as_str()).unwrap_or("PDF"),
    });

    let _response = client.send_command("saveDocument", options).await?;
    Ok(format!("Document saved to: {}", file_path))
}

async fn close_document(client: &Arc<AcrobatClient>, args: Value) -> Result<String> {
    let options = json!({
        "saveChanges": args.get("save_changes").and_then(|v| v.as_bool()).unwrap_or(false),
    });

    let _response = client.send_command("closeDocument", options).await?;
    Ok("Document closed".to_string())
}

async fn get_document_info(client: &Arc<AcrobatClient>, _args: Value) -> Result<String> {
    let response = client.send_command("getDocumentInfo", json!({})).await?;

    Ok(format!(
        "Document info:\n{}",
        serde_json::to_string_pretty(&response.document)?
    ))
}

async fn add_text(client: &Arc<AcrobatClient>, args: Value) -> Result<String> {
    let text = args
        .get("text")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing required field: text"))?;

    let options = json!({
        "page": args.get("page").and_then(|v| v.as_i64()).unwrap_or(1),
        "text": text,
        "x": args.get("x").and_then(|v| v.as_f64()).unwrap_or(72.0),
        "y": args.get("y").and_then(|v| v.as_f64()).unwrap_or(720.0),
        "fontSize": args.get("font_size").and_then(|v| v.as_f64()).unwrap_or(12.0),
        "fontName": args.get("font_name").and_then(|v| v.as_str()).unwrap_or("Helvetica"),
    });

    let _response = client.send_command("addText", options).await?;
    Ok("Text added successfully".to_string())
}

async fn extract_text(client: &Arc<AcrobatClient>, args: Value) -> Result<String> {
    let options = json!({
        "pageRange": args.get("page_range").and_then(|v| v.as_str()).unwrap_or("all"),
    });

    let response = client.send_command("extractText", options).await?;

    if let Some(data) = AcrobatClient::extract_response(&response) {
        Ok(format!("Extracted text:\n{}", data))
    } else {
        Ok("No text extracted".to_string())
    }
}

async fn export_as(client: &Arc<AcrobatClient>, args: Value) -> Result<String> {
    let file_path = args
        .get("file_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing required field: file_path"))?;
    let format = args
        .get("format")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing required field: format"))?;

    let options = json!({
        "filePath": file_path,
        "format": format,
        "quality": args.get("quality").and_then(|v| v.as_i64()).unwrap_or(90),
    });

    let _response = client.send_command("exportAs", options).await?;
    Ok(format!("Exported to: {} ({})", file_path, format))
}

async fn merge_documents(client: &Arc<AcrobatClient>, args: Value) -> Result<String> {
    let file_paths = args
        .get("file_paths")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("Missing required field: file_paths"))?;
    let output_path = args
        .get("output_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing required field: output_path"))?;

    let options = json!({
        "filePaths": file_paths,
        "outputPath": output_path,
    });

    let _response = client.send_command("mergeDocuments", options).await?;
    Ok(format!("Merged {} documents to: {}", file_paths.len(), output_path))
}

async fn split_document(client: &Arc<AcrobatClient>, args: Value) -> Result<String> {
    let page_ranges = args
        .get("page_ranges")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("Missing required field: page_ranges"))?;
    let output_dir = args
        .get("output_dir")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing required field: output_dir"))?;

    let options = json!({
        "pageRanges": page_ranges,
        "outputDir": output_dir,
        "namePattern": args.get("name_pattern").and_then(|v| v.as_str()).unwrap_or("split_{n}.pdf"),
    });

    let _response = client.send_command("splitDocument", options).await?;
    Ok(format!("Split document into {} parts in: {}", page_ranges.len(), output_dir))
}

async fn get_page_count(client: &Arc<AcrobatClient>, _args: Value) -> Result<String> {
    let response = client.send_command("getPageCount", json!({})).await?;

    if let Some(data) = AcrobatClient::extract_response(&response) {
        Ok(format!("Page count: {}", data))
    } else {
        Err(anyhow!("Failed to get page count"))
    }
}

async fn delete_pages(client: &Arc<AcrobatClient>, args: Value) -> Result<String> {
    let page_numbers = args
        .get("page_numbers")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("Missing required field: page_numbers"))?;

    let options = json!({
        "pageNumbers": page_numbers,
    });

    let _response = client.send_command("deletePages", options).await?;
    Ok(format!("Deleted {} pages", page_numbers.len()))
}

async fn rotate_pages(client: &Arc<AcrobatClient>, args: Value) -> Result<String> {
    let page_numbers = args
        .get("page_numbers")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("Missing required field: page_numbers"))?;
    let angle = args
        .get("angle")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| anyhow!("Missing required field: angle"))?;

    let options = json!({
        "pageNumbers": page_numbers,
        "angle": angle,
    });

    let _response = client.send_command("rotatePages", options).await?;
    Ok(format!("Rotated {} pages by {} degrees", page_numbers.len(), angle))
}

async fn add_bookmark(client: &Arc<AcrobatClient>, args: Value) -> Result<String> {
    let title = args
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing required field: title"))?;
    let page = args
        .get("page")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| anyhow!("Missing required field: page"))?;

    let options = json!({
        "title": title,
        "page": page,
        "parent": args.get("parent").and_then(|v| v.as_str()),
    });

    let _response = client.send_command("addBookmark", options).await?;
    Ok(format!("Added bookmark '{}' at page {}", title, page))
}

async fn set_metadata(client: &Arc<AcrobatClient>, args: Value) -> Result<String> {
    let options = json!({
        "title": args.get("title").and_then(|v| v.as_str()),
        "author": args.get("author").and_then(|v| v.as_str()),
        "subject": args.get("subject").and_then(|v| v.as_str()),
        "keywords": args.get("keywords").and_then(|v| v.as_str()),
    });

    let _response = client.send_command("setMetadata", options).await?;
    Ok("Metadata updated successfully".to_string())
}
