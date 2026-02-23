//! Command handlers for Acrobat operations
//!
//! Each command maps to an Acrobat JavaScript API call.
//! Commands are executed via the js_bridge module.

use crate::js_bridge;
use adobe_common::{Command, CommandResponse, ResponseStatus};
use anyhow::Result;
use serde_json::{json, Value};

/// Execute a command and return the response
///
/// # Errors
/// Returns error if command execution fails
pub fn execute_command(command: &Command) -> Result<CommandResponse> {
    let action = command.action.as_str();
    let options = &command.options;

    tracing::info!("Executing command: {} with options: {:?}", action, options);

    let result = match action {
        // Document operations
        "createDocument" => create_document(options),
        "openDocument" => open_document(options),
        "saveDocument" => save_document(options),
        "closeDocument" => close_document(options),
        "getDocumentInfo" => get_document_info(options),

        // Text operations
        "addText" => add_text(options),
        "extractText" => extract_text(options),

        // Export operations
        "exportAs" => export_as(options),

        // Multi-document operations
        "mergeDocuments" => merge_documents(options),
        "splitDocument" => split_document(options),

        // Page operations
        "getPageCount" => get_page_count(options),
        "deletePages" => delete_pages(options),
        "rotatePages" => rotate_pages(options),
        "insertPages" => insert_pages(options),
        "addBookmark" => add_bookmark(options),
        "setMetadata" => set_metadata(options),

        // Unknown command
        _ => Err(anyhow::anyhow!("Unknown command: {}", action)),
    };

    match result {
        Ok(response) => Ok(CommandResponse {
            sender_id: String::new(), // Will be filled by caller
            status: ResponseStatus::Success,
            response: Some(response),
            message: None,
            document: None,
        }),
        Err(e) => Ok(CommandResponse {
            sender_id: String::new(),
            status: ResponseStatus::Failure,
            response: None,
            message: Some(e.to_string()),
            document: None,
        }),
    }
}

// ============================================================================
// Document Operations
// ============================================================================

fn create_document(options: &Value) -> Result<Value> {
    let name = options
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled");
    let page_count = options
        .get("pageCount")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);
    let page_size = options
        .get("pageSize")
        .and_then(|v| v.as_str())
        .unwrap_or("LETTER");
    let custom_width = options.get("width").and_then(|v| v.as_f64());
    let custom_height = options.get("height").and_then(|v| v.as_f64());
    let (page_width, page_height) = if page_size.eq_ignore_ascii_case("CUSTOM") {
        (
            custom_width.unwrap_or_else(|| page_size_width("LETTER")),
            custom_height.unwrap_or_else(|| page_size_height("LETTER")),
        )
    } else {
        (page_size_width(page_size), page_size_height(page_size))
    };

    // Generate JavaScript to create document
    let js = format!(
        r#"
        (function() {{
            var doc = app.newDoc({{
                nWidth: {},
                nHeight: {}
            }});
            if (doc) {{
                doc.info.Title = "{}";
                // Add additional pages if needed
                for (var i = 1; i < {}; i++) {{
                    doc.newPage();
                }}
                return JSON.stringify({{
                    "success": true,
                    "documentName": doc.info.Title,
                    "pageCount": doc.numPages
                }});
            }}
            return JSON.stringify({{"success": false, "error": "Failed to create document"}});
        }})()
        "#,
        page_width,
        page_height,
        escape_js_string(name),
        page_count
    );

    execute_js_and_parse(&js, || {
        json!({
            "status": "ok",
            "documentName": name,
            "pageCount": page_count
        })
    })
}

fn open_document(options: &Value) -> Result<Value> {
    let file_path = options
        .get("filePath")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("filePath required"))?;

    let js = format!(
        r#"
        (function() {{
            try {{
                var doc = app.openDoc("{}");
                if (doc) {{
                    return JSON.stringify({{
                        "success": true,
                        "path": doc.path,
                        "title": doc.info.Title || "",
                        "numPages": doc.numPages
                    }});
                }}
                return JSON.stringify({{"success": false, "error": "Failed to open document"}});
            }} catch(e) {{
                return JSON.stringify({{"success": false, "error": e.toString()}});
            }}
        }})()
        "#,
        escape_js_path(file_path)
    );

    execute_js_and_parse(&js, || {
        json!({
            "status": "ok",
            "filePath": file_path
        })
    })
}

