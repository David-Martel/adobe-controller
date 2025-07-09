#!/bin/bash
# Adobe MCP Multi-Agent Setup Script with WSL/Windows Awareness and Fault Tolerance

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Detect environment
detect_environment() {
    if [[ -n "${WSL_DISTRO_NAME:-}" ]]; then
        echo "wsl"
    elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
        echo "windows"
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo "linux"
    else
        echo "unknown"
    fi
}

# Convert paths for cross-platform compatibility
normalize_path() {
    local path="$1"
    local env=$(detect_environment)
    
    if [[ "$env" == "wsl" ]]; then
        # Convert Windows paths to WSL paths if needed
        if [[ "$path" =~ ^[A-Za-z]: ]]; then
            wslpath -u "$path"
        else
            echo "$path"
        fi
    else
        echo "$path"
    fi
}

# Configuration
ENVIRONMENT=$(detect_environment)
REPO_ROOT=$(normalize_path "/mnt/t/projects/mcp_servers/adobe-controller")
STATE_DIR="$REPO_ROOT/.agent-state"
LOGS_DIR="$REPO_ROOT/.agent-logs"
CHECKPOINT_DIR="$REPO_ROOT/.agent-checkpoints"

echo -e "${BLUE}Adobe MCP Multi-Agent Setup${NC}"
echo -e "Environment: ${YELLOW}$ENVIRONMENT${NC}"
echo -e "Repository: ${YELLOW}$REPO_ROOT${NC}"
echo ""

# Create state directories
mkdir -p "$STATE_DIR" "$LOGS_DIR" "$CHECKPOINT_DIR"

# Agent configuration
declare -A agents=(
    ["uxp-research"]="research/uxp-capabilities"
    ["env-fixes"]="fix/wsl-windows-paths"
    ["premiere"]="feature/premiere-complete"
    ["illustrator"]="feature/illustrator-enhance"
    ["infrastructure"]="feature/core-improvements"
    ["testing"]="feature/test-framework"
    ["deployment"]="feature/auto-deploy"
)

