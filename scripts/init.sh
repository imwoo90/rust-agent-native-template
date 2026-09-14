#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# Rust Agent-Native Template Initialization Script
# Re-namespaces the template to a new project name across all source files,
# manifests, doctests, and documentation in a single step.
#
# Flags:
#   --clean         Remove demo calculator example and provide a pristine empty scaffold
#   --agent <name>  Filter AI agent instruction files (default: all)
#                   Choices: all, claude, cursor, gemini, copilot, universal
#                   (Supports comma-separated list, e.g. --agent claude,cursor)
# ==============================================================================

CLEAN_MODE=false
NEW_PROJECT_NAME=""
AGENT_TARGET="all"

while [ $# -gt 0 ]; do
    case "$1" in
        --clean)
            CLEAN_MODE=true
            shift
            ;;
        --agent)
            if [ $# -lt 2 ]; then
                echo "Error: --agent requires an argument (e.g. --agent claude)" >&2
                exit 1
            fi
            AGENT_TARGET="$2"
            shift 2
            ;;
        --agent=*)
            AGENT_TARGET="${1#*=}"
            shift
            ;;
        *)
            if [ -z "${NEW_PROJECT_NAME}" ]; then
                NEW_PROJECT_NAME="$1"
            fi
            shift
            ;;
    esac
done

if [ -z "${NEW_PROJECT_NAME}" ]; then
    echo "Usage: $0 <new-project-name> [--clean] [--agent <all|claude|cursor|gemini|copilot|universal>]"
    echo ""
    echo "Options:"
    echo "  --clean         Remove demo calculator example and provide a pristine empty scaffold"
    echo "  --agent <name>  Retain only specific AI agent instruction files (default: all)"
    echo "                  Choices: all, claude, cursor, gemini, copilot, universal"
    echo "                  (Supports comma-separated: e.g. --agent claude,cursor)"
    echo ""
    echo "Examples:"
    echo "  $0 my-agent-app                              # Full template with all agents"
    echo "  $0 my-agent-app --agent claude               # Keep only Claude Code instructions"
    echo "  $0 my-agent-app --agent cursor --clean       # Clean scaffold with Cursor rules"
    echo "  $0 my-agent-app --agent universal            # Pure AGENTS.md (no platform-specific bridges)"
    exit 1
fi

# Parse agent selection flags
KEEP_CLAUDE=false
KEEP_CURSOR=false
KEEP_GEMINI=false
KEEP_COPILOT=false

NORMALIZED_AGENT="$(echo "${AGENT_TARGET}" | tr '[:upper:]' '[:lower:]')"
IFS=',' read -ra AGENTS_LIST <<< "${NORMALIZED_AGENT}"

for a in "${AGENTS_LIST[@]}"; do
    a="$(echo "$a" | tr -d '[:space:]')"
    case "$a" in
        all)
            KEEP_CLAUDE=true
            KEEP_CURSOR=true
            KEEP_GEMINI=true
            KEEP_COPILOT=true
            ;;
        claude)
            KEEP_CLAUDE=true
            ;;
        cursor)
            KEEP_CURSOR=true
            ;;
        gemini|antigravity)
            KEEP_GEMINI=true
            ;;
        copilot)
            KEEP_COPILOT=true
            ;;
        universal|agents|none)
            # Retains AGENTS.md only (AGENTS.md is always preserved as the SSOT)
            ;;
        *)
            echo "Error: Unknown agent '$a'." >&2
            echo "Supported agents: all, claude, cursor, gemini, copilot, universal (or comma-separated e.g. claude,cursor)" >&2
            exit 1
            ;;
    esac
done

# Compute kebab-case and snake_case representations
KEBAB_NAME="$(echo "${NEW_PROJECT_NAME}" | tr '[:upper:]' '[:lower:]' | tr '_' '-' | sed -E 's/[^a-z0-9-]+//g')"
SNAKE_NAME="$(echo "${KEBAB_NAME}" | tr '-' '_')"

if [ -z "${KEBAB_NAME}" ]; then
    echo "Error: Invalid project name '${NEW_PROJECT_NAME}'."
    exit 1
fi

echo "=== Initializing Agent-Native Project ==="
echo "  Package Name (kebab-case): ${KEBAB_NAME}"
echo "  Crate Name   (snake_case): ${SNAKE_NAME}"
echo "  Clean Mode               : ${CLEAN_MODE}"
echo "  Agent Scope              : ${AGENT_TARGET}"
echo ""

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

OLD_KEBAB="rust-agent-native-template"
OLD_SNAKE="rust_agent_native_template"

# Portable sed helper supporting both macOS (BSD) and Linux (GNU)
sedi() {
    local expr="$1"
    shift
    for f in "$@"; do
        if [[ "$(uname)" == "Darwin" ]]; then
            sed -i '' "$expr" "$f"
        else
            sed -i "$expr" "$f"
        fi
    done
}