fn save_document(options: &Value) -> Result<Value> {
    let file_path = options.get("filePath").and_then(|v| v.as_str());

    let js = if let Some(path) = file_path {
        format!(
            r#"
            (function() {{
                try {{
                    var doc = this;
                    doc.saveAs("{}");
                    return JSON.stringify({{"success": true}});
                }} catch(e) {{
                    return JSON.stringify({{"success": false, "error": e.toString()}});
                }}
            }})()
            "#,
            escape_js_path(path)
        )
    } else {
        r#"
        (function() {
            try {
                var doc = this;
                doc.save();
                return JSON.stringify({"success": true});
            } catch(e) {
                return JSON.stringify({"success": false, "error": e.toString()});
            }
        })()
        "#
        .to_string()
    };

    execute_js_and_parse(&js, || json!({"status": "ok"}))
}

fn close_document(options: &Value) -> Result<Value> {
    let save_changes = options
        .get("saveChanges")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let js = format!(
        r#"
        (function() {{
            try {{
                var doc = this;
                var saveChanges = {};
                doc.closeDoc(!saveChanges);
                return JSON.stringify({{"success": true}});
            }} catch(e) {{
                return JSON.stringify({{"success": false, "error": e.toString()}});
            }}
        }})()
        "#,
        save_changes
    );

    execute_js_and_parse(&js, || json!({"status": "ok"}))
}

fn get_document_info(_options: &Value) -> Result<Value> {
    let js = r#"
        (function() {
            try {
                var doc = this;
                var box = doc.getPageBox("Crop", 0);
                return JSON.stringify({
                    "success": true,
                    "title": doc.info.Title || "",
                    "author": doc.info.Author || "",
                    "subject": doc.info.Subject || "",
                    "keywords": doc.info.Keywords || "",
                    "creator": doc.info.Creator || "",
                    "producer": doc.info.Producer || "",
                    "numPages": doc.numPages,
                    "pageSize": {
                        "width": box[2] - box[0],
                        "height": box[3] - box[1]
                    },
                    "path": doc.path
                });
            } catch(e) {
                return JSON.stringify({"success": false, "error": e.toString()});
            }
        })()
    "#;

    execute_js_and_parse(js, || {
        json!({
            "title": "Document",
            "numPages": 1,
            "path": ""
        })
    })
}

// ============================================================================
// Text Operations
// ============================================================================

fn add_text(options: &Value) -> Result<Value> {
    let page = options.get("page").and_then(|v| v.as_i64()).unwrap_or(1);
    let page_index = normalize_page_index(page);
    let text = options.get("text").and_then(|v| v.as_str()).unwrap_or("");
    let x = options.get("x").and_then(|v| v.as_f64()).unwrap_or(72.0);
    let y = options.get("y").and_then(|v| v.as_f64()).unwrap_or(720.0);
    let font_size = options
        .get("fontSize")
        .and_then(|v| v.as_f64())
        .unwrap_or(12.0);
    let font_name = options
        .get("fontName")
        .and_then(|v| v.as_str())
        .unwrap_or("Helvetica");

    let js = format!(
        r#"
        (function() {{
            try {{
                var doc = this;
                var annot = doc.addAnnot({{
                    page: {},
                    type: "FreeText",
                    rect: [{}, {}, {}, {}],
                    contents: "{}",
                    textFont: "{}",
                    textSize: {}
                }});
                return JSON.stringify({{"success": annot != null, "page": {}}});
            }} catch(e) {{
                return JSON.stringify({{"success": false, "error": e.toString()}});
            }}
        }})()
        "#,
        page_index,
        x,
        y,
        x + 200.0,
        y + font_size * 1.5,
        escape_js_string(text),
        escape_js_string(font_name),
        font_size,
        page_index + 1
    );

    execute_js_and_parse(&js, || json!({"status": "ok", "page": page_index + 1}))
}

