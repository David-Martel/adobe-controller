# Adobe Proxy - WebSocket Server

Unified WebSocket proxy server for Adobe MCP applications, written in Rust. This server replaces the Node.js `proxy-server` with a high-performance, type-safe implementation while maintaining full backward compatibility with existing UXP plugins.

## Overview

The proxy server acts as a message broker between MCP servers and Adobe Creative Cloud applications (Photoshop, Illustrator, InDesign, Premiere Pro). It uses Socket.IO-compatible protocol over WebSocket for communication.

## Architecture

```
MCP Server <---> Proxy Server <---> Adobe UXP Plugin
                 (port 3001)
```

### Key Components

- **WebSocket Server**: Axum-based server with Socket.IO protocol support
- **Client Registry**: DashMap-based concurrent client tracking
- **Application Routing**: Routes commands to specific Adobe applications
- **Broadcast Channels**: Tokio broadcast for efficient message distribution

## Protocol Compatibility

The server implements Socket.IO protocol compatibility to work with existing UXP plugins without modification:

### Socket.IO Packet Format

```
42["event_name",{data}]
```

- `4` = Engine.IO message packet
- `2` = Socket.IO EVENT packet
- Ping/Pong: `2` (ping) → `3` (pong)

### Supported Events

#### From UXP Plugins (Adobe Applications)

1. **register** - Register application client
   ```json
   ["register", {"application": "photoshop"}]
   ```
   Response:
   ```json
   ["registration_response", {
     "type": "registration",
     "status": "success",
     "message": "Registered for photoshop"
   }]
   ```

2. **command_packet_response** - Send command result back to MCP server
   ```json
   ["command_packet_response", {
     "packet": {
       "senderId": "uuid-of-mcp-server",
       "result": {...}
     }
   }]
   ```

#### From MCP Servers

1. **command_packet** - Send command to Adobe application
   ```json
   ["command_packet", {
     "application": "photoshop",
     "command": {
       "action": "createLayer",
       "params": {...}
     }
   }]
   ```

#### To MCP Servers

1. **packet_response** - Forward application response
   ```json
   ["packet_response", {
     "senderId": "uuid-of-mcp-server",
     "result": {...}
   }]
   ```

## Usage

### Command Line

```bash
# Default (127.0.0.1:3001)
adobe-proxy

# Custom host and port
adobe-proxy --host 0.0.0.0 --port 8080

# Help
adobe-proxy --help
```

### Arguments

- `--host <HOST>` - Host to bind to (default: `127.0.0.1`)
- `--port <PORT>` - Port to listen on (default: `3001`)

### Status Endpoint

HTTP GET endpoint for server health and metrics:

```bash
curl http://localhost:3001/status
```

Response:
```json
{
  "status": "running",
  "port": 3001,
  "clients": {
    "photoshop": 2,
    "illustrator": 1
  },
  "uptime": 3600
}
```

## Building

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run with logging
RUST_LOG=info cargo run
```

## Message Flow Examples

### 1. Application Registration

```
UXP Plugin → Proxy
42["register",{"application":"photoshop"}]

Proxy → UXP Plugin
42["registration_response",{"type":"registration","status":"success","message":"Registered for photoshop"}]
```

### 2. Command Execution

```
MCP Server → Proxy
42["command_packet",{"application":"photoshop","command":{"action":"createLayer"}}]

Proxy → UXP Plugin (photoshop)
42["command_packet",{"senderId":"abc-123","application":"photoshop","command":{"action":"createLayer"}}]

UXP Plugin → Proxy
42["command_packet_response",{"packet":{"senderId":"abc-123","result":{"layerId":5}}}]

Proxy → MCP Server (abc-123)
42["packet_response",{"senderId":"abc-123","result":{"layerId":5}}]
```

## Implementation Details

### Concurrency Model

- **Tokio Runtime**: Async I/O for WebSocket connections
- **DashMap**: Lock-free concurrent hashmap for client tracking
- **Broadcast Channels**: Efficient one-to-many message distribution
- **Task Spawning**: Independent send/receive tasks per connection

### Client Lifecycle

1. **Connect**: WebSocket upgrade, generate UUID, send Socket.IO handshake
2. **Register**: Optional registration for specific application
3. **Active**: Send/receive commands
4. **Disconnect**: Cleanup from client registry and application routing

### Error Handling

- Connection errors logged but don't crash server
- Invalid messages logged as warnings
- Missing sender IDs in responses handled gracefully
- Automatic cleanup on client disconnect

## Performance Characteristics

- **Zero-copy message passing** where possible
- **Concurrent client handling** via async tasks
- **Lock-free client tracking** with DashMap
- **Efficient broadcast** to multiple application clients

## Differences from Node.js Version

### Improvements

1. **Type Safety**: Compile-time guarantees for protocol messages
2. **Memory Safety**: No null/undefined errors, automatic cleanup
3. **Performance**: Lower latency, better throughput
4. **Concurrency**: True parallelism for multi-core utilization
5. **Resource Usage**: Lower memory footprint

### Maintained Compatibility

- Same Socket.IO protocol
- Same event names and structure
- Same port (3001)
- Same status endpoint format

## Dependencies

- `axum` - Web framework and WebSocket support
- `tokio` - Async runtime
- `dashmap` - Concurrent hashmap
- `serde/serde_json` - JSON serialization
- `uuid` - Unique client identifiers
- `tracing` - Structured logging
- `clap` - CLI argument parsing

## Testing

### Manual Testing

1. Start the proxy:
   ```bash
   cargo run --release
   ```

2. Connect with a WebSocket client (e.g., wscat):
   ```bash
   wscat -c ws://localhost:3001/socket.io/
   ```

3. Send registration:
   ```
   42["register",{"application":"photoshop"}]
   ```

4. Check status:
   ```bash
   curl http://localhost:3001/status
   ```

### Integration Testing

The proxy is designed to work transparently with existing UXP plugins. No changes are required to the plugin code.

## Logging

Set `RUST_LOG` environment variable for logging control:

```bash
# Info level (recommended)
RUST_LOG=info adobe-proxy

# Debug level (verbose)
RUST_LOG=debug adobe-proxy

# Trace level (very verbose)
RUST_LOG=trace adobe-proxy
```

## License

MIT License - See LICENSE file for details

## Author

Implementation based on the Node.js proxy-server by Mike Chambers.
