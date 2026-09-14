# Claude Code Configuration & Agent-Native Standards

This repository adheres to the **Agent-Native** architecture and compiler-enforced quality constraints defined in [AGENTS.md](file://AGENTS.md).

## Critical Guidelines for Claude Code

1. **Single Source of Truth**:
   - Always read [AGENTS.md](file://AGENTS.md) before planning or making architectural changes.
   - Every `.rs` file in `src/` must begin with a module-level doc comment (`//!`) of **at least 100 characters** describing its purpose, responsibilities, and architecture (AGENTS.md Rule 1).

2. **Compile-Time Architectural Limits (`build.rs`)**:
   - **Production Code Budget**: Keep active production code lines under **10,000 characters** per file (excluding comments and `#[cfg(test)]` blocks).
   - **Documentation Budget**: Keep doc comments under **4,000 characters** per file to preserve LLM context density.
   - **Function Budget**: Every function must be under **2,000 characters** (~40–50 lines). Decompose large functions into single-responsibility helpers.

3. **Public API Contracts & Panic-Free Guarantee (`Cargo.toml [lints]`)**:
   - Every public item (`pub fn`, `pub struct`, `pub enum`, `pub trait`, `pub type`, and public inherent method) must have `///` doc comments with executable doctests (`missing_docs = "deny"`).
   - **Never** use `.unwrap()` or `.expect()` in production code. Use typed `Result<T, E>` and `?` error propagation (`clippy::unwrap_used = "deny"`).

4. **Required Verification Suite**:
   Never consider a task complete without executing and passing:
   ```bash
   cargo check
   cargo test --all-targets
   cargo test --doc
   cargo clippy --all-targets
   RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
   ```