fn extract_text(options: &Value) -> Result<Value> {
    let (page_start, page_end) = parse_page_range(options);

    let js = format!(
        r#"
        (function() {{
            try {{
                var doc = this;
                var start = {};
                var end = {};
                if (start < 0) {{
                    start = 0;
                }}
                if (end < 0) {{
                    end = doc.numPages - 1;
                }}
                if (end < start) {{
                    end = start;
                }}
                var text = "";
                for (var i = start; i <= end && i < doc.numPages; i++) {{
                    for (var j = 0; j < doc.getPageNumWords(i); j++) {{
                        text += doc.getPageNthWord(i, j) + " ";
                    }}
                    text += "\n";
                }}
                return JSON.stringify({{"success": true, "text": text}});
            }} catch(e) {{
                return JSON.stringify({{"success": false, "error": e.toString()}});
            }}
        }})()
        "#,
        page_start,
        page_end
    );

    execute_js_and_parse(&js, || json!({"status": "ok", "text": ""}))
}

// ============================================================================
// Export Operations
// ============================================================================

fn export_as(options: &Value) -> Result<Value> {
    let file_path = options
        .get("filePath")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("filePath required"))?;
    let format = options
        .get("format")
        .and_then(|v| v.as_str())
        .unwrap_or("PDF");

    let conversion_id = match format.to_uppercase().as_str() {
        "DOCX" | "DOC" => "com.adobe.acrobat.docx",
        "XLSX" | "XLS" => "com.adobe.acrobat.xlsx",
        "PPTX" | "PPT" => "com.adobe.acrobat.pptx",
        "RTF" => "com.adobe.acrobat.rtf",
        "TXT" => "com.adobe.acrobat.txt",
        "PNG" | "JPEG" | "JPG" | "TIFF" => "com.adobe.acrobat.image",
        _ => "com.adobe.acrobat.pdf",
    };

    let js = format!(
        r#"
        (function() {{
            try {{
                var doc = this;
                doc.saveAs("{}", "{}");
                return JSON.stringify({{"success": true, "filePath": "{}", "format": "{}"}});
            }} catch(e) {{
                return JSON.stringify({{"success": false, "error": e.toString()}});
            }}
        }})()
        "#,
        escape_js_path(file_path),
        conversion_id,
        escape_js_path(file_path),
        format
    );

    execute_js_and_parse(&js, || {
        json!({
            "status": "ok",
            "filePath": file_path,
            "format": format
        })
    })
}

// ============================================================================
// Multi-document Operations
// ============================================================================

fn merge_documents(options: &Value) -> Result<Value> {
    let file_paths = options
        .get("filePaths")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("filePaths array required"))?;
    let output_path = options
        .get("outputPath")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("outputPath required"))?;

    let paths: Vec<&str> = file_paths.iter().filter_map(|v| v.as_str()).collect();

    if paths.is_empty() {
        return Err(anyhow::anyhow!("At least one file path required"));
    }

    let paths_json = serde_json::to_string(&paths)?;

    let js = format!(
        r#"
        (function() {{
            try {{
                var paths = {};
                if (paths.length === 0) {{
                    return JSON.stringify({{"success": false, "error": "No paths provided"}});
                }}

                // Open first document as base
                var doc = app.openDoc(paths[0]);
                if (!doc) {{
                    return JSON.stringify({{"success": false, "error": "Failed to open first document"}});
                }}

                // Insert pages from remaining documents
                for (var i = 1; i < paths.length; i++) {{
                    doc.insertPages({{
                        nPage: doc.numPages - 1,
                        cPath: paths[i]
                    }});
                }}

                doc.saveAs("{}");
                var pageCount = doc.numPages;
                doc.closeDoc(true);

                return JSON.stringify({{
                    "success": true,
                    "outputPath": "{}",
                    "mergedCount": paths.length,
                    "totalPages": pageCount
                }});
            }} catch(e) {{
                return JSON.stringify({{"success": false, "error": e.toString()}});
            }}
        }})()
        "#,
        paths_json,
        escape_js_path(output_path),
        escape_js_path(output_path)
    );

    execute_js_and_parse(&js, || {
        json!({
            "status": "ok",
            "outputPath": output_path,
            "mergedCount": paths.len()
        })
    })
}

