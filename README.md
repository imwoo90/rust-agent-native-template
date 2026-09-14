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
Every `cargo check`, `cargo build`, and `cargo test` runs an automated AST inspection across `src/`:
* **Rule 1: File Header Requirement**: Every non-test `.rs` file must begin with a `//!` module header of **at least 100 characters** describing its purpose, responsibility, and architecture.
* **Rule 2: Production Logical Code Limit**: Active production code (excluding comments, doc comments, and `#[cfg(test)]` modules) must not exceed **10,000 characters** (~250 SLOC). Preserves TDD incentives while forcing modular decomposition.
* **Rule 3: Documentation Limit**: Documentation comments must not exceed **4,000 characters**. Prevents bloat and keeps context dense.
* **Rule 4: Function Physical Limit**: Individual functions must not exceed **2,000 characters** (~40–50 lines). Enforces single responsibility and fits on a single screen/turn.

### 2. ⚙️ Standard Quality & Safety Lints (`Cargo.toml [lints]`)
* **Living LLM-Wiki Public API Contract**: `missing_docs = "deny"` mandates executable doc comments (`///`) on all public interfaces.
* **Panic-Free Production Guarantee**: `clippy::unwrap_used = "deny"` and `clippy::expect_used = "deny"` enforce typed `Result<T, E>` and the `?` operator in production.
* **Real-Time Editor Checking**: Configured via `.vscode/settings.json` (`"rust-analyzer.check.command": "clippy"`).

### 3. 🧠 Living LLM-Wiki (`mod.rs` = `index.md`)
* Each directory represents a cohesive module domain whose `mod.rs` serves as the `index.md` architecture catalog.
* Cross-module navigation uses native Rustdoc links (e.g., `[`[`Calculator`](crate::example::Calculator)`]`).

### 4. 🧪 Test-Driven Quality (TDD)
* **Unit Tests**: Co-located within source files (`#[cfg(test)] mod tests`).
* **Integration Tests**: Placed in `tests/` to verify public API contracts.
* **Compiler-Verified Doctests**: Every public interface doc comment includes executable examples verified by `cargo test --doc`.

### 4. 📜 Agent Specification (`AGENTS.md`)
* A shared contract and instruction manual for AI coding assistants (e.g., Antigravity, Claude, Copilot, Cursor).

---

## 📂 Project Layout

```text
rust-agent-native-template/
├── .cursor/
│   └── rules/agent-native.mdc     # Cursor IDE rules bridge
├── .github/
│   ├── workflows/ci.yml           # CI: formatting, linter, tests, intra-doc links
│   └── copilot-instructions.md    # GitHub Copilot instructions bridge
├── .vscode/
│   └── settings.json              # IDE real-time Clippy on save
├── scripts/
│   └── init.sh                    # 1-second automated project re-namespacing script
├── src/
│   ├── example/
│   │   ├── mod.rs                 # mod.rs as index.md with architectural summary
│   │   └── calculator.rs          # Calculator engine with doctests and unit tests
│   ├── lib.rs                     # Library entrypoint with crate-level living wiki
│   └── main.rs                    # Binary CLI entrypoint
├── tests/
│   ├── integration_test.rs        # External integration test suite
│   └── linter_test.rs             # Compile-time AST linter TDD verification suite
├── AGENTS.md                      # AI Agent & Developer collaboration standards (SSOT)
├── CLAUDE.md                      # Claude Code bridge (points to AGENTS.md)
├── GEMINI.md                      # Antigravity / Gemini bridge (points to AGENTS.md)
├── build.rs                       # Compile-time hard linter engine (Rules 1-4)
├── Cargo.toml                     # Cargo manifest ([lints] table and dependencies)
├── cargo-generate.toml            # cargo-generate template configuration
├── LICENSE                        # Apache License 2.0
└── README.md
```

---

## 🚀 Getting Started

### Option A: Using `cargo-generate` (Recommended)
```bash
# Generate a new project interactively
cargo generate imwoo90/rust-agent-native-template --name my-project
cd my-project
```

### Option B: Using GitHub Template / Direct Clone
```bash
# 1. Clone repository
git clone https://github.com/imwoo90/rust-agent-native-template.git my-project
cd my-project

# 2. Initialize and re-namespace to your project name in 1 second
./scripts/init.sh my-project
```

### 2. Verify Your Environment
Run the essential Agent-Native verification suite:

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
