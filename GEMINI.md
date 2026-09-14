# Antigravity / Gemini Workspace Prompt

This project follows the **Agent-Native** architecture and compiler-enforced quality constraints defined in [AGENTS.md](file://AGENTS.md).

## Core Directives

1. **Architecture & Philosophy**:
   - Source code serves as the living LLM-Wiki (`mod.rs` = `index.md`).
   - Strict compile-time constraints are enforced during `cargo check` and `cargo build` via `build.rs` and `Cargo.toml [lints]`.

2. **Compile-Time Limits (Hard Build Failure)**:
   - **Rule 1**: Every production `.rs` file must begin with a `//!` header of at least 100 characters.
   - **Rule 2**: Production logical code must not exceed 10,000 characters per file (excluding `#[cfg(test)]`).
   - **Rule 3**: Documentation comments must not exceed 4,000 characters per file.
   - **Rule 4**: Individual functions must not exceed 2,000 characters.
   - **Package Lints**: All public interfaces require `///` doc comments (`missing_docs = "deny"`), and `.unwrap()`/`.expect()` are prohibited in production (`clippy::unwrap_used = "deny"`).

3. **Verification**:
   Always verify code changes with:
   ```bash
   cargo check
   cargo test --all-targets
   cargo test --doc
   cargo clippy --all-targets
   RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
   ```