fn split_document(options: &Value) -> Result<Value> {
    let page_ranges_value = options
        .get("pageRanges")
        .ok_or_else(|| anyhow::anyhow!("pageRanges array required"))?;
    let page_ranges = normalize_page_ranges(page_ranges_value)?;
    let output_dir = options
        .get("outputDir")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("outputDir required"))?;
    let name_pattern = options
        .get("namePattern")
        .and_then(|v| v.as_str())
        .unwrap_or("split_{n}.pdf");

    let ranges_json = serde_json::to_string(&page_ranges)?;

    let js = format!(
        r#"
        (function() {{
            try {{
                var doc = this;
                var ranges = {};
                var outputDir = "{}";
                var namePattern = "{}";
                var outputs = [];

                for (var i = 0; i < ranges.length; i++) {{
                    var range = ranges[i];
                    var start = range.start;
                    var end = range.end;
                    if (start < 0) {{
                        start = 0;
                    }}
                    if (end < 0) {{
                        end = doc.numPages - 1;
                    }}
                    if (end < start) {{
                        end = start;
                    }}

                    var fileName = namePattern.replace("{{n}}", (i + 1).toString());
                    var outputPath = outputDir + "/" + fileName;
                    doc.extractPages({{
                        nStart: start,
                        nEnd: end,
                        cPath: outputPath
                    }});
                    outputs.push(outputPath);
                }}

                return JSON.stringify({{
                    "success": true,
                    "outputDir": outputDir,
                    "splitCount": ranges.length,
                    "outputs": outputs
                }});
            }} catch(e) {{
                return JSON.stringify({{"success": false, "error": e.toString()}});
            }}
        }})()
        "#,
        ranges_json,
        escape_js_path(output_dir),
        escape_js_string(name_pattern)
    );

    execute_js_and_parse(&js, || {
        json!({
            "status": "ok",
            "outputDir": output_dir,
            "splitCount": page_ranges.len()
        })
    })
}

// ============================================================================
// Page Operations
// ============================================================================

fn get_page_count(_options: &Value) -> Result<Value> {
    let js = r#"
        (function() {
            try {
                var doc = this;
                return JSON.stringify({"success": true, "pageCount": doc.numPages});
            } catch(e) {
                return JSON.stringify({"success": false, "error": e.toString()});
            }
        })()
    "#;

    execute_js_and_parse(js, || json!({"status": "ok", "pageCount": 1}))
}

fn delete_pages(options: &Value) -> Result<Value> {
    let pages_value = options
        .get("pages")
        .or_else(|| options.get("pageNumbers"))
        .ok_or_else(|| anyhow::anyhow!("pages array required"))?;
    let page_nums = normalize_page_numbers(pages_value)?;

    if page_nums.is_empty() {
        return Err(anyhow::anyhow!("At least one page number required"));
    }

    let pages_json = serde_json::to_string(&page_nums)?;

    let js = format!(
        r#"
        (function() {{
            try {{
                var doc = this;
                var pages = {};
                // Sort in reverse order to maintain indices
                pages.sort(function(a, b) {{ return b - a; }});

                var deleted = 0;
                for (var i = 0; i < pages.length; i++) {{
                    if (pages[i] < doc.numPages) {{
                        doc.deletePages(pages[i]);
                        deleted++;
                    }}
                }}

                return JSON.stringify({{"success": true, "deletedCount": deleted}});
            }} catch(e) {{
                return JSON.stringify({{"success": false, "error": e.toString()}});
            }}
        }})()
        "#,
        pages_json
    );

    execute_js_and_parse(&js, || {
        json!({
            "status": "ok",
            "deletedCount": page_nums.len()
        })
    })
}

fn rotate_pages(options: &Value) -> Result<Value> {
    let pages_value = options
        .get("pages")
        .or_else(|| options.get("pageNumbers"))
        .ok_or_else(|| anyhow::anyhow!("pages array required"))?;
    let angle = options.get("angle").and_then(|v| v.as_i64()).unwrap_or(90);

    // Validate angle
    if ![0, 90, 180, 270].contains(&angle) {
        return Err(anyhow::anyhow!(
            "Invalid angle: {}. Must be 0, 90, 180, or 270",
            angle
        ));
    }

    let page_nums = normalize_page_numbers(pages_value)?;
    let pages_json = serde_json::to_string(&page_nums)?;

    let js = format!(
        r#"
        (function() {{
            try {{
                var doc = this;
                var pages = {};
                var angle = {};
                var rotated = 0;

                for (var i = 0; i < pages.length; i++) {{
                    if (pages[i] < doc.numPages) {{
                        doc.setPageRotations(pages[i], pages[i], angle);
                        rotated++;
                    }}
                }}

                return JSON.stringify({{"success": true, "rotatedCount": rotated, "angle": angle}});
            }} catch(e) {{
                return JSON.stringify({{"success": false, "error": e.toString()}});
            }}
        }})()
        "#,
        pages_json, angle
    );

    execute_js_and_parse(&js, || {
        json!({
            "status": "ok",
            "rotatedCount": page_nums.len(),
            "angle": angle
        })
    })
}

