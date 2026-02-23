# Acrobat MCP Server

Model Context Protocol (MCP) server for Adobe Acrobat automation. Provides AI assistants with programmatic control over PDF documents through a WebSocket proxy connection.

## Features

- **Document Management**: Create, open, save, and close PDF documents
- **Content Manipulation**: Add text, extract content, manage pages
- **Document Operations**: Merge, split, rotate, and delete pages
- **Metadata & Navigation**: Set document properties, add bookmarks
- **Export**: Convert PDFs to various formats (PNG, JPEG, DOCX, PPTX)
- **Zero-copy JSON-RPC**: Efficient MCP protocol implementation over stdio

## Architecture

```
┌─────────────┐         ┌──────────────┐         ┌────────────────┐
│  MCP Client │ stdio   │ acrobat-mcp  │ WebSocket│  Proxy Server  │
│  (Claude)   │◄───────►│   (Rust)     │◄────────►│   (Node.js)    │
└─────────────┘         └──────────────┘         └────────────────┘
                                                          │
                                                    ExtendScript
                                                          ▼
                                                  ┌────────────────┐
                                                  │ Adobe Acrobat  │
                                                  │   (Windows)    │
                                                  └────────────────┘
```

## Building

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Binary location (custom RustCache configuration)
# T:\RustCache\cargo-target\release\acrobat-mcp.exe
```

## Usage

### Command Line

```bash
# Default proxy URL (ws://localhost:3001)
acrobat-mcp

# Custom proxy URL
acrobat-mcp --proxy-url ws://localhost:8080

# Custom timeout (milliseconds)
acrobat-mcp --timeout 60000

