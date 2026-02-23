# Integration Guide

This guide explains how to migrate from the Node.js proxy-server to the Rust adobe-proxy.

## Quick Start

### 1. Build the Proxy

```bash
cd T:/projects/mcp_servers/adobe-controller/adobe-mcp-rs/adobe-proxy
cargo build --release
```

Binary location: `target/release/adobe-proxy.exe`

### 2. Start the Server

```bash
# Using cargo
cargo run --release

# Or run the binary directly
./target/release/adobe-proxy.exe
```

### 3. Verify it's Running

```bash
curl http://localhost:3001/status
```

Expected output:
```json
{
  "status": "running",
  "port": 3001,
  "clients": {},
  "uptime": 5
}
```

## Migration from Node.js

### What Changes

- **Runtime**: Node.js → Rust binary
- **Performance**: Improved latency and throughput
- **Memory**: Lower memory footprint

### What Stays the Same

- **Protocol**: Socket.IO WebSocket (no changes)
- **Port**: 3001 (default)
- **Events**: Same event names and data structures
- **UXP Plugins**: No changes required

### Side-by-Side Comparison

| Feature | Node.js (proxy.js) | Rust (adobe-proxy) |
|---------|-------------------|-------------------|
| Port | 3001 | 3001 |
| Protocol | Socket.IO | Socket.IO-compatible |
| Events | register, command_packet, command_packet_response | Same |
| Status endpoint | /status | /status |
| Max buffer size | 50 MB | Unlimited (streaming) |
| Concurrency | Event loop | Multi-threaded async |
| Memory | ~50 MB | ~5-10 MB |
| Startup time | ~500 ms | ~50 ms |

## Testing with Existing UXP Plugins

The Rust proxy is designed for drop-in replacement. No changes to UXP plugin code required.

### Test with Photoshop Plugin

1. Start the Rust proxy:
   ```bash
   adobe-proxy
   ```

2. Launch Photoshop and load your UXP plugin

3. The plugin should connect automatically to `ws://localhost:3001/socket.io/`

4. Check the proxy logs for connection messages:
   ```
   INFO User connected: <uuid>
   INFO Client <uuid> registered for application: photoshop
   ```

5. Verify in the status endpoint:
   ```bash
   curl http://localhost:3001/status
   ```

   Should show:
   ```json
   {
     "status": "running",
     "port": 3001,
     "clients": {
       "photoshop": 1
     },
     "uptime": 30
   }
   ```

## Debugging Connection Issues

### Enable Debug Logging

```bash
RUST_LOG=debug adobe-proxy
```

### Common Issues

1. **Port already in use**
   - Solution: Stop the Node.js proxy first or use `--port` to change port
   ```bash
   adobe-proxy --port 3002
   ```

2. **No clients connecting**
   - Check UXP plugin is configured to connect to correct URL
   - Verify firewall allows connections to port 3001
   - Check proxy logs for connection attempts

3. **Messages not routing**
   - Enable trace logging: `RUST_LOG=trace adobe-proxy`
   - Check application name matches in register event
   - Verify senderId is preserved in command_packet_response

## Performance Tuning

### For High-Volume Scenarios

1. **Increase file descriptor limit** (Linux/macOS):
   ```bash
   ulimit -n 4096
   ```

2. **Use release build** (optimized):
   ```bash
   cargo build --release
   ```

3. **Bind to specific interface**:
   ```bash
   adobe-proxy --host 0.0.0.0  # All interfaces
   ```

### Monitoring

Check status endpoint periodically:

```bash
watch -n 5 'curl -s http://localhost:3001/status | jq'
```

## Production Deployment

### As a System Service (Windows)

Create `adobe-proxy.xml` for NSSM or Windows Service:

```xml
<service>
  <id>adobe-proxy</id>
  <name>Adobe MCP Proxy</name>
  <description>WebSocket proxy for Adobe MCP applications</description>
  <executable>T:\projects\mcp_servers\adobe-controller\adobe-mcp-rs\target\release\adobe-proxy.exe</executable>
  <arguments>--host 127.0.0.1 --port 3001</arguments>
  <logpath>C:\logs\adobe-proxy</logpath>
  <logmode>rotate</logmode>
</service>
```

Install with NSSM:
```powershell
nssm install AdobeProxy "T:\projects\mcp_servers\adobe-controller\adobe-mcp-rs\target\release\adobe-proxy.exe" "--host 127.0.0.1 --port 3001"
nssm start AdobeProxy
```

### As a System Service (Linux/macOS)

Create systemd service file `/etc/systemd/system/adobe-proxy.service`:

```ini
[Unit]
Description=Adobe MCP Proxy Server
After=network.target

[Service]
Type=simple
User=youruser
WorkingDirectory=/opt/adobe-proxy
ExecStart=/opt/adobe-proxy/adobe-proxy --host 127.0.0.1 --port 3001
Restart=always
RestartSec=10
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl enable adobe-proxy
sudo systemctl start adobe-proxy
sudo systemctl status adobe-proxy
```

## Health Checks

### HTTP Health Check

```bash
curl http://localhost:3001/status
```

Returns:
- HTTP 200 OK if server is running
- JSON with server metrics

### WebSocket Health Check

Using wscat:
```bash
wscat -c ws://localhost:3001/socket.io/
```

Should receive Socket.IO handshake:
```
< 0{"sid":"<uuid>"}
```

### Automated Monitoring

Example health check script:

```bash
#!/bin/bash
STATUS=$(curl -s http://localhost:3001/status | jq -r '.status')
if [ "$STATUS" != "running" ]; then
  echo "Proxy server is not running!"
  exit 1
fi
echo "Proxy server is healthy"
exit 0
```

## Rollback to Node.js

If issues occur, rollback is simple:

1. Stop the Rust proxy
2. Start the Node.js proxy:
   ```bash
   cd T:/projects/mcp_servers/adobe-controller/adobe-mcp-unified/proxy-server
   node proxy.js
   ```

No changes to UXP plugins required - they will reconnect automatically.

## Support

### Logs

All logs go to stdout/stderr. Redirect for persistent logging:

```bash
adobe-proxy 2>&1 | tee proxy.log
```

### Log Levels

- `error` - Only errors
- `warn` - Warnings and errors
- `info` - Normal operation (recommended)
- `debug` - Detailed debugging
- `trace` - Very verbose

Set with `RUST_LOG` environment variable:
```bash
RUST_LOG=info adobe-proxy
```

### Getting Help

1. Check logs at debug level
2. Verify status endpoint shows connected clients
3. Test with wscat to isolate WebSocket issues
4. Compare protocol messages with Node.js version