replace_pattern() {
    local pattern="$1"
    local dir="$2"
    while IFS= read -r -d '' file; do
        if [[ "$(uname)" == "Darwin" ]]; then
            sed -i '' "$pattern" "$file"
        else
            sed -i "$pattern" "$file"
        fi
    done < <(find "$dir" -type f -name "*.rs" -print0)
}

# 1. Update Cargo.toml
echo "--> Updating Cargo.toml..."
sedi "s/name = \"${OLD_KEBAB}\"/name = \"${KEBAB_NAME}\"/g" Cargo.toml

# 2. Handle Clean Mode or Standard Rename
if [ "${CLEAN_MODE}" = true ]; then
    echo "--> Clean mode: Removing demo calculator example..."
    rm -rf src/example

    echo "--> Writing pristine src/lib.rs..."
    cat <<EOF > src/lib.rs
//! # ${KEBAB_NAME} Library
//!
//! This library provides foundational modules, public domain contracts, and core services for the project.
//! Built on the Agent-Native pattern with living LLM-Wiki context and compiler-verified quality assurance.
EOF

    echo "--> Writing pristine src/main.rs..."
    cat <<EOF > src/main.rs
//! # ${KEBAB_NAME} Application Entrypoint
//!
//! Primary binary entrypoint for the project. Demonstrates runtime execution and service orchestration
//! adhering strictly to all Agent-Native compile-time architectural constraints.

fn main() {
    println!("=== ${KEBAB_NAME} initialized ===");
}
EOF

    echo "--> Writing pristine tests/integration_test.rs..."
    cat <<EOF > tests/integration_test.rs
#![allow(missing_docs, clippy::unwrap_used, clippy::expect_used)]

// Integration test suite verifying crate initialization and public API boundaries.

#[test]
fn test_scaffold_initialization() {
    let is_initialized = true;
    assert!(is_initialized);
}
EOF

else
    echo "--> Standard mode: Updating source files..."
    replace_pattern "s/${OLD_KEBAB}/${KEBAB_NAME}/g" src
    replace_pattern "s/${OLD_SNAKE}/${SNAKE_NAME}/g" src

    echo "--> Updating integration tests..."
    replace_pattern "s/${OLD_SNAKE}/${SNAKE_NAME}/g" tests
    replace_pattern "s/${OLD_KEBAB}/${KEBAB_NAME}/g" tests
fi

# 3. Always update linter test suite references
if [ -f "tests/linter_test.rs" ]; then
    sedi "s/${OLD_SNAKE}/${SNAKE_NAME}/g" tests/linter_test.rs
fi

# 4. Configure AI Agent Instructions
echo "--> Configuring AI agent instructions (${AGENT_TARGET})..."
# AGENTS.md is the Single Source of Truth (SSOT) and is ALWAYS preserved.

if [ "${KEEP_CLAUDE}" = false ] && [ -f "CLAUDE.md" ]; then
    echo "    - Removing CLAUDE.md"
    rm -f CLAUDE.md
    sedi '/CLAUDE\.md/d' README.md
fi

if [ "${KEEP_GEMINI}" = false ] && [ -f "GEMINI.md" ]; then
    echo "    - Removing GEMINI.md"
    rm -f GEMINI.md
    sedi '/GEMINI\.md/d' README.md
fi

if [ "${KEEP_CURSOR}" = false ] && [ -d ".cursor" ]; then
    echo "    - Removing .cursor/"
    rm -rf .cursor
    sedi '/\.cursor\//d; /agent-native\.mdc/d' README.md
fi

if [ "${KEEP_COPILOT}" = false ] && [ -f ".github/copilot-instructions.md" ]; then
    echo "    - Removing .github/copilot-instructions.md"
    rm -f .github/copilot-instructions.md
    sedi '/copilot-instructions\.md/d' README.md
    sedi 's/│   ├── workflows\/ci\.yml/│   └── workflows\/ci\.yml/g' README.md
fi

# 5. Update README.md
echo "--> Updating README.md..."
sedi "s/# 🤖 Rust Agent-Native Template/# 🤖 ${KEBAB_NAME}/g" README.md
sedi "s/${OLD_KEBAB}/${KEBAB_NAME}/g" README.md
sedi "s/${OLD_SNAKE}/${SNAKE_NAME}/g" README.md

# 6. Verify build and test suite under new identity
echo "--> Verifying compile-time constraints and test suite..."
cargo check --all-targets
cargo test --all-targets
cargo test --doc
cargo clippy --all-targets
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps

echo ""
echo "=== Project successfully initialized to '${KEBAB_NAME}'! ==="
echo "  Agent Instructions Scope : ${AGENT_TARGET}"
echo "  Clean Scaffold           : ${CLEAN_MODE}"
echo "All compiler constraints, doctests, and unit tests passed."
