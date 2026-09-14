# Rust Agent-Native Guide & Standards (AGENTS.md)

This file defines the project-specific coding guidelines, architectural constraints, and compilation standards for this repository.

---

## 1. Architectural Baseline: Agent-Native Core
This project strictly adheres to the compiler-enforced **Agent-Native** architecture.

👉 **Mandatory Baseline Reading**: Consult [AGENT_NATIVE.md](file://AGENT_NATIVE.md) for the authoritative specification on:
- **Core Philosophy**: Living LLM-Wiki (`mod.rs` = `index.md`), compile-checked intra-doc links, and living doctests.
- **Compiler Hard Limits**: Rules 1-4 enforced via `build.rs` and `build_linter.rs` (module headers, logical code limits, inline test limits, function limits).
- **Anti-Code-Golfing Principle**: Decompose into cohesive submodules; never compress variable names or whitespace.
- **Safety Lints**: `missing_docs = "deny"`, `clippy::unwrap_used = "deny"`, `clippy::expect_used = "deny"` in `Cargo.toml`.

---

## 2. Project Architecture & Domain Knowledge
* Document your project's high-level architecture, module domains, and key data flows here.
* Leverage `mod.rs` in each directory as the domain's `index.md` architecture catalog.
* Cross-module navigation uses native Rustdoc links (e.g. `[`[`Calculator`](crate::example::Calculator)`]`).

---

## 3. Verification Protocol
Always verify code changes before completing any task:

```bash
# 1. Check compiler linter rules (Agent-Native constraints)
cargo check

# 2. Run unit tests, integration tests, and doctests
cargo test --all-targets

# 3. Verify doc link integrity and ensure zero warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps

# 4. Optional: Run Clippy if configured
cargo clippy --all-targets
```

---

## 4. Project-Specific Directives & Guidelines
*(Add any framework-specific guides, domain models, or runtime setup instructions below)*
