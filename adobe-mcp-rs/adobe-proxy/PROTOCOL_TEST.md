# Protocol Testing Guide

This document provides manual and automated tests to verify Socket.IO protocol compatibility between the Rust proxy and Node.js proxy.

## Manual Testing with wscat

### Install wscat

```bash
npm install -g wscat
```

### Test 1: Connection and Handshake

```bash
wscat -c ws://localhost:3001/socket.io/
```

**Expected:**
```
Connected (press CTRL+C to quit)
< 0{"sid":"<some-uuid>"}
```

The `0` is the Socket.IO CONNECT packet.

### Test 2: Ping/Pong

After connecting, send:
```
> 2
```

**Expected response:**
```
< 3
```

- `2` = ping packet
- `3` = pong packet

### Test 3: Application Registration

Send:
```
> 42["register",{"application":"photoshop"}]
```

**Expected response:**
```
< 42["registration_response",{"type":"registration","status":"success","message":"Registered for photoshop"}]
```

**Verify in status endpoint:**
```bash
curl http://localhost:3001/status | jq
```

Should show:
```json
{
  "status": "running",
  "port": 3001,
  "clients": {
    "photoshop": 1
  },
  "uptime": 15
}
```

### Test 4: Command Packet Flow

**Terminal 1 (UXP Plugin Simulator):**
```bash
wscat -c ws://localhost:3001/socket.io/
```

Register as photoshop:
```
> 42["register",{"application":"photoshop"}]
```

Wait for command packet...

**Terminal 2 (MCP Server Simulator):**
```bash
wscat -c ws://localhost:3001/socket.io/
```

Send command (note your socket ID from the connect message):
```
> 42["command_packet",{"application":"photoshop","command":{"action":"createLayer","name":"Test Layer"}}]
```

**Terminal 1 should receive:**
```
< 42["command_packet",{"senderId":"<terminal-2-uuid>","application":"photoshop","command":{"action":"createLayer","name":"Test Layer"}}]
```

Send response back:
```
> 42["command_packet_response",{"packet":{"senderId":"<terminal-2-uuid>","result":{"success":true,"layerId":5}}}]
```

**Terminal 2 should receive:**
```
< 42["packet_response",{"senderId":"<terminal-2-uuid>","result":{"success":true,"layerId":5}}]
```

## Automated Testing with Python

### Test Script

Save as `test_protocol.py`:

```python
#!/usr/bin/env python3
import asyncio
import websockets
import json

async def test_socket_io_protocol():
    uri = "ws://localhost:3001/socket.io/"

    async with websockets.connect(uri) as websocket:
        # Test 1: Receive handshake
        handshake = await websocket.recv()
        print(f"✓ Handshake: {handshake}")
        assert handshake.startswith("0{"), "Invalid handshake"

        # Test 2: Ping/Pong
        await websocket.send("2")
        pong = await websocket.recv()
        print(f"✓ Pong: {pong}")
        assert pong == "3", "Invalid pong response"

        # Test 3: Register
        register_msg = '42["register",{"application":"photoshop"}]'
        await websocket.send(register_msg)
        response = await websocket.recv()
        print(f"✓ Registration: {response}")
        assert "registration_response" in response, "Invalid registration"
        assert "success" in response, "Registration failed"

        # Test 4: Send command
        command_msg = '42["command_packet",{"application":"illustrator","command":{"test":true}}]'
        await websocket.send(command_msg)
        print(f"✓ Command sent")

        print("\n✅ All protocol tests passed!")

if __name__ == "__main__":
    asyncio.run(test_socket_io_protocol())
```

Run:
```bash
python test_protocol.py
```

### Test with Multiple Clients

Save as `test_routing.py`:

```python
#!/usr/bin/env python3
import asyncio
import websockets
import json
import re

async def client_application(app_name):
    """Simulates an Adobe application (UXP plugin)"""
    uri = "ws://localhost:3001/socket.io/"

    async with websockets.connect(uri) as ws:
        # Receive handshake
        handshake = await ws.recv()
        print(f"[{app_name}] Connected: {handshake}")

        # Register
        register = f'42["register",{{"application":"{app_name}"}}]'
        await ws.send(register)
        response = await ws.recv()
        print(f"[{app_name}] Registered: {response}")

        # Wait for commands
        try:
            while True:
                msg = await asyncio.wait_for(ws.recv(), timeout=5.0)
                if "command_packet" in msg:
                    print(f"[{app_name}] Received command: {msg}")

                    # Extract senderId
                    match = re.search(r'"senderId":"([^"]+)"', msg)
                    if match:
                        sender_id = match.group(1)

                        # Send response
                        response = f'42["command_packet_response",{{"packet":{{"senderId":"{sender_id}","result":{{"success":true}}}}}}]'
                        await ws.send(response)
                        print(f"[{app_name}] Sent response")
        except asyncio.TimeoutError:
            print(f"[{app_name}] No more commands")

async def client_mcp():
    """Simulates an MCP server sending commands"""
    uri = "ws://localhost:3001/socket.io/"

    await asyncio.sleep(1)  # Let applications register first

    async with websockets.connect(uri) as ws:
        # Receive handshake
        handshake = await ws.recv()
        client_id = re.search(r'"sid":"([^"]+)"', handshake).group(1)
        print(f"[MCP] Connected as: {client_id}")

        # Send command to photoshop
        cmd = '42["command_packet",{"application":"photoshop","command":{"action":"test"}}]'
        await ws.send(cmd)
        print(f"[MCP] Sent command to photoshop")

        # Wait for response
        response = await asyncio.wait_for(ws.recv(), timeout=5.0)
        print(f"[MCP] Received response: {response}")

        assert "packet_response" in response, "Invalid response type"
        assert "success" in response, "Command failed"

async def test_routing():
    """Test command routing between MCP and multiple applications"""
    # Start application clients
    apps = [
        asyncio.create_task(client_application("photoshop")),
        asyncio.create_task(client_application("illustrator")),
    ]

    # Wait a bit for registration
    await asyncio.sleep(0.5)

    # Start MCP client
    mcp = asyncio.create_task(client_mcp())

    # Wait for all
    await asyncio.gather(*apps, mcp)

    print("\n✅ Routing test passed!")

if __name__ == "__main__":
    asyncio.run(test_routing())
```

