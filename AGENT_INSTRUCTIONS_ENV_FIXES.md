# Environment Fix Agent Instructions

## Primary Mission
Fix WSL vs Windows venv path resolution issues to ensure the Adobe MCP system works seamlessly across different environments.

## Critical Focus Areas

### 1. Virtual Environment Path Resolution
- [ ] Fix venv activation scripts for both WSL and Windows
- [ ] Handle path conversions between WSL paths (/mnt/c) and Windows paths (C:\)
- [ ] Create universal path normalization utilities
- [ ] Test with various Python installation methods

### 2. Python Executable Detection
- [ ] Support both `python` and `python.exe`
- [ ] Handle Python from Windows Store
- [ ] Support conda environments
- [ ] Create robust Python detection script

### 3. Cross-Platform Script Support
- [ ] Make all .ps1 scripts work in PowerShell Core
- [ ] Create .sh equivalents for WSL/Linux
- [ ] Handle spaces in paths correctly
- [ ] Support long path names on Windows

### 4. Environment Variables
- [ ] Properly set PYTHONPATH across platforms
- [ ] Handle PATH modifications for both environments
- [ ] Create environment detection utilities
- [ ] Document required environment variables

## Testing Requirements
- Test on native Windows (PowerShell)
- Test on WSL2 with Windows Python
- Test on WSL2 with Linux Python
- Test with paths containing spaces
- Test with non-ASCII characters in paths

## Deliverables
1. Universal path normalization module
2. Cross-platform venv activation scripts
3. Environment detection utilities
4. Comprehensive test suite
5. Setup validation script