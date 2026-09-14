# Rust Agent-Native Guide & Standards (AGENTS.md)

This file defines the coding guidelines, architectural constraints, and compilation standards for the project. The codebase follows an **Agent-Native** design based on the **"Code as Documentation"** philosophy, optimized for high-efficiency collaboration between human developers and AI Agents.

---

## 1. Core Philosophy: Code as Documentation (LLM-Wiki)

To prevent documentation drift and maximize signal-to-noise ratio in LLM context windows, the Rust source code itself serves as the single source of truth for implementation, architecture, and documentation.

1. **`mod.rs` = `index.md`**:
   * Leveraging Rust's module tree, every directory's `mod.rs` acts as the `index.md` (table of contents & architecture catalog) for that module.
   * Use module-level doc comments (`//!`) at the top of `mod.rs` to detail high-level architectural responsibilities, data flows, and design decisions in Markdown.
2. **Compiler-Verified Documentation**:
   * Never write documentation that cannot be compiled. Structs, traits, and public functions must use standard doc comments (`///`) with executable code examples (`doctests`).
   * When running `cargo test`, the compiler executes all doctests, guaranteeing documentation never goes stale.
3. **Compile-Checked Intra-Doc Links**:
   * Reference other types, modules, or functions using Rust's native intra-doc link syntax (e.g., `[`[`Calculator`](crate::example::Calculator)`]`).
   * Rustdoc validates every link at build time (`RUSTDOCFLAGS="-D warnings" cargo doc --no-deps`), eliminating broken links and hallucinations.

---

## 2. LLM-Agent Constraints (Enforced at Compile-Time via `build.rs`)

To keep files compact, modular, and optimized for LLM context windows, strict architectural limits are enforced during `cargo check`, `cargo build`, and `cargo test`:

1. **Rule 1: File-Level Living Wiki Header (Min 100 Characters)**:
   * Every non-test production `.rs` file must begin with a file-level doc comment (`//!`) of **at least 100 characters** describing its purpose, responsibilities, and architecture.
2. **Rule 2: Production Logical Code Limit (Max 10,000 Characters)**:
   * The total character count of active production code lines (excluding comments, doc comments, empty lines, and `#[cfg(test)]` blocks) must be **under 10,000 characters** (approx. 200–300 lines of SLOC).
   * Note: Comprehensive unit tests in `#[cfg(test)]` do not count against this limit, fully preserving TDD incentives.
   * Exceeding this limit indicates bloated responsibility; split into cohesive submodules.
   * *Escape Hatch*: For declarative UI files or code-generated suites, annotate the file with `#![allow(clippy::too_many_lines)]` to bypass this constraint.
3. **Rule 3: File Documentation Limit (Max 4,000 Characters)**:
   * The total character count of documentation comments (`//`, `///`, `//!`, `/* */`) must be **under 4,000 characters** (approx. 50–80 lines).
   * This forces descriptions to remain concise and high-signal, preventing LLM context bloat.
4. **Rule 4: Function Physical Size Limit (Max 2,000 Characters)**:
   * A single function (including its signature, body, comments, and braces) must be **under 2,000 characters** (approx. 40–50 physical lines).
   * Ensures every function fits cleanly on a single screen or within a single context window turn.
   * *Escape Hatch*: For macro-heavy UI components (e.g. Dioxus `rsx!`), large router tables (e.g. Axum), or complex state machines that cannot be cleanly split, annotate the function with `#[allow(clippy::too_many_lines)]` to explicitly bypass this check.

---

## 3. Standard Quality & Safety Lints (Enforced via `Cargo.toml [lints]`)

Leveraging Rust 1.74+ package-level lint declarations, standard compiler and Clippy lints are elevated to hard compilation errors:

1. **`missing_docs = "deny"` (Rust Standard Lint)**:
   * Enforces the Living LLM-Wiki contract: every public struct, enum, function, trait, and type alias must have outer doc comments (`///`).
2. **`clippy::unwrap_used = "deny"` & `clippy::expect_used = "deny"` (Clippy Standard Lints)**:
   * Prohibits `.unwrap()` and `.expect()` in production code, enforcing typed error handling (`Result<T, E>` and `?`).
   * Editor real-time checking is configured via `.vscode/settings.json` (`"rust-analyzer.check.command": "clippy"`).

---

## 4. Development Workflow (TDD)

When contributing or adding features, always adhere to the Test-Driven Development (TDD) cycle:

1. **Write Failing Tests First**: Write a failing unit test (`#[cfg(test)]`) or doctest describing the expected behavior.
2. **Implement Code & Docs**: Write the implementation alongside concise, accurate doc comments.
3. **Verify Compliance**:
   ```bash
   # 1. Check compiler linter rules
   cargo check

   # 2. Run unit tests, integration tests, and doctests
   cargo test

   # 3. Verify doc link integrity and ensure zero warnings
   RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
   ```

---

## 5. AI Agent Navigation Guide

For AI Agents interacting with this repository:

1. **Entry Points**: Start with `AGENTS.md` (this file) and `README.md` to grasp project guidelines and compilation rules.
2. **Module Indexing**: Treat `mod.rs` in any directory as the module's architecture map. Read the `//!` header to understand submodules and dependencies before diving into child files.
3. **Graph Traversal via Rustdoc Links**: Follow compile-checked intra-doc links (`[Type]`) to traverse dependencies deterministically.
4. **Pre-commit Verification**: Never consider a task complete without passing `cargo check`, `cargo test`, and `cargo doc`.
