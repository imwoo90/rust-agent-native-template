#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# Rust Agent-Native Template Initialization Script
# Re-namespaces the template to a new project name across all source files,
# manifests, doctests, and documentation in a single step.
# ==============================================================================

if [ $# -lt 1 ]; then
    echo "Usage: $0 <new-project-name>"
    echo "Example: $0 my-agent-app"
    exit 1
fi

NEW_PROJECT_NAME="$1"

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
echo ""

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

OLD_KEBAB="rust-agent-native-template"
OLD_SNAKE="rust_agent_native_template"

# 1. Update Cargo.toml
echo "--> Updating Cargo.toml..."
sed -i "s/name = \"${OLD_KEBAB}\"/name = \"${KEBAB_NAME}\"/g" Cargo.toml

# 2. Update src/lib.rs, src/main.rs
echo "--> Updating source files..."
find src -type f -name "*.rs" -exec sed -i "s/${OLD_KEBAB}/${KEBAB_NAME}/g" {} +
find src -type f -name "*.rs" -exec sed -i "s/${OLD_SNAKE}/${SNAKE_NAME}/g" {} +

# 3. Update tests/
echo "--> Updating integration and linter tests..."
find tests -type f -name "*.rs" -exec sed -i "s/${OLD_SNAKE}/${SNAKE_NAME}/g" {} +
find tests -type f -name "*.rs" -exec sed -i "s/${OLD_KEBAB}/${KEBAB_NAME}/g" {} +

# 4. Update README.md
echo "--> Updating README.md..."
sed -i "s/# 🤖 Rust Agent-Native Template/# 🤖 ${KEBAB_NAME}/g" README.md
sed -i "s/${OLD_KEBAB}/${KEBAB_NAME}/g" README.md
sed -i "s/${OLD_SNAKE}/${SNAKE_NAME}/g" README.md

# 5. Verify build and test suite under new identity
echo "--> Verifying compile-time constraints and test suite..."
cargo check --all-targets
cargo test --all-targets
cargo test --doc

echo ""
echo "=== Project successfully re-namespaced to '${KEBAB_NAME}'! ==="
echo "All compiler constraints, doctests, and unit tests passed."
