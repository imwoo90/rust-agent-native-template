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

#[test]
fn test_comment_containing_raw_string_syntax_does_not_lock_parser() {
    let source = r##"//! # Raw String In Comment Test Module
//!
//! Verifies that comments mentioning raw string syntax like `r#"..."#` do not falsely lock the parser.

// Tip: You can define raw strings using r#"syntax"# in Rust.
// This second comment should still be recognized as a comment, not code!
pub fn hello() -> &'static str {
    "hello"
}
"##;
    let res = linter::check_source(Path::new("src/raw_comment.rs"), source);
    assert!(
        res.is_ok(),
        "Expected comment with raw string syntax to pass: {:?}",
        res
    );
}

#[test]
fn test_unicode_korean_comments_do_not_suffer_byte_penalty() {
    // 60 lines of Korean comments, ~30 chars each = ~1,800 Korean chars.
    // In UTF-8 bytes, this is ~5,400 bytes.
    // If the linter used bytes (Rule 3 limit = 4,000 bytes), this would fail.
    // Under true Unicode char counting, 1,800 chars <= 4,000 chars, so it passes!
    let mut korean_doc = String::new();
    for i in 0..60 {
        korean_doc.push_str(&format!(
            "// 한국어 주석 테스트 줄 {:02}: 컴파일러 토큰 예산이 유니코드 글자 수로 계산되는지 검증합니다.\n",
            i
        ));
    }

    let source = format!(
        r#"//! # 한국어 유니코드 주석 테스트 모듈
//!
//! 본 모듈은 CJK 다국어 주석이 UTF-8 바이트(Byte) 단위가 아닌 유니코드 글자(Char) 단위로 측정됨을 증명하는 공식 검증 모듈입니다.

{}
pub fn run_korean_task() -> bool {{
    true
}}
"#,
        korean_doc
    );

    let res = linter::check_source(Path::new("src/korean_doc.rs"), &source);
    assert!(
        res.is_ok(),
        "Expected Unicode Korean comments to pass without byte penalty: {:?}",
        res
    );
}

#[test]
fn test_impl_level_escape_hatch_inherits_to_inner_methods() {
    let mut big_body = String::new();
    for i in 0..100 {
        big_body.push_str(&format!("        let _variable_{} = {};\n", i, i));
    }

    let source = format!(
        r#"//! # Impl Escape Hatch Test Module
//!
//! Verifies that `#[allow(clippy::too_many_lines)]` placed on an `impl` block inherits to all methods.

pub struct MacroUi;

#[allow(clippy::too_many_lines)]
impl MacroUi {{
    pub fn method_one(&self) {{
{}
    }}

    pub fn method_two(&self) {{
{}
    }}
}}
"#,
        big_body, big_body
    );

    let res = linter::check_source(Path::new("src/impl_ui.rs"), &source);
    assert!(
        res.is_ok(),
        "Expected impl-level escape hatch to inherit to inner methods: {:?}",
        res
    );
}

#[test]
fn test_boundary_exact_100_chars_header_passes() {
    // Exactly 100 characters of module doc text across the `//!` lines
    // "1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890" = 100 chars
    let exact_100 = "1234567890".repeat(10);
    assert_eq!(exact_100.chars().count(), 100);

    let source = format!(
        r#"//! {}

pub fn boundary_ok() {{}}
"#,
        exact_100
    );

    let res = linter::check_source(Path::new("src/boundary_100.rs"), &source);
    assert!(
        res.is_ok(),
        "Expected exactly 100-character header to pass Rule 1: {:?}",
        res
    );
}

#[test]
fn test_boundary_99_chars_header_fails() {
    // Exactly 99 characters of module doc text
    let exact_99 = format!("{}123456789", "1234567890".repeat(9));
    assert_eq!(exact_99.chars().count(), 99);

    let source = format!(
        r#"//! {}

pub fn boundary_fail() {{}}
"#,
        exact_99
    );

    let res = linter::check_source(Path::new("src/boundary_99.rs"), &source);
    assert!(
        res.is_err(),
        "Expected 99-character header to fail Rule 1 constraint"
    );
    let err = res.unwrap_err();
    assert!(err.contains("Rule 1"), "Error was: {}", err);
}

#[test]
fn test_compound_cfg_with_not_feature_and_test_scope() {
    // Verifies that `#[cfg(all(not(feature = "mock"), test))]` correctly identifies the test scope
    // and does NOT count towards production code limit.
    let mut big_test = String::new();
    for i in 0..10 {
        big_test.push_str(&format!("    pub fn test_fn_{}() {{\n", i));
        for j in 0..30 {
            big_test.push_str(&format!("        let _x_{}_{} = {};\n", i, j, j));
        }
        big_test.push_str("    }\n");
    }

    let source = format!(
        r#"//! # Compound CFG Test Module
//!
//! Verifies that `#[cfg(all(not(feature = "mock"), test))]` is recognized as a test scope.

#[cfg(all(not(feature = "mock"), test))]
mod tests {{
{}
}}

pub fn prod_fn() {{}}
"#,
        big_test
    );

    let res = linter::check_source(Path::new("src/compound_cfg.rs"), &source);
    assert!(
        res.is_ok(),
        "Expected compound cfg test scope to pass: {:?}",
        res
    );
}