fn insert_pages(options: &Value) -> Result<Value> {
    let source_path = options
        .get("sourcePath")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("sourcePath required"))?;
    let after_page = options
        .get("afterPage")
        .and_then(|v| v.as_i64())
        .unwrap_or(-1);

    let js = format!(
        r#"
        (function() {{
            try {{
                var doc = this;
                var insertAt = {};
                if (insertAt < 0) {{
                    insertAt = doc.numPages - 1;
                }}

                doc.insertPages({{
                    nPage: insertAt,
                    cPath: "{}"
                }});

                return JSON.stringify({{
                    "success": true,
                    "sourcePath": "{}",
                    "insertedAt": insertAt
                }});
            }} catch(e) {{
                return JSON.stringify({{"success": false, "error": e.toString()}});
            }}
        }})()
        "#,
        after_page,
        escape_js_path(source_path),
        escape_js_path(source_path)
    );

    execute_js_and_parse(&js, || {
        json!({
            "status": "ok",
            "sourcePath": source_path
        })
    })
}

// ============================================================================
// Bookmark & Metadata Operations
// ============================================================================

fn add_bookmark(options: &Value) -> Result<Value> {
    let title = options
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("title required"))?;
    let page = options
        .get("page")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| anyhow::anyhow!("page required"))?;
    let parent = options.get("parent").and_then(|v| v.as_str()).unwrap_or("");
    let page_index = normalize_page_index(page);

    let js = format!(
        r#"
        (function() {{
            try {{
                var doc = this;
                var root = doc.bookmarkRoot;
                var title = "{}";
                var parentName = "{}";
                var pageIndex = {};

                function findBookmark(node, name) {{
                    if (!node) return null;
                    if (node.name === name) return node;
                    if (node.children && node.children.length) {{
                        for (var i = 0; i < node.children.length; i++) {{
                            var found = findBookmark(node.children[i], name);
                            if (found) return found;
                        }}
                    }}
                    return null;
                }}

                var parentNode = root;
                if (parentName) {{
                    var match = findBookmark(root, parentName);
                    if (match) {{
                        parentNode = match;
                    }}
                }}

                var action = "this.pageNum=" + pageIndex;
                var bookmark = parentNode.createChild(title, action);

                return JSON.stringify({{
                    "success": bookmark != null,
                    "title": title,
                    "page": pageIndex + 1,
                    "parent": parentName
                }});
            }} catch(e) {{
                return JSON.stringify({{"success": false, "error": e.toString()}});
            }}
        }})()
        "#,
        escape_js_string(title),
        escape_js_string(parent),
        page_index
    );

    execute_js_and_parse(&js, || {
        json!({
            "status": "ok",
            "title": title,
            "page": page_index + 1
        })
    })
}

fn set_metadata(options: &Value) -> Result<Value> {
    let title = options.get("title").and_then(|v| v.as_str());
    let author = options.get("author").and_then(|v| v.as_str());
    let subject = options.get("subject").and_then(|v| v.as_str());
    let keywords = options.get("keywords").and_then(|v| v.as_str());

    let mut assignments = String::new();
    if let Some(value) = title {
        assignments.push_str(&format!(
            "doc.info.Title = \"{}\";\n",
            escape_js_string(value)
        ));
    }
    if let Some(value) = author {
        assignments.push_str(&format!(
            "doc.info.Author = \"{}\";\n",
            escape_js_string(value)
        ));
    }
    if let Some(value) = subject {
        assignments.push_str(&format!(
            "doc.info.Subject = \"{}\";\n",
            escape_js_string(value)
        ));
    }
    if let Some(value) = keywords {
        assignments.push_str(&format!(
            "doc.info.Keywords = \"{}\";\n",
            escape_js_string(value)
        ));
    }
    if assignments.is_empty() {
        assignments.push_str("// No metadata changes provided\n");
    }

    let js = format!(
        r#"
        (function() {{
            try {{
                var doc = this;
                {}
                return JSON.stringify({{"success": true}});
            }} catch(e) {{
                return JSON.stringify({{"success": false, "error": e.toString()}});
            }}
        }})()
        "#,
        assignments
    );

    execute_js_and_parse(&js, || json!({"status": "ok"}))
}