# Create agent instruction templates
create_agent_instructions() {
    local agent_name="$1"
    local agent_type="$2"
    local instructions_file="$STATE_DIR/${agent_name}-instructions.md"
    
    cat > "$instructions_file" << EOF
# Agent Instructions: $agent_name

## Environment Awareness
- Current environment: $ENVIRONMENT
- Always use cross-platform path handling
- Test on both WSL and native Windows
- Use \`$(detect_environment)\` to check runtime environment

## Progress Tracking
- Update $STATE_DIR/${agent_name}-progress.json after each task
- Create checkpoints in $CHECKPOINT_DIR/${agent_name}/
- Log all activities to $LOGS_DIR/${agent_name}.log
- Use --resume feature if interrupted

## WSL/Windows Compatibility Rules
1. Always use forward slashes in paths
2. Use \`os.path.normpath()\` in Python code
3. Test path resolution with both WSL and Windows paths
4. Handle both .bat and .sh scripts appropriately
5. Check for both python and python.exe

## Fault Tolerance
- Save work incrementally (every 10-15 minutes)
- Create git commits for each completed subtask
- Document decisions in $STATE_DIR/${agent_name}-decisions.md
- If interrupted, resume with: \`claude --resume\`

## Communication
- Check $STATE_DIR/coordinator-messages.json for updates
- Post status to $STATE_DIR/${agent_name}-status.json
- Flag blockers in $STATE_DIR/blockers.json

EOF

    # Add agent-specific instructions
    case "$agent_type" in
        "uxp-research")
            cat >> "$instructions_file" << EOF

## UXP Research Specific Tasks
1. Document all UXP APIs for Premiere and Illustrator
2. Test debugging capabilities on both platforms
3. Research automated deployment methods
4. Create compatibility matrix for different Adobe versions
5. Document performance optimization techniques

Use extended thinking for architectural decisions:
"Think deeply about the UXP plugin architecture..."
EOF
            ;;
        "env-fixes")
            cat >> "$instructions_file" << EOF

## Environment Fix Specific Tasks
1. Fix venv path resolution for both WSL and Windows
2. Create robust environment detection utilities
3. Test with various Python installation methods
4. Handle spaces in paths correctly
5. Create cross-platform activation scripts

Test commands:
- WSL: \`/mnt/c/Program Files/...\`
- Windows: \`C:\\Program Files\\...\`
EOF
            ;;
        "premiere")
            cat >> "$instructions_file" << EOF

## Premiere Development Tasks
1. Implement full timeline manipulation API
2. Add effect application system
3. Create export/render operations
4. Build project management features
5. Ensure all paths work in both environments

Remember to test with actual Premiere Pro on Windows!
EOF
            ;;
    esac
}

# Create progress tracking structure
init_progress_tracking() {
    local agent_name="$1"
    local progress_file="$STATE_DIR/${agent_name}-progress.json"
    
    if [[ ! -f "$progress_file" ]]; then
        cat > "$progress_file" << EOF
{
    "agent": "$agent_name",
    "started": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
    "environment": "$ENVIRONMENT",
    "tasks": {
        "total": 0,
        "completed": 0,
        "in_progress": 0,
        "blocked": 0
    },
    "checkpoints": [],
    "last_update": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
}
EOF
    fi
}

# Setup individual agent
setup_agent() {
    local name="$1"
    local branch="${agents[$name]}"
    local worktree_path="../adobe-mcp-$name"
    
    echo -e "${BLUE}Setting up agent: $name${NC}"
    
    cd "$REPO_ROOT"
    
    # Check if worktree exists
    if git worktree list | grep -q "$worktree_path"; then
        echo -e "${YELLOW}Worktree already exists, updating...${NC}"
        cd "$worktree_path"
        git fetch origin
    else
        # Create new worktree
        echo -e "${GREEN}Creating worktree: $worktree_path${NC}"
        git worktree add "$worktree_path" -b "$branch" || {
            # Branch might already exist
            git worktree add "$worktree_path" "$branch"
        }
    fi
    
    # Create agent-specific files
    create_agent_instructions "$name" "$name"
    init_progress_tracking "$name"
    
    # Create launch script
    local launch_script="$STATE_DIR/launch-${name}.sh"
    cat > "$launch_script" << EOF
#!/bin/bash
# Launch script for $name agent

cd "$(normalize_path "$REPO_ROOT/$worktree_path")"

# Show instructions
cat "$STATE_DIR/${name}-instructions.md"
echo ""
echo "Launching Claude Code for $name agent..."
echo "Use --resume if this session was interrupted"
echo ""

# Set environment variables for WSL/Windows compatibility
export ADOBE_MCP_ENVIRONMENT="$ENVIRONMENT"
export ADOBE_MCP_AGENT="$name"
export ADOBE_MCP_STATE_DIR="$STATE_DIR"
export ADOBE_MCP_CHECKPOINT_DIR="$CHECKPOINT_DIR/$name"

# Create checkpoint directory
mkdir -p "\$ADOBE_MCP_CHECKPOINT_DIR"

# Launch with appropriate profile
case "$name" in
    "uxp-research")
        claude -p research "Review the instructions above, then begin researching Adobe UXP capabilities for Premiere and Illustrator. Focus on automated deployment, debugging, and cross-platform compatibility. Save your findings incrementally."
        ;;
    "env-fixes")
        claude -p systems "Review the instructions above, then fix the WSL vs Windows venv path resolution issues. Create robust cross-platform solutions. Test thoroughly on both environments."
        ;;
    "premiere")
        claude -p development "Review the instructions above, then complete the Premiere Pro implementation. Focus on performance and stability. Test all features on Windows."
        ;;
    "illustrator")
        claude -p development "Review the instructions above, then enhance the Illustrator implementation. Add advanced features while maintaining compatibility."
        ;;
    "infrastructure")
        claude -p systems "Review the instructions above, then implement core infrastructure improvements. Focus on reliability and monitoring."
        ;;
    "testing")
        claude -p testing "Review the instructions above, then create comprehensive test suites. Ensure cross-platform compatibility."
        ;;
    "deployment")
        claude -p automation "Review the instructions above, then create automated UXP plugin deployment. Eliminate all manual steps."
        ;;
esac
EOF
    
    chmod +x "$launch_script"
    echo -e "${GREEN}Created launch script: $launch_script${NC}"
}

# Create coordinator dashboard
create_coordinator_dashboard() {
    local dashboard="$STATE_DIR/dashboard.sh"
    cat > "$dashboard" << 'EOF'
#!/bin/bash
# Adobe MCP Multi-Agent Coordinator Dashboard

STATE_DIR="$(dirname "$0")"

echo "Adobe MCP Multi-Agent Status Dashboard"
echo "======================================"
echo ""

# Show agent status
for progress_file in "$STATE_DIR"/*-progress.json; do
    if [[ -f "$progress_file" ]]; then
        agent=$(basename "$progress_file" -progress.json)
        echo "Agent: $agent"
        jq -r '. | "  Started: \(.started)\n  Tasks: \(.tasks.completed)/\(.tasks.total) completed\n  Status: \(.status // "active")\n  Last Update: \(.last_update)"' "$progress_file"
        echo ""
    fi
done

# Show blockers
if [[ -f "$STATE_DIR/blockers.json" ]]; then
    echo "Current Blockers:"
    jq -r '.[] | "  - [\(.agent)] \(.description)"' "$STATE_DIR/blockers.json" 2>/dev/null || echo "  None"
    echo ""
fi

# Show recent checkpoints
echo "Recent Checkpoints:"
find "$STATE_DIR/../.agent-checkpoints" -name "*.checkpoint" -mtime -1 -exec basename {} \; | head -5
echo ""

echo "Launch scripts available in: $STATE_DIR/launch-*.sh"
EOF
    chmod +x "$dashboard"
}

# Create resume helper
create_resume_helper() {
    local helper="$STATE_DIR/resume-agent.sh"
    cat > "$helper" << 'EOF'
#!/bin/bash
# Resume an interrupted agent session

if [[ $# -eq 0 ]]; then
    echo "Usage: $0 <agent-name>"
    echo "Available agents:"
    ls -1 "$(dirname "$0")"/launch-*.sh | sed 's/.*launch-\(.*\)\.sh/  - \1/'
    exit 1
fi

AGENT="$1"
WORKTREE="../adobe-mcp-$AGENT"

if [[ ! -d "$WORKTREE" ]]; then
    echo "Error: Worktree $WORKTREE not found"
    exit 1
fi

cd "$WORKTREE"
echo "Resuming $AGENT agent session..."
echo "This will show available conversations to resume."
echo ""

claude --resume
EOF
    chmod +x "$helper"
}

# Main setup process
main() {
    echo -e "${BLUE}Starting multi-agent setup...${NC}"
    
    # Phase 1: Research and Environment agents (can start immediately)
    phase1_agents=("uxp-research" "env-fixes" "deployment")
    
    echo -e "\n${YELLOW}Phase 1: Setting up research and environment agents${NC}"
    for agent in "${phase1_agents[@]}"; do
        setup_agent "$agent"
    done
    
    # Phase 2: Development agents (setup but don't start yet)
    phase2_agents=("premiere" "illustrator" "infrastructure")
    
    echo -e "\n${YELLOW}Phase 2: Setting up development agents${NC}"
    for agent in "${phase2_agents[@]}"; do
        setup_agent "$agent"
    done
    
    # Phase 3: Testing agent
    phase3_agents=("testing")
    
    echo -e "\n${YELLOW}Phase 3: Setting up testing agent${NC}"
    for agent in "${phase3_agents[@]}"; do
        setup_agent "$agent"
    done
    
    # Create management tools
    create_coordinator_dashboard
    create_resume_helper
    
    echo -e "\n${GREEN}Setup complete!${NC}"
    echo -e "\nTo launch agents:"
    echo -e "  ${BLUE}Phase 1 (start now):${NC}"
    for agent in "${phase1_agents[@]}"; do
        echo -e "    $STATE_DIR/launch-${agent}.sh"
    done
    
    echo -e "\n  ${BLUE}Phase 2 (after Phase 1 research):${NC}"
    for agent in "${phase2_agents[@]}"; do
        echo -e "    $STATE_DIR/launch-${agent}.sh"
    done
    
    echo -e "\n  ${BLUE}Phase 3 (after Phase 2 development):${NC}"
    for agent in "${phase3_agents[@]}"; do
        echo -e "    $STATE_DIR/launch-${agent}.sh"
    done
    
    echo -e "\n${YELLOW}Management tools:${NC}"
    echo -e "  Dashboard: $STATE_DIR/dashboard.sh"
    echo -e "  Resume interrupted agent: $STATE_DIR/resume-agent.sh <agent-name>"
    echo -e "\n${YELLOW}Important:${NC} Each agent will save progress incrementally."
    echo -e "Use ${BLUE}claude --resume${NC} if any agent session is interrupted."
}

# Run main
main "$@"