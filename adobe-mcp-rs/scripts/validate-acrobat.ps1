# Validate Acrobat MCP toolchain (requires Acrobat + bridge plugin)

param(
    [string]$ProxyUrl = "ws://127.0.0.1:3001",
    [string]$ProxyHost = "127.0.0.1",
    [int]$ProxyPort = 3001,
    [string]$ProxyPath,
    [string]$McpPath,
    [string]$OutputDir = "$env:USERPROFILE\\Documents\\Adobe_MCP_Tests\\Acrobat",
    [int]$TimeoutMs = 30000
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$rustRoot = Join-Path $repoRoot "adobe-mcp-rs"

if (-not $ProxyPath) {
    $ProxyPath = Join-Path $rustRoot "target\\debug\\adobe-proxy.exe"
}

if (-not $McpPath) {
    $McpPath = Join-Path $rustRoot "target\\debug\\acrobat-mcp.exe"
}

if (-not (Test-Path $ProxyPath)) {
    throw "Proxy binary not found at $ProxyPath. Build with: cargo build --package adobe-proxy"
}

if (-not (Test-Path $McpPath)) {
    throw "Acrobat MCP binary not found at $McpPath. Build with: cargo build --package acrobat-mcp"
}

if (-not (Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
}

function Test-Proxy {
    try {
        $response = Invoke-WebRequest -Uri "http://${ProxyHost}:${ProxyPort}/status" -TimeoutSec 2
        return $response.StatusCode -eq 200
    } catch {
        return $false
    }
}

$proxyProcess = $null
if (-not (Test-Proxy)) {
    $proxyProcess = Start-Process -FilePath $ProxyPath -ArgumentList "--host", $ProxyHost, "--port", $ProxyPort -PassThru
    Start-Sleep -Seconds 2
    if (-not (Test-Proxy)) {
        if ($proxyProcess) {
            Stop-Process -Id $proxyProcess.Id -ErrorAction SilentlyContinue
        }
        throw "Proxy did not start on http://${ProxyHost}:${ProxyPort}/status"
    }
}

$psi = New-Object System.Diagnostics.ProcessStartInfo
$psi.FileName = $McpPath
$psi.Arguments = "--proxy-url $ProxyUrl --timeout $TimeoutMs"
$psi.WorkingDirectory = $rustRoot
$psi.RedirectStandardInput = $true
$psi.RedirectStandardOutput = $true
$psi.RedirectStandardError = $true
$psi.UseShellExecute = $false
$psi.CreateNoWindow = $true

$process = [System.Diagnostics.Process]::Start($psi)
if (-not $process) {
    throw "Failed to start Acrobat MCP server"
}

function Read-Response {
    param([System.IO.TextReader]$Reader, [int]$Timeout = 15000)
    $task = $Reader.ReadLineAsync()
    if (-not $task.Wait($Timeout)) {
        throw "Timed out waiting for MCP response"
    }
    return $task.Result
}

function Invoke-McpRequest {
    param([int]$Id, [string]$Method, [hashtable]$Params)
    $payload = @{ jsonrpc = "2.0"; id = $Id; method = $Method }
    if ($Params) {
        $payload.params = $Params
    }
    $json = $payload | ConvertTo-Json -Compress
    $process.StandardInput.WriteLine($json)
    $responseLine = Read-Response -Reader $process.StandardOutput
    if (-not $responseLine) {
        throw "Empty response from MCP server"
    }
    return $responseLine | ConvertFrom-Json
}

try {
    $results = @()
    $results += Invoke-McpRequest -Id 1 -Method "initialize" -Params @{}
    $tools = Invoke-McpRequest -Id 2 -Method "tools/list" -Params @{}
    $results += $tools

    $createArgs = @{ name = "MCP Validation"; page_count = 2; page_size = "LETTER" }
    $results += Invoke-McpRequest -Id 3 -Method "tools/call" -Params @{ name = "create_document"; arguments = $createArgs }

    $results += Invoke-McpRequest -Id 4 -Method "tools/call" -Params @{ name = "get_document_info"; arguments = @{} }
    $results += Invoke-McpRequest -Id 5 -Method "tools/call" -Params @{ name = "add_text"; arguments = @{ text = "MCP validation"; page = 1; font_size = 18 } }
    $results += Invoke-McpRequest -Id 6 -Method "tools/call" -Params @{ name = "get_page_count"; arguments = @{} }
    $results += Invoke-McpRequest -Id 7 -Method "tools/call" -Params @{ name = "set_metadata"; arguments = @{ title = "MCP Validation"; author = "Adobe MCP"; subject = "Validation"; keywords = "mcp,acrobat" } }
    $results += Invoke-McpRequest -Id 8 -Method "tools/call" -Params @{ name = "add_bookmark"; arguments = @{ title = "Start"; page = 1 } }

    $outputPath = Join-Path $OutputDir "mcp-validation.pdf"
    $results += Invoke-McpRequest -Id 9 -Method "tools/call" -Params @{ name = "save_document"; arguments = @{ file_path = $outputPath } }
    $results += Invoke-McpRequest -Id 10 -Method "tools/call" -Params @{ name = "close_document"; arguments = @{ save_changes = $true } }

    Write-Host "Validation complete. Tool count: $($tools.result.tools.Count). Output: $outputPath" -ForegroundColor Green
} finally {
    if ($process -and -not $process.HasExited) {
        $process.Kill()
    }
    if ($proxyProcess -and -not $proxyProcess.HasExited) {
        Stop-Process -Id $proxyProcess.Id -ErrorAction SilentlyContinue
    }
}
