# Adobe MCP Rust Workspace Build Script
# Uses sccache for caching and CargoTools for parallel builds

param(
    [switch]$Release,
    [switch]$Clean,
    [switch]$Test,
    [switch]$Check,
    [switch]$Clippy,
    [string]$Package
)

$ErrorActionPreference = "Stop"
$workspace = $PSScriptRoot

Write-Host "Adobe MCP Rust Workspace Builder" -ForegroundColor Cyan
Write-Host "=================================" -ForegroundColor Cyan

# Ensure sccache is running
if (Get-Command sccache -ErrorAction SilentlyContinue) {
    Write-Host "Using sccache for compilation caching" -ForegroundColor Green
    $env:RUSTC_WRAPPER = "sccache"
    sccache --show-stats | Select-String "Compile requests"
}

# Set target directory to shared cache
$env:CARGO_TARGET_DIR = "T:\RustCache\cargo-target"

if ($Clean) {
    Write-Host "`nCleaning workspace..." -ForegroundColor Yellow
    cargo clean
    exit 0
}

if ($Check) {
    Write-Host "`nRunning cargo check..." -ForegroundColor Yellow
    if ($Package) {
        cargo check --package $Package
    } else {
        cargo check --workspace
    }
    exit $LASTEXITCODE
}

if ($Clippy) {
    Write-Host "`nRunning clippy..." -ForegroundColor Yellow
    if ($Package) {
        cargo clippy --package $Package -- -D warnings
    } else {
        cargo clippy --workspace -- -D warnings
    }
    exit $LASTEXITCODE
}

# Build
$buildArgs = @("build")
if ($Release) {
    $buildArgs += "--release"
    Write-Host "`nBuilding in RELEASE mode..." -ForegroundColor Green
} else {
    Write-Host "`nBuilding in DEBUG mode..." -ForegroundColor Yellow
}

if ($Package) {
    $buildArgs += "--package", $Package
    Write-Host "Package: $Package" -ForegroundColor Cyan
} else {
    $buildArgs += "--workspace"
    Write-Host "Building all packages..." -ForegroundColor Cyan
}

$startTime = Get-Date
cargo @buildArgs
$buildExitCode = $LASTEXITCODE
$elapsed = (Get-Date) - $startTime

if ($buildExitCode -eq 0) {
    Write-Host "`nBuild successful in $($elapsed.TotalSeconds.ToString('F2'))s" -ForegroundColor Green

    # Show binary locations
    $targetDir = if ($Release) { "release" } else { "debug" }
    $binPath = Join-Path $env:CARGO_TARGET_DIR $targetDir

    Write-Host "`nBinaries:" -ForegroundColor Cyan
    Get-ChildItem -Path $binPath -Filter "*.exe" -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -match "^(adobe-proxy|acrobat-mcp|acrobat-bridge)" } |
        ForEach-Object {
            Write-Host "  $($_.Name) - $([math]::Round($_.Length / 1MB, 2)) MB" -ForegroundColor White
        }
} else {
    Write-Host "`nBuild failed with exit code $buildExitCode" -ForegroundColor Red
    exit $buildExitCode
}

if ($Test) {
    Write-Host "`nRunning tests..." -ForegroundColor Yellow
    if ($Package) {
        cargo test --package $Package
    } else {
        cargo test --workspace
    }
    exit $LASTEXITCODE
}

# Show sccache stats after build
if (Get-Command sccache -ErrorAction SilentlyContinue) {
    Write-Host "`nCache statistics:" -ForegroundColor Cyan
    sccache --show-stats | Select-String "Compile requests|Cache hits|Cache misses"
}
