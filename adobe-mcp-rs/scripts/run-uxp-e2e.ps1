# Run Adobe MCP Rust proxy + MCP servers with UXP plugin deployment (manual app requirement)

param(
    [string]$ProxyPath,
    [ValidateSet("photoshop","acrobat","illustrator","indesign")]
    [string]$McpServer = "photoshop",
    [ValidateSet("rust","python")]
    [string]$McpRuntime = "rust",
    [string]$ProxyUrl = "ws://127.0.0.1:3001",
    [switch]$DeployUxp,
    [switch]$Verbose
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$rustRoot = Join-Path $repoRoot "adobe-mcp-rs"
$pythonRoot = Join-Path $repoRoot "adobe-mcp-unified"
$uxpDeploy = Join-Path $repoRoot "adobe-mcp-unified\\scripts\\deploy-uxp-plugin.ps1"
$uxpPlugins = Join-Path $repoRoot "adobe-mcp-unified\\uxp-plugins"

function Write-Log {
    param([string]$Message)
    if ($Verbose) {
        Write-Host $Message
    }
}

if (-not $ProxyPath) {
    $ProxyPath = Join-Path $rustRoot "target\\debug\\adobe-proxy.exe"
}

if (-not (Test-Path $ProxyPath)) {
    throw "Proxy binary not found at $ProxyPath. Build with: cargo build --workspace"
}

if ($McpRuntime -eq "python" -and $McpServer -eq "acrobat") {
    throw "Acrobat MCP server is only available in Rust. Use -McpRuntime rust."
}

if ($DeployUxp) {
    if (-not (Test-Path $uxpDeploy)) {
        throw "UXP deploy script not found at $uxpDeploy"
    }
    & $uxpDeploy deploy-all $uxpPlugins -Verbose:$Verbose
}

$proxyProcess = Start-Process -FilePath $ProxyPath -ArgumentList "--host", "127.0.0.1", "--port", "3001" -PassThru
Write-Log "Started proxy (PID=$($proxyProcess.Id))"

Start-Sleep -Seconds 2

$mcpBin = Join-Path $rustRoot "target\\debug\\$McpServer-mcp.exe"
if ($McpRuntime -eq "rust" -and -not (Test-Path $mcpBin)) {
    Stop-Process -Id $proxyProcess.Id -ErrorAction SilentlyContinue
    throw "MCP server binary not found at $mcpBin. Build with: cargo build --workspace"
}

$env:PHOTOSHOP_PROXY_URL = $ProxyUrl
$env:ACROBAT_PROXY_URL = $ProxyUrl
$env:ILLUSTRATOR_PROXY_URL = $ProxyUrl
$env:INDESIGN_PROXY_URL = $ProxyUrl

$args = @()
if ($McpRuntime -eq "rust") {
    $args = @("--proxy-url", $ProxyUrl)
}

if ($McpRuntime -eq "rust") {
    $mcpProcess = Start-Process -FilePath $mcpBin -ArgumentList $args -PassThru
    Write-Log "Started MCP server (PID=$($mcpProcess.Id))"
} else {
    $pythonExe = Join-Path $pythonRoot ".venv\\Scripts\\python.exe"
    if (-not (Test-Path $pythonExe)) {
        $pythonExe = "python"
    }
    $mcpProcess = Start-Process -FilePath $pythonExe -ArgumentList "-m", "adobe_mcp.$McpServer" -WorkingDirectory $pythonRoot -PassThru
    Write-Log "Started MCP server (PID=$($mcpProcess.Id))"
}

Write-Host "Proxy and MCP server running. Use your MCP client to send requests."
Write-Host "Press Enter to stop..."
[void][System.Console]::ReadLine()

Stop-Process -Id $mcpProcess.Id -ErrorAction SilentlyContinue
Stop-Process -Id $proxyProcess.Id -ErrorAction SilentlyContinue
