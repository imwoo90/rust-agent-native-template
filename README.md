# 🤖 Rust Agent-Native Template

[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![CI](https://img.shields.io/badge/CI-GitHub%20Actions-blue.svg?style=flat-square&logo=githubactions)](https://github.com/imwoo90/rust-agent-native-template/actions)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg?style=flat-square)](LICENSE)

An **Agent-Native** Rust project template engineered for seamless collaboration between human developers and AI coding agents. 

Built around Andrej Karpathy's **LLM-Wiki** philosophy and compiler-enforced quality assurance, this template guarantees that the codebase itself acts as a living, self-documenting, high-density knowledge base with zero documentation drift.

---

## 🌟 Why Agent-Native?

Traditional codebases separate implementation from documentation and rely on manual code reviews to enforce modularity. Over time, external docs rot, prompt context gets polluted with boilerplate, and AI agents struggle with massive monolithic files and hallucinations.

**Agent-Native architecture inverts this paradigm:**
* **The Compiler is the Arbiter**: Modularity, function size, and documentation presence are verified at compile time via `build.rs`.
* **Zero-Hallucination Intra-Doc Links**: Relationships between types and modules are validated by `rustdoc`. If a link breaks, the build breaks.
* **Living Doctests**: Code examples in documentation are compiled and executed during `cargo test`.
* **High Signal-to-Noise Ratio (SNR)**: Strict character budgets keep files compact and optimal for LLM context windows.

---

## 🏛️ Core Architectural Pillars

### 1. 🛡️ Compile-Time Hard Linter (`build.rs`)
Every `cargo check`, `cargo build`, and `cargo test` runs an automated AST/text inspection across `src/`:
* **Rule 1: File Header Requirement**: Every non-test `.rs` file must begin with a `//!` module header of **at least 100 characters** describing its purpose, responsibility, and architecture.
* **Rule 2: Production Logical Code Limit**: Active production code (excluding comments, doc comments, and `#[cfg(test)]` modules) must not exceed **10,000 characters** (~250 SLOC). Preserves TDD incentives while forcing modular decomposition.
* **Rule 3: Documentation Limit**: Documentation comments must not exceed **4,000 characters**. Prevents bloat and keeps context dense.
* **Rule 4: Function Physical Limit**: Individual functions must not exceed **2,000 characters** (~40–50 lines). Enforces single responsibility and fits on a single screen/turn.
* **Rule 5: Living LLM-Wiki: Public API Documentation**: Every public item (`pub fn`, `pub struct`, `pub enum`, `pub trait`, `pub type`, and public inherent method) must have `///` documentation comments.
* **Rule 6: Panic-Free Production Guarantee**: Prohibits `.unwrap()` and `.expect()` in production code, mandating typed error handling (`Result<T, E>` and `?`).

### 2. 🧠 Living LLM-Wiki (`mod.rs` = `index.md`)
* Each directory represents a cohesive module domain whose `mod.rs` serves as the `index.md` architecture catalog.
* Cross-module navigation uses native Rustdoc links (e.g., `[`[`Calculator`](crate::example::Calculator)`]`).

### 3. 🧪 Test-Driven Quality (TDD)
* **Unit Tests**: Co-located within source files (`#[cfg(test)] mod tests`).
* **Integration Tests**: Placed in `tests/` to verify public API contracts.
* **Compiler-Verified Doctests**: Every public interface doc comment includes executable examples verified by `cargo test --doc`.

### 4. 📜 Agent Specification (`AGENTS.md`)
* A shared contract and instruction manual for AI coding assistants (e.g., Antigravity, Claude, Copilot, Cursor).

---

## 📂 Project Layout

```text
rust-agent-native-template/
├── .github/
│   └── workflows/
│       └── ci.yml                 # CI: formatting, linter, tests, intra-doc links
├── src/
│   ├── example/
│   │   ├── mod.rs                 # mod.rs as index.md with architectural summary
│   │   └── calculator.rs          # Calculator engine with doctests and unit tests
│   ├── lib.rs                     # Library entrypoint with crate-level living wiki
│   └── main.rs                    # Binary CLI entrypoint
├── tests/
│   └── integration_test.rs        # External integration test suite
├── AGENTS.md                      # AI Agent & Developer collaboration standards
├── build.rs                       # Compile-time hard linter engine
├── Cargo.toml                     # Cargo manifest (Rust 2024 edition)
├── LICENSE                        # Apache License 2.0
└── README.md
```

---

## 🚀 Getting Started

### 1. Use as a GitHub Template
Click the **"Use this template"** button at the top of the repository to create your new repository.

Or clone directly:
```bash
git clone https://github.com/imwoo90/rust-agent-native-template.git my-project
cd my-project
```

### 2. Verify Your Environment
Run the three essential Agent-Native verification commands:

```bash
# 1. Compile-time linter verification
cargo check

# 2. Run unit tests, integration tests, and doctests
cargo test

# 3. Verify documentation and intra-doc link integrity
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
```

### 3. Run the CLI
```bash
cargo run
```

---

## 🤝 Collaboration with AI Agents

When working with an AI coding assistant in this repository, simply instruct the agent:
> *"Please read `AGENTS.md` before making changes. Ensure all code follows the compile-time limits and pass `cargo test` and `cargo doc`."*

The agent will automatically adhere to the file limits, maintain module-level `//!` documentation, and write executable doctests.

---

## 📄 License

Licensed under the Apache License, Version 2.0 ([LICENSE](LICENSE)).
