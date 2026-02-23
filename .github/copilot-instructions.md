# Copilot instructions

## Build, test, and lint

> Run commands from the repository root unless noted.

### Python (adobe-mcp-unified)
- Install: `cd adobe-mcp-unified` then `pip install -e .`
- Dev install: `cd adobe-mcp-unified` then `pip install -e ".[dev]"`
- Lint: `cd adobe-mcp-unified` then `ruff adobe_mcp/`
- Format: `cd adobe-mcp-unified` then `black adobe_mcp/` and `isort adobe_mcp/`
- Type check: `cd adobe-mcp-unified` then `mypy adobe_mcp/`

### Proxy server
- Install deps: `cd adobe-mcp-unified\proxy-server` then `npm install`
- Run proxy: `cd adobe-mcp-unified` then `adobe-proxy` (or `node proxy-server/proxy.js`)

### Tests
- Full suite (Windows): `cd adobe-mcp-unified` then `./run-tests.ps1`
- Coverage: `cd adobe-mcp-unified` then `./run-tests.ps1 -Coverage`
- Manual Illustrator test: `cd adobe-mcp-unified` then `./run-tests.ps1 -Manual`
- Single pytest file: `cd adobe-mcp-unified` then `python -m pytest tests/test_illustrator.py -v -s`
- Standalone tests: `cd adobe-mcp-unified` then `python test_basic.py` or `python test_mcp_servers.py`
- Interactive shell: `cd adobe-mcp-unified` then `python mcp_shell.py`
- PowerShell test suite: `cd adobe-mcp-unified` then `./Test-AdobeMCP.ps1`

### Rust (adobe-mcp-rs)
- Build: `cd adobe-mcp-rs` then `cargo build`
- Test: `cd adobe-mcp-rs` then `cargo test`

## High-level architecture
- 3-tier flow: MCP servers (Python) ↔ proxy server (Node.js WebSocket on port 3001) ↔ UXP plugins (JavaScript) running inside Adobe apps.
- Each app has its own MCP server module (`adobe_mcp/{photoshop,premiere,illustrator,indesign}`) and a matching UXP plugin under `uxp-plugins/{app}`.
- Shared Python utilities live in `adobe_mcp/shared/` (socket client, logging, fonts, detection). The socket client is the single path to the proxy.
- Commands follow a command pattern: MCP servers create `createCommand(action, options)` payloads, the proxy routes them, and plugins execute handlers and return `{status, message, data}`.
- The Rust workspace (`adobe-mcp-rs`) targets Acrobat and is separate from the unified Python implementation.

## Key conventions
- Proxy dependency: all MCP servers require the proxy on port 3001; Illustrator includes a built-in proxy but still uses the same routing.
- UXP plugins must be manually loaded via Adobe UXP Developer Tools (manifest.json in each plugin folder).
- Error payloads use `{status: "ok"|"error", message: string, data: any}`.
- Font utilities normalize to PostScript names and cap results (see `adobe_mcp/shared/fonts.py`).
- When adding features, implement all three layers: MCP server tool → UXP command handler → plugin command registration.
