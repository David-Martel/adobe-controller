# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Adobe MCP is a unified Model Context Protocol server that enables AI-powered automation of Adobe Creative Suite applications (Photoshop, Premiere Pro, Illustrator, and InDesign). It uses a 3-tier architecture with Python MCP servers, a Node.js WebSocket proxy, and JavaScript UXP plugins.

## Development Commands

### Installation and Setup
```bash
# Install Python dependencies (from adobe-mcp-unified/ directory)
pip install -e .

# Install proxy server dependencies
cd proxy-server
npm install
cd ..

# Start the proxy server (required for all MCP servers)
adobe-proxy
# Or manually: node proxy-server/proxy.js
```

### Running MCP Servers
```bash
# Individual servers (require proxy to be running)
adobe-photoshop
adobe-premiere
adobe-illustrator
adobe-indesign
```

### Testing
```bash
# Run automated test suite (Windows)
./run-tests.ps1

# Run with coverage
./run-tests.ps1 -Coverage

# Run manual Illustrator test
./run-tests.ps1 -Manual

# Run specific test file
python -m pytest tests/test_illustrator.py -v -s
```

### Development Dependencies
```bash
# Install dev dependencies
pip install -e ".[dev]"

# Format code
black adobe_mcp/
isort adobe_mcp/

# Lint code
ruff adobe_mcp/

# Type checking
mypy adobe_mcp/
```

## Architecture

### Core Components

1. **MCP Servers** (`adobe_mcp/`): FastMCP-based servers that expose tools to AI clients
   - Each application has its own server module (photoshop/, premiere/, illustrator/, indesign/)
   - Shared utilities in `shared/` handle socket communication, logging, and common operations

2. **Proxy Server** (`proxy-server/`): Node.js WebSocket bridge on port 3001
   - Routes commands between MCP servers and UXP plugins
   - Manages real-time bidirectional communication

3. **UXP Plugins** (`uxp-plugins/`): JavaScript plugins that execute commands within Adobe apps
   - Each app has its own plugin with manifest.json and command handlers
   - Commands are organized in separate modules (e.g., layers.js, filters.js)

### Communication Flow
1. AI client sends command to MCP server via stdin/stdout
2. MCP server formats command and sends to proxy via WebSocket
3. Proxy forwards to appropriate UXP plugin
4. Plugin executes in Adobe app and returns result
5. Response flows back through proxy to MCP server to AI client

### Key Design Patterns

- **Command Pattern**: All operations use `createCommand(action, options)` structure
- **Socket Client**: Centralized WebSocket client in `shared/socket_client.py` handles all proxy communication
- **Error Handling**: Commands return `{status: "ok"|"error", message: string, data: any}`
- **Font Management**: Shared font utilities in `shared/fonts.py` with PostScript name resolution

## Important Implementation Notes

1. **Proxy Dependency**: All MCP servers require the proxy server to be running on port 3001
2. **UXP Plugin Loading**: Plugins must be manually loaded via Adobe UXP Developer Tools
3. **Environment Detection**: Use `shared/adobe_detector.py` for finding Adobe installations
4. **WebSocket Timeout**: Default timeout is 20 seconds for command execution
5. **Font Limit**: Maximum 1000 fonts returned to prevent overwhelming AI context

## Adding New Features

1. Add tool method to appropriate MCP server (e.g., `adobe_mcp/photoshop/server.py`)
2. Implement command handler in corresponding UXP plugin (e.g., `uxp-plugins/photoshop/commands/`)
3. Register command in plugin's command index
4. Test via proxy with manual test script

## Windows-Specific Scripts

- `install.ps1`: Sets up virtual environment and dependencies
- `test-setup.ps1`: Validates environment configuration
- `run-tests.ps1`: Test runner with environment management
- `scripts/EnvUtils.psm1`: PowerShell module for environment handling
- `scripts/ProxyManager.psm1`: PowerShell module for proxy server management