# GitHub Copilot Custom Instructions

This project follows the **Agent-Native** architecture defined in `AGENTS.md`.

- Ensure every production `.rs` file begins with a `//!` living wiki header of at least 100 characters.
- Write modular code: active production code must be under 10,000 characters per file; functions must be under 2,000 characters.
- Document all public items (`pub struct`, `pub fn`, `pub enum`, `pub trait`) with `///` doc comments.
- Do not use `.unwrap()` or `.expect()` in production code. Use `Result<T, E>` and `?`.
- Verify with `cargo check`, `cargo test`, and `cargo clippy`.