# Environment variables
ACROBAT_PROXY_URL=ws://localhost:3001 acrobat-mcp
ACROBAT_TIMEOUT=30000 acrobat-mcp
```

### MCP Configuration

Add to your `mcp.json`:

```json
{
  "mcpServers": {
    "acrobat": {
      "command": "T:/RustCache/cargo-target/release/acrobat-mcp.exe",
      "args": ["--proxy-url", "ws://localhost:3001"],
      "env": {
        "ACROBAT_TIMEOUT": "30000"
      }
    }
  }
}
```

## Available Tools

### Document Management

#### `create_document`
Create a new PDF document with specified size and page count.

**Parameters:**
- `name` (required): Document name
- `page_size` (optional): Page size preset (LETTER, LEGAL, A4, A3, CUSTOM)
- `page_count` (optional): Number of pages (default: 1)
- `width` (optional): Custom width in points (for CUSTOM page_size)
- `height` (optional): Custom height in points (for CUSTOM page_size)

**Example:**
```json
{
  "name": "create_document",
  "arguments": {
    "name": "Report.pdf",
    "page_size": "LETTER",
    "page_count": 5
  }
}
```

#### `open_document`
Open an existing PDF document.

**Parameters:**
- `file_path` (required): Absolute path to PDF file

**Example:**
```json
{
  "name": "open_document",
  "arguments": {
    "file_path": "C:/Documents/report.pdf"
  }
}
```

#### `save_document`
Save the current document.

**Parameters:**
- `file_path` (required): Path to save the document
- `format` (optional): Save format (PDF, PDF_A, PDF_X)

#### `close_document`
Close the currently active document.

**Parameters:**
- `save_changes` (optional): Save changes before closing (default: false)

#### `get_document_info`
Get information about the current document (name, path, page count, etc.)

### Content Manipulation

#### `add_text`
Add text to a specific page.

**Parameters:**
- `text` (required): Text content to add
- `page` (optional): Page number (1-based, default: 1)
- `x` (optional): X coordinate in points (default: 72)
- `y` (optional): Y coordinate in points (default: 720)
- `font_size` (optional): Font size in points (default: 12)
- `font_name` (optional): Font name (default: "Helvetica")

**Example:**
```json
{
  "name": "add_text",
  "arguments": {
    "text": "Hello, World!",
    "page": 1,
    "x": 100,
    "y": 700,
    "font_size": 14,
    "font_name": "Arial"
  }
}
```

#### `extract_text`
Extract text from specified page range.

**Parameters:**
- `page_range` (optional): Page range (e.g., "1-5", "all")

### Page Operations

#### `get_page_count`
Get the number of pages in the current document.

#### `delete_pages`
Delete specified pages from the document.

**Parameters:**
- `page_numbers` (required): Array of page numbers to delete (1-based)

#### `rotate_pages`
Rotate specified pages by angle.

**Parameters:**
- `page_numbers` (required): Array of page numbers to rotate (1-based)
- `angle` (required): Rotation angle in degrees (90, 180, 270)

### Document Operations

#### `merge_documents`
Merge multiple PDF documents into one.

**Parameters:**
- `file_paths` (required): Array of PDF file paths to merge
- `output_path` (required): Output file path for merged PDF

**Example:**
```json
{
  "name": "merge_documents",
  "arguments": {
    "file_paths": ["C:/docs/part1.pdf", "C:/docs/part2.pdf"],
    "output_path": "C:/docs/merged.pdf"
  }
}
```

#### `split_document`
Split document into multiple PDFs by page ranges.

**Parameters:**
- `page_ranges` (required): Array of page ranges (e.g., ["1-3", "4-6"])
- `output_dir` (required): Output directory for split PDFs
- `name_pattern` (optional): Filename pattern (default: "split_{n}.pdf")

### Export & Conversion

#### `export_as`
Export document to different format.

**Parameters:**
- `file_path` (required): Output file path
- `format` (required): Export format (PDF, PNG, JPEG, TIFF, DOCX, PPTX)
- `quality` (optional): Quality for image formats 1-100 (default: 90)

**Example:**
```json
{
  "name": "export_as",
  "arguments": {
    "file_path": "C:/exports/image.png",
    "format": "PNG",
    "quality": 95
  }
}
```

### Metadata & Navigation

#### `add_bookmark`
Add a bookmark to a specific page.

**Parameters:**
- `title` (required): Bookmark title
- `page` (required): Target page number (1-based)
- `parent` (optional): Parent bookmark title

#### `set_metadata`
Set document metadata.

**Parameters:**
- `title` (optional): Document title
- `author` (optional): Document author
- `subject` (optional): Document subject
- `keywords` (optional): Document keywords

## Error Handling

The server returns errors in the standard MCP format:

```json
{
  "content": [
    {
      "type": "text",
      "text": "Error: Command timeout after 30000ms"
    }
  ],
  "isError": true
}
```

Common errors:
- **Connection failed**: Proxy server not running or unreachable
- **Command timeout**: Acrobat didn't respond within timeout period
- **Command failed**: Acrobat returned an error (e.g., file not found)
- **Protocol error**: Invalid WebSocket message format
- **Invalid params**: Missing or invalid tool parameters

## Performance

- **Binary size**: ~2MB (release build)
- **Memory footprint**: ~5MB baseline + WebSocket buffer
- **Startup time**: <100ms
- **Command latency**: 50-500ms (network + Acrobat processing)

## Logging

Logs are written to stderr (not stdout, to avoid corrupting JSON-RPC):

```bash
# View logs in real-time
acrobat-mcp 2>&1 | tee acrobat-mcp.log

# Set log level
RUST_LOG=debug acrobat-mcp
```

## Dependencies

- `tokio`: Async runtime
- `tokio-tungstenite`: WebSocket client
- `serde_json`: JSON serialization
- `adobe-common`: Shared Adobe MCP types
- `clap`: CLI argument parsing
- `tracing`: Structured logging
- `anyhow`: Error handling

## License

Same as parent project (see root LICENSE).

## Related Projects

- `adobe-proxy`: WebSocket proxy server (Node.js)
- `acrobat-bridge`: ExtendScript bridge plugin
- `adobe-common`: Shared Rust types and protocols
