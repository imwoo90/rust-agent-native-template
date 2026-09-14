# Agent-Native Architecture Standards & Guidelines (AGENT_NATIVE.md)

This specification defines the universal architectural standards, compile-time constraints, and development workflows for **Agent-Native** Rust codebases.

Built upon the **"Code as Documentation" (Living LLM-Wiki)** philosophy, this framework turns the Rust compiler into an active collaborator and strict arbiter, ensuring codebases remain compact, modular, and self-documenting with zero documentation drift.

---

## 1. Core Philosophy: Code as Documentation (LLM-Wiki)

To prevent documentation rot and maximize the signal-to-noise ratio (SNR) in LLM context windows, the Rust source code itself acts as the living knowledge base for architecture, interfaces, and design decisions.

1. **`mod.rs` = `index.md`**:
   * Every directory's `mod.rs` serves as the `index.md` (table of contents and architectural map) for that module.
   * Module-level doc comments (`//!`) at the top of `mod.rs` describe high-level responsibilities, domain boundaries, data flows, and design rationale.
2. **Compiler-Verified Documentation**:
   * Never write documentation that cannot be compiled. Structs, traits, and public functions must use standard doc comments (`///`) with executable code examples (`doctests`).
   * When running `cargo test`, the compiler executes all doctests, guaranteeing documentation never becomes obsolete.
3. **Compile-Checked Intra-Doc Links**:
   * Reference other types, modules, or functions using Rust's native intra-doc link syntax (e.g. `[`[`Calculator`](crate::example::Calculator)`]`).
   * Rustdoc validates every link at build time (`RUSTDOCFLAGS="-D warnings" cargo doc --no-deps`), eliminating broken references and hallucinations.

---

## 2. LLM-Agent Constraints (Enforced at Compile-Time via `build.rs` & `build_linter.rs`)

To prevent monoliths and keep files optimized for LLM context windows, strict AST constraints are enforced during `cargo check`, `cargo build`, and `cargo test`:

1. **Rule 1: File-Level Living Wiki Header (Min 100 Characters)**:
   * Every non-test production `.rs` file must begin with a file-level doc comment (`//!`) of **at least 100 characters** describing its purpose, responsibilities, and architecture.
2. **Rule 2: Production Logical Code Limit (Max 10,000 Characters)**:
   * The character count of active production code (excluding comments, doc comments, blank lines, and `#[cfg(test)]` modules) must not exceed **10,000 characters** (~200–300 lines of SLOC).
   * Exceeding this limit indicates bloated responsibility; split into cohesive submodules.
   * *Strict Enforcement*: Limits cannot be bypassed via code attributes (`#[allow(...)]`). To adjust project-wide ceilings, configure `.agent-lint.toml`.
3. **Rule 2b: Inline Unit Test Limit in `src/` (Max 5,000 Characters)**:
   * Inline unit tests (`#[cfg(test)]`) within a `src/` file must not exceed **5,000 characters**.
   * When unit tests grow beyond 5,000 characters:
     - **Integration tests** (public API): move to the root `tests/` directory (e.g. `tests/<module>_test.rs`).
     - **Unit tests** (requiring access to private/crate items): extract into a dedicated submodule file (e.g. `src/<module>/tests.rs` or `src/<module>_tests.rs` with `#[cfg(test)] mod tests;`).
     This preserves encapsulation while keeping production files compact for LLM context windows.
4. **Rule 3: File Documentation Limit (Max 4,000 Characters)**:
   * Total character count of documentation comments (`//`, `///`, `//!`, `/* */`) must not exceed **4,000 characters** (~50–80 lines).
   * Keeps descriptions concise and high-signal, preventing LLM prompt bloat.
5. **Rule 4: Function Physical Size Limit (Max 2,000 Characters)**:
   * A single function (production or test, including signature, body, comments, and braces) must not exceed **2,000 characters** (~40–50 physical lines).
   * Ensures every function fits cleanly on a single screen or within a single context window turn. Oversized test functions must be refactored into smaller test cases or helper assertions.
   * *Strict Enforcement*: Functions cannot bypass limits via code attributes (`#[allow(...)]`). Decompose into smaller helper functions.
6. **Anti-Code-Golfing Principle**:
   * Never compress variable names (e.g. `transaction_context` -> `tc`), eliminate idiomatic whitespace/newlines, or abuse macros to artificially circumvent character limits. Limits exist to force clean architectural decomposition into cohesive submodules and helper functions.

---

## 3. Standard Quality & Safety Lints (Enforced via `Cargo.toml [lints]`)

Leveraging Rust 1.74+ package-level lint declarations, standard compiler and Clippy lints are elevated to hard compilation errors:

1. **`missing_docs = "deny"` (Rust Standard Lint)**:
   * Enforces the Living LLM-Wiki contract: every public struct, enum, function, trait, and type alias must have outer doc comments (`///`).
2. **`clippy::unwrap_used = "deny"` & `clippy::expect_used = "deny"` (Clippy Standard Lints)**:
   * Prohibits `.unwrap()` and `.expect()` in production code, enforcing typed error handling (`Result<T, E>` and `?`).
   * *Test Exception*: In tests (`tests/` or `#[cfg(test)]`), you may annotate `#![allow(clippy::unwrap_used, clippy::expect_used)]` or return `Result<(), Box<dyn std::error::Error>>` to streamline assertions.

---

## 4. Development Workflow (TDD & Verification Baseline)

When implementing features or bug fixes, always adhere to the verification cycle:

1. **Write Failing Tests First**: Write unit tests or doctests describing expected behavior before implementation.
2. **Implement Code & Docs**: Write clean, idiomatic code alongside concise, accurate doc comments.
3. **Verify Baseline Compliance**:
   ```bash
   # 1. Check compiler linter rules (Agent-Native constraints)
   cargo check

   # 2. Run unit tests, integration tests, and doctests
   cargo test --all-targets

   # 3. Verify doc link integrity and ensure zero warnings
   RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
   ```

---

## 5. AI Agent Navigation Guide

For AI coding agents interacting with an Agent-Native repository:

1. **Entry Points**: Always start with `AGENTS.md` and `README.md` to grasp project guidelines, domain architecture, and verification commands.
2. **Module Indexing**: Treat `mod.rs` in any directory as the module's architecture map. Read the `//!` header to understand submodules and dependencies before diving into child files.
3. **Deterministic Traversal**: Follow compile-checked intra-doc links (`[Type]`) to traverse dependencies deterministically without blind full-codebase searching.
4. **Pre-commit Verification**: Never consider a task complete without passing the verification commands specified in the project's `AGENTS.md`.