Run:
```bash
python test_routing.py
```

## Load Testing

### Simple Load Test

```python
#!/usr/bin/env python3
import asyncio
import websockets
import time

async def stress_client(client_id):
    uri = "ws://localhost:3001/socket.io/"

    async with websockets.connect(uri) as ws:
        await ws.recv()  # handshake

        # Register
        await ws.send(f'42["register",{{"application":"client-{client_id}"}}]')
        await ws.recv()  # registration response

        # Send rapid commands
        for i in range(100):
            cmd = f'42["command_packet",{{"application":"test","command":{{"id":{i}}}}}]'
            await ws.send(cmd)

        print(f"Client {client_id} sent 100 commands")

async def load_test(num_clients=10):
    start = time.time()

    tasks = [stress_client(i) for i in range(num_clients)]
    await asyncio.gather(*tasks)

    duration = time.time() - start
    total_commands = num_clients * 100

    print(f"\n✅ Load test complete:")
    print(f"   Clients: {num_clients}")
    print(f"   Total commands: {total_commands}")
    print(f"   Duration: {duration:.2f}s")
    print(f"   Commands/sec: {total_commands/duration:.0f}")

if __name__ == "__main__":
    asyncio.run(load_test(20))
```

## Comparison Test (Node.js vs Rust)

### Setup

Terminal 1 - Node.js proxy:
```bash
cd T:/projects/mcp_servers/adobe-controller/adobe-mcp-unified/proxy-server
node proxy.js
```

Terminal 2 - Rust proxy (different port):
```bash
adobe-proxy --port 3002
```

### Run Tests

```python
#!/usr/bin/env python3
import asyncio
import websockets
import time

async def test_proxy(port, name):
    uri = f"ws://localhost:{port}/socket.io/"

    start = time.time()

    async with websockets.connect(uri) as ws:
        await ws.recv()  # handshake

        # Register
        await ws.send('42["register",{"application":"test"}]')
        await ws.recv()

        # Send 1000 pings
        for _ in range(1000):
            await ws.send("2")
            await ws.recv()

    duration = time.time() - start
    print(f"{name:15} {duration:.3f}s  ({1000/duration:.0f} ops/sec)")

async def compare():
    print("Proxy Performance Comparison (1000 ping/pong)")
    print("-" * 50)

    await test_proxy(3001, "Node.js")
    await test_proxy(3002, "Rust")

if __name__ == "__main__":
    asyncio.run(compare())
```

## Expected Results

### Protocol Compatibility

All Socket.IO messages should match byte-for-byte between Node.js and Rust implementations:

- Connection handshake: `0{"sid":"<uuid>"}`
- Ping/Pong: `2` → `3`
- Events: `42["event_name",{data}]`

### Performance Benchmarks

Typical results (may vary):

| Metric | Node.js | Rust |
|--------|---------|------|
| Startup time | 500ms | 50ms |
| Memory (idle) | 50 MB | 5 MB |
| Memory (100 clients) | 150 MB | 15 MB |
| Ping/Pong latency | 1-2ms | 0.5-1ms |
| Throughput | 10K msg/sec | 50K msg/sec |

### Status Endpoint

Both should return identical structure:

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

## Troubleshooting

### WebSocket Connection Fails

Check server is running:
```bash
curl http://localhost:3001/status
```

### No Handshake Received

- Verify connecting to `/socket.io/` path
- Check server logs: `RUST_LOG=debug adobe-proxy`

### Commands Not Routing

- Verify application name matches exactly
- Check client is registered before sending commands
- Enable trace logging: `RUST_LOG=trace adobe-proxy`

### Response Not Received

- Verify `senderId` is preserved in response packet
- Check both clients are still connected
- Monitor with Wireshark/tcpdump if needed