// ============================================================================
// Helper Functions
// ============================================================================

fn normalize_page_index(value: i64) -> i64 {
    if value <= 0 {
        0
    } else {
        value - 1
    }
}

fn normalize_page_numbers(value: &Value) -> Result<Vec<i64>> {
    let pages = value
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("pages array required"))?;
    let mut page_nums: Vec<i64> = pages.iter().filter_map(|v| v.as_i64()).collect();

    if page_nums.is_empty() {
        return Err(anyhow::anyhow!("At least one page number required"));
    }

    let treat_as_one_based = !page_nums.iter().any(|n| *n == 0);
    if treat_as_one_based {
        page_nums = page_nums
            .into_iter()
            .map(|n| if n <= 0 { 0 } else { n - 1 })
            .collect();
    }

    Ok(page_nums)
}

fn parse_page_range_str(range: &str) -> Option<(i64, i64)> {
    let trimmed = range.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.eq_ignore_ascii_case("all") {
        return Some((0, -1));
    }
    if let Some((start_raw, end_raw)) = trimmed.split_once('-') {
        let start = start_raw.trim().parse::<i64>().ok()?;
        let end = end_raw.trim().parse::<i64>().ok()?;
        let start_idx = normalize_page_index(start);
        let end_idx = normalize_page_index(end);
        return Some((start_idx, end_idx.max(start_idx)));
    }
    if let Ok(value) = trimmed.parse::<i64>() {
        let idx = normalize_page_index(value);
        return Some((idx, idx));
    }
    None
}

fn parse_page_range(options: &Value) -> (i64, i64) {
    if let Some(range) = options.get("pageRange").and_then(|v| v.as_str()) {
        if let Some(parsed) = parse_page_range_str(range) {
            return parsed;
        }
    }

    let start = options.get("pageStart").and_then(|v| v.as_i64()).unwrap_or(0);
    let end = options.get("pageEnd").and_then(|v| v.as_i64()).unwrap_or(-1);

    (normalize_page_index(start), if end < 0 { -1 } else { normalize_page_index(end) })
}

fn normalize_page_ranges(value: &Value) -> Result<Vec<Value>> {
    let ranges = value
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("pageRanges array required"))?;
    let mut normalized = Vec::new();

    for range in ranges {
        if let Some(range_str) = range.as_str() {
            if let Some((start, end)) = parse_page_range_str(range_str) {
                normalized.push(json!({"start": start, "end": end}));
            }
            continue;
        }

        if let Some(range_array) = range.as_array() {
            if range_array.is_empty() {
                continue;
            }
            let start = range_array.get(0).and_then(|v| v.as_i64()).unwrap_or(0);
            let end = range_array
                .get(1)
                .and_then(|v| v.as_i64())
                .unwrap_or(start);
            let start_idx = normalize_page_index(start);
            let end_idx = normalize_page_index(end).max(start_idx);
            normalized.push(json!({"start": start_idx, "end": end_idx}));
            continue;
        }

        if let Some(range_obj) = range.as_object() {
            let start = range_obj
                .get("start")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let end = range_obj
                .get("end")
                .and_then(|v| v.as_i64())
                .unwrap_or(start);
            let start_idx = normalize_page_index(start);
            let end_idx = normalize_page_index(end).max(start_idx);
            normalized.push(json!({"start": start_idx, "end": end_idx}));
        }
    }

    if normalized.is_empty() {
        return Err(anyhow::anyhow!("pageRanges array required"));
    }

    Ok(normalized)
}

