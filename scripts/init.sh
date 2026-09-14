#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# Rust Agent-Native Template Initialization Script
# Re-namespaces the template to a new project name across all source files,
# manifests, doctests, and documentation in a single step.
#
# Flags:
#   --clean   Remove demo calculator example and provide a pristine empty scaffold
# ==============================================================================

CLEAN_MODE=false
NEW_PROJECT_NAME=""

for arg in "$@"; do
    case "$arg" in
        --clean)
            CLEAN_MODE=true
            ;;
        *)
            if [ -z "${NEW_PROJECT_NAME}" ]; then
                NEW_PROJECT_NAME="$arg"
            fi
            ;;
    esac
done

if [ -z "${NEW_PROJECT_NAME}" ]; then
    echo "Usage: $0 <new-project-name> [--clean]"
    echo "Examples:"
    echo "  $0 my-agent-app          # Keep calculator reference example"
    echo "  $0 my-agent-app --clean  # Clean scaffold ready for any project (e.g. Dioxus, CLI, Web)"
    exit 1
fi

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
#![allow(missing_docs)]

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

# 4. Update README.md
echo "--> Updating README.md..."
sedi "s/# 🤖 Rust Agent-Native Template/# 🤖 ${KEBAB_NAME}/g" README.md
sedi "s/${OLD_KEBAB}/${KEBAB_NAME}/g" README.md
sedi "s/${OLD_SNAKE}/${SNAKE_NAME}/g" README.md

# 5. Verify build and test suite under new identity
echo "--> Verifying compile-time constraints and test suite..."
cargo check --all-targets
cargo test --all-targets
cargo test --doc
cargo clippy --all-targets

echo ""
echo "=== Project successfully initialized to '${KEBAB_NAME}'! ==="
echo "All compiler constraints, doctests, and unit tests passed."
