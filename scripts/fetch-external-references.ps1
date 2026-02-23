$ErrorActionPreference = "Stop"

$targets = @(
    @{ Url = "https://raw.githubusercontent.com/AdobeDocs/uxp-photoshop-plugin-samples/main/LICENSE"; Path = "docs\external-references\adobe-uxp-samples\LICENSE" },
    @{ Url = "https://raw.githubusercontent.com/AdobeDocs/uxp-photoshop-plugin-samples/main/io-websocket-example/README.md"; Path = "docs\external-references\adobe-uxp-samples\io-websocket-example\README.md" },
    @{ Url = "https://raw.githubusercontent.com/AdobeDocs/uxp-photoshop-plugin-samples/main/io-websocket-example/plugin/index.html"; Path = "docs\external-references\adobe-uxp-samples\io-websocket-example\plugin\index.html" },
    @{ Url = "https://raw.githubusercontent.com/AdobeDocs/uxp-photoshop-plugin-samples/main/io-websocket-example/plugin/index.js"; Path = "docs\external-references\adobe-uxp-samples\io-websocket-example\plugin\index.js" },
    @{ Url = "https://raw.githubusercontent.com/AdobeDocs/uxp-photoshop-plugin-samples/main/io-websocket-example/plugin/manifest.json"; Path = "docs\external-references\adobe-uxp-samples\io-websocket-example\plugin\manifest.json" },
    @{ Url = "https://raw.githubusercontent.com/AdobeDocs/uxp-photoshop-plugin-samples/main/io-websocket-example/server/index.js"; Path = "docs\external-references\adobe-uxp-samples\io-websocket-example\server\index.js" },
    @{ Url = "https://raw.githubusercontent.com/AdobeDocs/uxp-photoshop-plugin-samples/main/io-websocket-example/server/package.json"; Path = "docs\external-references\adobe-uxp-samples\io-websocket-example\server\package.json" },
    @{ Url = "https://raw.githubusercontent.com/AdobeDocs/uxp-photoshop-plugin-samples/main/desktop-helper-sample/README.md"; Path = "docs\external-references\adobe-uxp-samples\desktop-helper-sample\README.md" },
    @{ Url = "https://raw.githubusercontent.com/AdobeDocs/uxp-photoshop-plugin-samples/main/desktop-helper-sample/uxp/package.json"; Path = "docs\external-references\adobe-uxp-samples\desktop-helper-sample\uxp\package.json" },
    @{ Url = "https://raw.githubusercontent.com/AdobeDocs/uxp-photoshop-plugin-samples/main/desktop-helper-sample/uxp/webpack.config.js"; Path = "docs\external-references\adobe-uxp-samples\desktop-helper-sample\uxp\webpack.config.js" },
    @{ Url = "https://raw.githubusercontent.com/AdobeDocs/uxp-photoshop-plugin-samples/main/desktop-helper-sample/uxp/plugin/index.html"; Path = "docs\external-references\adobe-uxp-samples\desktop-helper-sample\uxp\plugin\index.html" },
    @{ Url = "https://raw.githubusercontent.com/AdobeDocs/uxp-photoshop-plugin-samples/main/desktop-helper-sample/uxp/plugin/manifest.json"; Path = "docs\external-references\adobe-uxp-samples\desktop-helper-sample\uxp\plugin\manifest.json" },
    @{ Url = "https://raw.githubusercontent.com/AdobeDocs/uxp-photoshop-plugin-samples/main/desktop-helper-sample/uxp/src/index.js"; Path = "docs\external-references\adobe-uxp-samples\desktop-helper-sample\uxp\src\index.js" },
    @{ Url = "https://raw.githubusercontent.com/AdobeDocs/uxp-photoshop-plugin-samples/main/desktop-helper-sample/uxp/src/utils.js"; Path = "docs\external-references\adobe-uxp-samples\desktop-helper-sample\uxp\src\utils.js" },
    @{ Url = "https://raw.githubusercontent.com/AdobeDocs/uxp-photoshop-plugin-samples/main/desktop-helper-sample/helper/src/App.jsx"; Path = "docs\external-references\adobe-uxp-samples\desktop-helper-sample\helper\src\App.jsx" },
    @{ Url = "https://raw.githubusercontent.com/AdobeDocs/uxp-photoshop-plugin-samples/main/desktop-helper-sample/helper/src/index.css"; Path = "docs\external-references\adobe-uxp-samples\desktop-helper-sample\helper\src\index.css" },
    @{ Url = "https://raw.githubusercontent.com/AdobeDocs/uxp-photoshop-plugin-samples/main/desktop-helper-sample/helper/src/index.js"; Path = "docs\external-references\adobe-uxp-samples\desktop-helper-sample\helper\src\index.js" },
    @{ Url = "https://raw.githubusercontent.com/mikechambers/adb-mcp/main/LICENSE.md"; Path = "docs\external-references\adb-mcp\LICENSE.md" },
    @{ Url = "https://raw.githubusercontent.com/mikechambers/adb-mcp/main/adb-proxy-socket/README.md"; Path = "docs\external-references\adb-mcp\adb-proxy-socket\README.md" },
    @{ Url = "https://raw.githubusercontent.com/mikechambers/adb-mcp/main/adb-proxy-socket/package.json"; Path = "docs\external-references\adb-mcp\adb-proxy-socket\package.json" },
    @{ Url = "https://raw.githubusercontent.com/mikechambers/adb-mcp/main/adb-proxy-socket/proxy.js"; Path = "docs\external-references\adb-mcp\adb-proxy-socket\proxy.js" },
    @{ Url = "https://raw.githubusercontent.com/modelcontextprotocol/rust-sdk/main/LICENSE"; Path = "docs\external-references\rust-sdk\LICENSE" },
    @{ Url = "https://raw.githubusercontent.com/modelcontextprotocol/rust-sdk/main/README.md"; Path = "docs\external-references\rust-sdk\README.md" },
    @{ Url = "https://raw.githubusercontent.com/modelcontextprotocol/rust-sdk/main/examples/servers/README.md"; Path = "docs\external-references\rust-sdk\examples\servers\README.md" },
    @{ Url = "https://raw.githubusercontent.com/modelcontextprotocol/rust-sdk/main/examples/servers/Cargo.toml"; Path = "docs\external-references\rust-sdk\examples\servers\Cargo.toml" },
    @{ Url = "https://raw.githubusercontent.com/modelcontextprotocol/rust-sdk/main/examples/servers/src/memory_stdio.rs"; Path = "docs\external-references\rust-sdk\examples\servers\src\memory_stdio.rs" },
    @{ Url = "https://raw.githubusercontent.com/modelcontextprotocol/rust-sdk/main/examples/servers/src/calculator_stdio.rs"; Path = "docs\external-references\rust-sdk\examples\servers\src\calculator_stdio.rs" },
    @{ Url = "https://raw.githubusercontent.com/modelcontextprotocol/rust-sdk/main/examples/servers/src/counter_stdio.rs"; Path = "docs\external-references\rust-sdk\examples\servers\src\counter_stdio.rs" },
    @{ Url = "https://raw.githubusercontent.com/modelcontextprotocol/rust-sdk/main/examples/servers/src/prompt_stdio.rs"; Path = "docs\external-references\rust-sdk\examples\servers\src\prompt_stdio.rs" },
    @{ Url = "https://raw.githubusercontent.com/modelcontextprotocol/rust-sdk/main/examples/servers/src/elicitation_stdio.rs"; Path = "docs\external-references\rust-sdk\examples\servers\src\elicitation_stdio.rs" },
    @{ Url = "https://raw.githubusercontent.com/modelcontextprotocol/rust-sdk/main/examples/servers/src/common/mod.rs"; Path = "docs\external-references\rust-sdk\examples\servers\src\common\mod.rs" },
    @{ Url = "https://raw.githubusercontent.com/modelcontextprotocol/rust-sdk/main/examples/servers/src/common/generic_service.rs"; Path = "docs\external-references\rust-sdk\examples\servers\src\common\generic_service.rs" }
)

foreach ($target in $targets) {
    $dest = Join-Path "T:\projects\mcp_servers\adobe-controller" $target.Path
    $dir = Split-Path -Parent $dest
    if (-not (Test-Path $dir)) {
        New-Item -ItemType Directory -Force -Path $dir | Out-Null
    }
    Invoke-WebRequest -Uri $target.Url -OutFile $dest
}
