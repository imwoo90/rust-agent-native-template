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

/// A public function.
pub fn run() {}
"#;
    let res = linter::check_source(Path::new("src/sample.rs"), source);
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(err.contains("Rule 1"), "Error was: {}", err);
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
fn test_braces_in_strings_and_comments_pass() {
    let source = r#"//! # Braces In Strings And Comments Test Module
//!
//! Verifies that string literals containing braces (such as JSON mockups) and comments with braces
//! do not distort the AST parser or cause false-positive function size errors.

/// Generates a JSON string literal with braces.
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

/// An engine struct.
pub struct Engine;

impl Engine {{
    /// An oversized method that violates Rule 4.
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
fn test_public_struct_missing_doc_fails_rule_5() {
    let source = r#"//! # Missing Doc Test Module
//!
//! This module intentionally verifies that public structs without documentation comments are properly
//! detected and rejected by Rule 5 (Living LLM-Wiki).

pub struct UndocumentedStruct;
"#;
    let res = linter::check_source(Path::new("src/missing_doc.rs"), source);
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(
        err.contains("Rule 5") && err.contains("UndocumentedStruct"),
        "Expected Rule 5 violation, got: {}",
        err
    );
}

#[test]
fn test_public_fn_missing_doc_fails_rule_5() {
    let source = r#"//! # Missing Doc Test Module
//!
//! This module intentionally verifies that public functions without documentation comments are properly
//! detected and rejected by Rule 5 (Living LLM-Wiki).

pub fn undocumented_function() {}
"#;
    let res = linter::check_source(Path::new("src/missing_doc.rs"), source);
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(
        err.contains("Rule 5") && err.contains("undocumented_function"),
        "Expected Rule 5 violation, got: {}",
        err
    );
}

#[test]
fn test_public_method_in_impl_missing_doc_fails_rule_5() {
    let source = r#"//! # Missing Doc Test Module
//!
//! This module intentionally verifies that public methods in inherent impl blocks without documentation
//! comments are properly detected and rejected by Rule 5 (Living LLM-Wiki).

/// A documented struct.
pub struct DocumentedStruct;

impl DocumentedStruct {
    pub fn undocumented_method(&self) {}
}
"#;
    let res = linter::check_source(Path::new("src/missing_doc.rs"), source);
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(
        err.contains("Rule 5") && err.contains("undocumented_method"),
        "Expected Rule 5 violation, got: {}",
        err
    );
}

#[test]
fn test_private_items_do_not_require_doc() {
    let source = r#"//! # Private Items Test Module
//!
//! Verifies that internal/private helper items do not require documentation comments.

struct InternalHelper;

impl InternalHelper {
    fn internal_step(&self) {}
}

fn internal_run() {}
"#;
    let res = linter::check_source(Path::new("src/private.rs"), source);
    assert!(res.is_ok(), "Expected private items to pass: {:?}", res);
}

#[test]
fn test_production_unwrap_fails_rule_6() {
    let source = r#"//! # Prohibited Unwrap Module
//!
//! Verifies that calling `.unwrap()` in production code is rejected by Rule 6.

/// Performs an action with a dangerous unwrap call.
pub fn risky_operation(opt: Option<i32>) -> i32 {
    opt.unwrap()
}
"#;
    let res = linter::check_source(Path::new("src/risky.rs"), source);
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(
        err.contains("Rule 6") && err.contains(".unwrap()"),
        "Expected Rule 6 violation, got: {}",
        err
    );
}

#[test]
fn test_production_expect_fails_rule_6() {
    let source = r#"//! # Prohibited Expect Module
//!
//! Verifies that calling `.expect()` in production code is rejected by Rule 6.

/// Performs an action with a dangerous expect call.
pub fn risky_operation(opt: Option<i32>) -> i32 {
    opt.expect("must not be None")
}
"#;
    let res = linter::check_source(Path::new("src/risky.rs"), source);
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(
        err.contains("Rule 6") && err.contains(".expect()"),
        "Expected Rule 6 violation, got: {}",
        err
    );
}

#[test]
fn test_test_code_allows_unwrap_and_expect() {
    let source = r#"//! # Safe Production With Tests Module
//!
//! Verifies that calling `.unwrap()` and `.expect()` inside `#[cfg(test)]` modules is allowed.

/// Safe production logic.
pub fn safe_calc(val: i32) -> Result<i32, ()> {
    Ok(val * 2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_test_scope() {
        let res = safe_calc(10);
        assert_eq!(res.unwrap(), 20);
        assert_eq!(res.expect("should succeed"), 20);
    }
}
"#;
    let res = linter::check_source(Path::new("src/test_scope.rs"), source);
    assert!(
        res.is_ok(),
        "Expected tests to allow unwrap/expect, got: {:?}",
        res
    );
}

#[test]
fn test_safe_unwrap_alternatives_pass_in_production() {
    let source = r#"//! # Safe Fallbacks Test Module
//!
//! Verifies that idiomatic non-panicking fallbacks like `unwrap_or`, `unwrap_or_default`,
//! and `unwrap_or_else` pass cleanly in production code.

/// Evaluates options safely without panicking.
pub fn safe_fallbacks(opt: Option<i32>) -> i32 {
    let a = opt.unwrap_or(0);
    let b = opt.unwrap_or_default();
    let c = opt.unwrap_or_else(|| 42);
    a + b + c
}
"#;
    let res = linter::check_source(Path::new("src/safe_fallbacks.rs"), source);
    assert!(
        res.is_ok(),
        "Expected safe unwrap fallbacks to pass: {:?}",
        res
    );
}

#[test]
fn test_attributed_and_multiline_functions_pass() {
    let source = r#"//! # Attributed Function Test Module
//!
//! Verifies that functions preceded by outer attributes (e.g. `#[inline]`, `#[allow(...)]`)
//! and functions with multi-line signatures and where clauses are parsed accurately.

/// A worker struct for background tasks.
pub struct Worker;

impl Worker {
    /// Processes a generic job payload asynchronously.
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
