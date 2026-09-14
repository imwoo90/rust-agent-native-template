#![allow(missing_docs, clippy::unwrap_used, clippy::expect_used)]

#[path = "../build.rs"]
mod linter;

use std::path::Path;

#[test]
fn test_valid_file_passes() {
    let source = r#"//! # Valid Test Module
//!
//! This is a valid test module designed to verify that the compile-time AST linter correctly accepts
//! production source files that meet all file-header, character count, and function size requirements.

/// A sample public struct.
pub struct Sample;

impl Sample {
    /// Executes the sample operation.
    pub fn execute(&self) -> bool {
        true
    }
}
"#;
    let res = linter::check_source(Path::new("src/sample.rs"), source);
    assert!(res.is_ok(), "Expected valid source to pass: {:?}", res);
}

#[test]
fn test_missing_file_header_fails_rule_1() {
    let source = r#"//! Short header

pub fn run() {}
"#;
    let res = linter::check_source(Path::new("src/sample.rs"), source);
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(err.contains("Rule 1"), "Error was: {}", err);
}

#[test]
fn test_production_file_with_test_in_name_still_enforces_rules() {
    // Files like `src/contest.rs`, `src/attestation.rs`, or `src/latest.rs`
    // must NOT bypass Rule 1 or Rule 2 simply because their name contains the substring "test".
    let source = r#"//! Short header

pub fn run() {}
"#;
    let res = linter::check_source(Path::new("src/contest.rs"), source);
    assert!(
        res.is_err(),
        "Expected production file with 'test' in name to enforce Rule 1"
    );
    let err = res.unwrap_err();
    assert!(err.contains("Rule 1"), "Error was: {}", err);
}