#[test]
fn test_compound_cfg_with_feature_test_utils_is_not_test_scope() {
    // Verifies that `#[cfg(all(feature = "test-utils"))]` is NOT falsely classified as a test scope
    // because "test-utils" is a feature flag name, not the `test` predicate.
    let mut big_body = String::new();
    for i in 0..20 {
        big_body.push_str(&format!("    pub fn helper_{}() {{\n", i));
        for j in 0..30 {
            big_body.push_str(&format!("        let _v_{}_{} = {};\n", i, j, j));
        }
        big_body.push_str("    }\n");
    }

    let source = format!(
        r#"//! # Feature Test Utils Module
//!
//! Verifies that `#[cfg(all(feature = "test-utils"))]` is recognized as production code.

#[cfg(all(feature = "test-utils"))]
mod helpers {{
{}
}}
"#,
        big_body
    );

    let res = linter::check_source(Path::new("src/feature_suite.rs"), &source);
    assert!(
        res.is_err(),
        "Expected feature = 'test-utils' to be counted as production code and exceed Rule 2"
    );
    let err = res.unwrap_err();
    assert!(err.contains("Rule 2"), "Error was: {}", err);
}

#[test]
fn test_lifetimes_with_inline_comments_do_not_corrupt_parser() {
    // Verifies that odd numbers of lifetimes ('a) do not leave quotes open and swallow inline comments.
    let source = r#"//! # Lifetime Module Header
//!
//! This module verifies that lifetimes like `'a` and `'b` do not interfere with inline comment detection.

pub fn parse<'a, 'b>(input: &'a str, _fallback: &'b str) -> Option<&'a str> { // Inline comment here
    let _char_lit = 'c'; // Char literal should not lock parser
    Some(input)
}
"#;
    let res = linter::check_source(Path::new("src/lifetime.rs"), source);
    assert!(res.is_ok(), "Expected lifetime source to pass: {:?}", res);
}

#[test]
fn test_large_test_function_exceeding_2000_chars_is_exempt_from_rule_4() {
    // Verifies that test functions inside #[cfg(test)] exceeding 2,000 chars are exempt from Rule 4.
    let mut large_test_body = String::new();
    for i in 0..70 {
        large_test_body.push_str(&format!("        let _expected_table_entry_{} = {};\n", i, i * 10));
    }

    let source = format!(
        r#"//! # Large Test Function Module
//!
//! Verifies that AGENTS.md Rule 4 does not penalize large test functions inside #[cfg(test)].

pub fn small_prod() -> bool {{
    true
}}

#[cfg(test)]
mod tests {{
    #[test]
    fn test_large_dataset() {{
{}
        assert!(true);
    }}
}}
"#,
        large_test_body
    );

    let res = linter::check_source(Path::new("src/test_large_fn.rs"), &source);
    assert!(
        res.is_ok(),
        "Expected large test function inside #[cfg(test)] to pass Rule 4: {:?}",
        res
    );
}

#[test]
fn test_test_comments_do_not_consume_production_doc_budget() {
    // Verifies that large explanatory comments in unit tests do not cause Rule 3 violation (> 4,000 chars).
    let mut big_test_comments = String::new();
    for i in 0..100 {
        big_test_comments.push_str(&format!("    // Test step {}: Detailed comment explaining test behavior and assertions\n", i));
    }

    let source = format!(
        r#"//! # Test Comments Isolation Module
//!
//! Verifies that comments inside test suites do not deplete the file documentation budget.

pub fn prod_func() {{}}

#[cfg(test)]
mod tests {{
{}
    #[test]
    fn sample_test() {{
        assert_eq!(1 + 1, 2);
    }}
}}
"#,
        big_test_comments
    );

    let res = linter::check_source(Path::new("src/test_comments.rs"), &source);
    assert!(
        res.is_ok(),
        "Expected test comments not to violate Rule 3: {:?}",
        res
    );
}

#[test]
fn test_multisegment_test_attribute_tokio_test_exempt() {
    // Verifies that #[tokio::test] functions are recognized as test scopes and exempt from Rule 4.
    let mut large_async_body = String::new();
    for i in 0..70 {
        large_async_body.push_str(&format!("    let _state_{} = {};\n", i, i * 5));
    }

    let source = format!(
        r#"//! # Tokio Test Module
//!
//! Verifies that functions annotated with multi-segment test attributes like `#[tokio::test]` are exempt from Rule 4.

pub fn prod_small() {{}}

#[tokio::test]
async fn test_async_workflow() {{
{}
}}
"#,
        large_async_body
    );

    let res = linter::check_source(Path::new("src/async_test.rs"), &source);
    assert!(
        res.is_ok(),
        "Expected #[tokio::test] function to be exempt from Rule 4: {:?}",
        res
    );
}

