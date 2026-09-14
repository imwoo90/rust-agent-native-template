//! # Example Submodule: Architecture & Computation Catalog
//!
//! This module showcases the **`mod.rs` = `index.md`** pattern. It acts as the high-level
//! architectural catalog for the computation sub-domain, detailing component boundaries,
//! data flows, and public interface re-exports.
//!
//! ## Submodules
//! - [`calculator`]: Core calculation engine providing verified basic arithmetic operations.
//!
//! ## Search Tags
//! #example, #calculator, #arithmetic

pub mod calculator;

pub use calculator::Calculator;
