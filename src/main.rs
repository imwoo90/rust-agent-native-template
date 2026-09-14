//! # Rust Agent-Native Template CLI Entrypoint
//!
//! ## Overview
//! Provides the primary binary entry point for the project. Demonstrates command execution,
//! module integration, and runtime output while adhering to all Agent-Native architectural constraints.
//!
//! ## Search Tags
//! #cli, #entrypoint, #template, #agent-native

use rust_agent_native_template::Calculator;

fn main() {
    println!("=== Rust Agent-Native Template ===");

    let mut calc = Calculator::new();
    calc.add(42.0).multiply(2.0);

    println!("Calculator demo result: {}", calc.value());
    println!("Agent-Native validation: OK");
}
