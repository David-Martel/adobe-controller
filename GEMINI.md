# Adobe MCP Controller

This project is a comprehensive Model Context Protocol (MCP) server system designed to control Adobe Creative Suite applications (Photoshop, Premiere Pro, Illustrator, InDesign, Acrobat) via AI agents. It enables natural language interaction with complex creative software.

## Project Overview

The project is structured around a 3-tier architecture to bridge the gap between external AI agents and local Adobe desktop applications:

1.  **MCP Servers (Python):** act as the interface for AI agents (like Gemini or Claude). They translate natural language intent into structured commands.
2.  **Proxy Server (Node.js/Python):** Manages WebSocket connections to the Adobe applications, handling command routing and event listening.
3.  **UXP Plugins (JavaScript):** specialized plugins installed within each Adobe application that execute the actual commands via the Adobe API.

A parallel **Rust implementation** (`adobe-mcp-rs`) exists, specifically targeting Acrobat and potentially offering a higher-performance bridge in the future.

## Key Directories

*   **`adobe-mcp-unified/`**: The main Python-based implementation.
    *   `adobe_mcp/`: Source code for the MCP servers (`photoshop`, `premiere`, `illustrator`, `indesign`).
    *   `proxy-server/`: Node.js WebSocket proxy (bridging MCP to UXP).
    *   `uxp-plugins/`: Source code for the Adobe UXP plugins.
    *   `tests/`: Python test suite.
*   **`adobe-mcp-rs/`**: Rust workspace for native implementations (currently focused on Acrobat).
*   **`scripts/`**: Global utility scripts (e.g., `setup-adobe-agents.sh`).

## Setup and Installation

### Prerequisites
*   Python 3.10+
*   Node.js 18+
*   Rust (cargo) - *if working on `adobe-mcp-rs`*
*   Adobe Creative Cloud Apps (Photoshop v26+, Premiere v25+, etc.)
*   Adobe UXP Developer Tools

### Unified Python Setup (Primary)
Navigate to `adobe-mcp-unified` and use the provided scripts:

*   **Windows:**
    ```powershell
    .\install.ps1
    ```
    Or manually: `pip install -e .` and `npm install` in `proxy-server`.

## Running the Servers

### 1. Start the Proxy
The proxy is required for communication with UXP plugins.
```bash
# From adobe-mcp-unified/
adobe-proxy
# OR
node proxy-server/proxy.js
```

### 2. Start MCP Servers
Each application has its own MCP entry point:
```bash
adobe-photoshop
adobe-premiere
adobe-illustrator
adobe-indesign
```
*Note: Illustrator implementation currently includes a built-in proxy.*

### 3. Load UXP Plugins
Use **Adobe UXP Developer Tools** to load the plugin manifest from `adobe-mcp-unified/uxp-plugins/{app}/manifest.json` into the target application.

## Development Workflows

### Testing
*   **Run all tests (Windows):** `.\adobe-mcp-unified\run-tests.ps1`
*   **Python Tests:** `pytest` inside `adobe-mcp-unified`

### Multi-Agent Development
This project utilizes a multi-agent development strategy (detailed in `MULTI_AGENT_PLAN.md`). Key concepts:
*   **Specialized Agents:** Work is divided among agents like "UXP Research", "Premiere Dev", and "Infrastructure".
*   **Worktrees:** Development often happens in dedicated git worktrees (e.g., `../adobe-mcp-premiere`).
*   **Coordination:** A central plan tracks task distribution.

### Rust Development
For `adobe-mcp-rs`:
*   Build: `cargo build`
*   Test: `cargo test`

## Important Configuration Files
*   `adobe-mcp-unified/pyproject.toml`: Python dependencies and entry points.
*   `adobe-mcp-rs/Cargo.toml`: Rust workspace members.
*   `MULTI_AGENT_PLAN.md`: Strategic roadmap for the multi-agent development process.
