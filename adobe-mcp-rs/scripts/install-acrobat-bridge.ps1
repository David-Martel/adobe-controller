# Install Acrobat bridge plugin (Windows)

param(
    [string]$BridgeDllPath,
    [string]$PluginPath = "C:\\Program Files\\Adobe\\Acrobat DC\\Acrobat\\plug_ins\\AcrobatMcpBridge.api"
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)

if (-not $BridgeDllPath) {
    $BridgeDllPath = Join-Path $repoRoot "adobe-mcp-rs\\target\\debug\\acrobat_bridge.dll"
    if (-not (Test-Path $BridgeDllPath)) {
        $BridgeDllPath = "T:\\RustCache\\cargo-target\\debug\\acrobat_bridge.dll"
    }
}

if (-not (Test-Path $BridgeDllPath)) {
    throw "Bridge DLL not found at $BridgeDllPath. Build with: cargo build --package acrobat-bridge"
}

$pluginDir = Split-Path -Parent $PluginPath
if (-not (Test-Path $pluginDir)) {
    throw "Acrobat plug-ins folder not found at $pluginDir. Install Acrobat or update -PluginPath."
}

Copy-Item -Path $BridgeDllPath -Destination $PluginPath -Force
Write-Host "Installed Acrobat bridge to $PluginPath"
