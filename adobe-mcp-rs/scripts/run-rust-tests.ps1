# Run Rust tests with sccache and consistent target dir

param(
    [switch]$Clippy,
    [switch]$Fmt,
    [switch]$Build
)

$ErrorActionPreference = "Stop"
$workspace = Split-Path -Parent $PSScriptRoot

if (Get-Command sccache -ErrorAction SilentlyContinue) {
    $env:RUSTC_WRAPPER = "sccache"
}

$env:CARGO_TARGET_DIR = "T:\RustCache\cargo-target"

Push-Location $workspace
try {
    if ($Fmt) {
        cargo fmt --all
    }
    if ($Clippy) {
        cargo clippy --workspace -- -D warnings
    }
    cargo test --workspace
    if ($Build) {
        cargo build --workspace
    }
} finally {
    Pop-Location
}