#[test]
fn test_syntax_error_passes_gracefully_to_rustc() {
    // When code has a syntax error/typo, build.rs should gracefully pass through to rustc
    // so that rustc can emit its rich diagnostic messages with spans and suggestions.
    let source = r#"//! # Broken Syntax Test Module
//!
//! Intentionally contains invalid syntax to verify that build.rs does not mask compiler errors.

pub fn broken( {
"#;
    let res = linter::check_source(Path::new("src/broken.rs"), source);
    assert!(
        res.is_ok(),
        "Expected syntax error to pass through to rustc: {:?}",
        res
    );
}

#[test]
fn test_excessive_logical_code_fails_rule_2() {
    // Generate 20 functions of ~700 characters each (total > 14,000 chars, each function < 2,000 chars)
    let mut big_body = String::new();
    for f in 0..20 {
        big_body.push_str(&format!("pub fn func_{}() {{\n", f));
        for i in 0..30 {
            big_body.push_str(&format!("    let _var_{}_{} = {};\n", f, i, i));
        }
        big_body.push_str("}\n\n");
    }

    let source = format!(
        r#"//! # Excessive Code Module
//!
//! This module intentionally exceeds the 10,000-character logical code budget across multiple
//! functions to verify that AGENTS.md Rule 2 triggers and halts compilation.

{}
"#,
        big_body
    );

    let res = linter::check_source(Path::new("src/excessive.rs"), &source);
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(
        err.contains("Rule 2"),
        "Expected Rule 2 violation, got: {}",
        err
    );
}

#[test]
fn test_large_test_module_does_not_penalize_production_code_limit() {
    // Generate 10 test functions, each ~1,000 chars (total > 10,000 chars of tests)
    let mut big_test_code = String::new();
    for f in 0..10 {
        big_test_code.push_str(&format!("    #[test]\n    fn test_case_{}() {{\n", f));
        for i in 0..30 {
            big_test_code.push_str(&format!("        let _test_var_{}_{} = {};\n", f, i, i));
        }
        big_test_code.push_str("    }\n\n");
    }

    let source = format!(
        r#"//! # Test-Heavy Module
//!
//! Verifies that comprehensive unit tests inside `#[cfg(test)]` do not count against the
//! production logical code limit of 10,000 characters (AGENTS.md Rule 2).

/// A public production worker struct.
pub struct Worker;

impl Worker {{
    /// Runs a single production task safely.
    pub fn perform_task(&self) -> i32 {{
        42
    }}
}}

#[cfg(test)]
mod tests {{
    use super::*;

{}
}}
"#,
        big_test_code
    );

    let res = linter::check_source(Path::new("src/worker.rs"), &source);
    assert!(
        res.is_ok(),
        "Expected test code to be excluded from production code limit, got: {:?}",
        res
    );
}

#[test]
fn test_excessive_documentation_fails_rule_3() {
    let mut big_doc = String::new();
    for i in 0..80 {
        big_doc.push_str(&format!(
            "//! Documentation line {:03} with verbose explanation to exceed character limit.\n",
            i
        ));
    }

    let source = format!(
        r#"//! # Excessive Doc Module
//!
{}
pub fn short_code() {{}}
"#,
        big_doc
    );

    let res = linter::check_source(Path::new("src/verbose.rs"), &source);
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(
        err.contains("Rule 3"),
        "Expected Rule 3 violation, got: {}",
        err
    );
}

#[test]
fn test_large_function_in_impl_block_fails_rule_4() {
    let mut big_body = String::new();
    for i in 0..100 {
        big_body.push_str(&format!("        let _variable_{} = {};\n", i, i));
    }

    let source = format!(
        r#"//! # Large Method Test Module
//!
//! Verifies that methods inside `impl` blocks exceeding 2,000 characters are correctly detected
//! by the AST visitor and rejected per AGENTS.md Rule 4.

pub struct Engine;

impl Engine {{
    pub fn oversized_method(&self) {{
{}
    }}
}}
"#,
        big_body
    );

    let res = linter::check_source(Path::new("src/engine.rs"), &source);
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(
        err.contains("Rule 4") && err.contains("oversized_method"),
        "Expected Rule 4 violation on oversized_method, got: {}",
        err
    );
}

#[test]
fn test_escape_hatch_allows_large_functions() {
    // Verifies that functions annotated with #[allow(clippy::too_many_lines)]
    // can exceed the 2,000-character limit (e.g. for complex Dioxus rsx! components or state machines).
    let mut big_body = String::new();
    for i in 0..100 {
        big_body.push_str(&format!("        let _variable_{} = {};\n", i, i));
    }

    let source = format!(
        r#"//! # Escape Hatch Test Module
//!
//! Verifies that the escape hatch attribute allows complex functions to exceed limits when explicitly intended.

pub struct MacroUi;

impl MacroUi {{
    #[allow(clippy::too_many_lines)]
    pub fn complex_ui_component(&self) {{
{}
    }}
}}
"#,
        big_body
    );

    let res = linter::check_source(Path::new("src/ui.rs"), &source);
    assert!(
        res.is_ok(),
        "Expected function with escape hatch to pass: {:?}",
        res
    );
}

#[test]
fn test_braces_in_strings_and_comments_pass() {
    let source = r#"//! # Braces In Strings And Comments Test Module
//!
//! Verifies that string literals containing braces (such as JSON mockups) and comments with braces
//! do not distort the AST parser or cause false-positive function size errors.

pub fn generate_json() -> &'static str {
    // Note: { this opening brace in a comment } should not confuse the parser!
    let _open_bracket = "{";
    let _close_bracket = "}";
    let _nested = "{ \"key\": [1, 2, { \"sub\": true }] }";
    "done"
}
"#;
    let res = linter::check_source(Path::new("src/json_test.rs"), source);
    assert!(
        res.is_ok(),
        "Expected braces in strings/comments to pass: {:?}",
        res
    );
}

#[test]
fn test_raw_string_with_comment_markers_does_not_corrupt_doc_count() {
    let source = r##"//! # Raw String Test Module
//!
//! Verifies that raw string literals containing comment delimiters like /* and //
//! do not corrupt the comment or production code counters.

pub fn get_sql() -> &'static str {
    let _query = r#"
    /* SQL block comment inside string literal */
    SELECT * FROM users WHERE active = true; // line comment inside string literal
    "#;
    _query
}
"##;
    let res = linter::check_source(Path::new("src/sql_query.rs"), source);
    assert!(
        res.is_ok(),
        "Expected raw strings with comment delimiters to pass: {:?}",
        res
    );
}

#[test]
fn test_attributed_and_multiline_functions_pass() {
    let source = r#"//! # Attributed Function Test Module
//!
//! Verifies that functions preceded by outer attributes (e.g. `#[inline]`, `#[allow(...)]`)
//! and functions with multi-line signatures and where clauses are parsed accurately.

pub struct Worker;

impl Worker {
    #[inline]
    #[allow(dead_code)]
    pub async fn process_job<T>(
        &self,
        job_payload: T,
    ) -> Result<T, ()>
    where
        T: Send + Sync + 'static,
    {
        Ok(job_payload)
    }
}
"#;
    let res = linter::check_source(Path::new("src/worker.rs"), source);
    assert!(
        res.is_ok(),
        "Expected attributed/multiline function to pass: {:?}",
        res
    );
}