/// Execute JavaScript and parse the result, falling back to default on error
fn execute_js_and_parse<F>(script: &str, default_fn: F) -> Result<Value>
where
    F: FnOnce() -> Value,
{
    match js_bridge::execute_js(script) {
        Ok(result) => {
            if result.success {
                if let Some(value) = result.value {
                    // Try to parse as JSON
                    match serde_json::from_str::<Value>(&value) {
                        Ok(parsed) => {
                            // Check if the JS reported success
                            if parsed
                                .get("success")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(true)
                            {
                                Ok(parsed)
                            } else {
                                let error = parsed
                                    .get("error")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("Unknown error");
                                Err(anyhow::anyhow!("JavaScript error: {}", error))
                            }
                        }
                        Err(_) => {
                            // Return raw value as string
                            Ok(json!({"result": value}))
                        }
                    }
                } else {
                    Ok(default_fn())
                }
            } else {
                Err(anyhow::anyhow!(
                    "JavaScript execution failed: {}",
                    result.error.unwrap_or_default()
                ))
            }
        }
        Err(e) => {
            tracing::warn!("JS execution error, using default: {}", e);
            Ok(default_fn())
        }
    }
}

/// Escape a string for use in JavaScript
fn escape_js_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Escape a file path for use in JavaScript
fn escape_js_path(path: &str) -> String {
    // Convert backslashes to forward slashes for cross-platform compatibility
    path.replace('\\', "/")
}

/// Get page width for a named page size
fn page_size_width(size: &str) -> f64 {
    match size.to_uppercase().as_str() {
        "LETTER" => 612.0,
        "LEGAL" => 612.0,
        "A4" => 595.0,
        "A3" => 842.0,
        "TABLOID" => 792.0,
        _ => 612.0, // Default to letter
    }
}

/// Get page height for a named page size
fn page_size_height(size: &str) -> f64 {
    match size.to_uppercase().as_str() {
        "LETTER" => 792.0,
        "LEGAL" => 1008.0,
        "A4" => 842.0,
        "A3" => 1191.0,
        "TABLOID" => 1224.0,
        _ => 792.0, // Default to letter
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_js_string() {
        assert_eq!(escape_js_string("hello"), "hello");
        assert_eq!(escape_js_string("hello\"world"), "hello\\\"world");
        assert_eq!(escape_js_string("line1\nline2"), "line1\\nline2");
        assert_eq!(escape_js_string("path\\to\\file"), "path\\\\to\\\\file");
    }

    #[test]
    fn test_escape_js_path() {
        assert_eq!(escape_js_path("C:\\Users\\test"), "C:/Users/test");
        assert_eq!(escape_js_path("/home/test"), "/home/test");
    }

    #[test]
    fn test_page_sizes() {
        assert_eq!(page_size_width("LETTER"), 612.0);
        assert_eq!(page_size_height("LETTER"), 792.0);
        assert_eq!(page_size_width("A4"), 595.0);
        assert_eq!(page_size_height("A4"), 842.0);
    }

    #[test]
    fn test_execute_command_unknown() {
        let cmd = Command {
            action: "unknownCommand".to_string(),
            options: json!({}),
        };
        let result = execute_command(&cmd).unwrap();
        assert_eq!(result.status, ResponseStatus::Failure);
        assert!(result.message.is_some());
    }

    #[test]
    fn test_execute_command_get_page_count() {
        let cmd = Command {
            action: "getPageCount".to_string(),
            options: json!({}),
        };
        let result = execute_command(&cmd).unwrap();
        assert_eq!(result.status, ResponseStatus::Success);
    }

    #[test]
    fn test_open_document_missing_path() {
        let cmd = Command {
            action: "openDocument".to_string(),
            options: json!({}),
        };
        let result = execute_command(&cmd).unwrap();
        assert_eq!(result.status, ResponseStatus::Failure);
        assert!(result.message.unwrap().contains("filePath required"));
    }

    #[test]
    fn test_rotate_pages_invalid_angle() {
        let cmd = Command {
            action: "rotatePages".to_string(),
            options: json!({"pages": [0], "angle": 45}),
        };
        let result = execute_command(&cmd).unwrap();
        assert_eq!(result.status, ResponseStatus::Failure);
        assert!(result.message.unwrap().contains("Invalid angle"));
    }

    #[test]
    fn test_delete_pages_empty() {
        let cmd = Command {
            action: "deletePages".to_string(),
            options: json!({"pages": []}),
        };
        let result = execute_command(&cmd).unwrap();
        assert_eq!(result.status, ResponseStatus::Failure);
    }
}
