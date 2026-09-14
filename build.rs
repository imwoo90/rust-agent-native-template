#![allow(missing_docs)]

use std::fs;
use std::ops::RangeInclusive;
use std::path::Path;
use syn::spanned::Spanned;
use syn::visit::Visit;

// Agent-Native Architectural Limits (AGENTS.md Rules 1-4)
const MIN_MODULE_DOC_CHARS: usize = 100;
const MAX_LOGICAL_CODE_CHARS: usize = 10_000;
const MAX_DOC_CHARS: usize = 4_000;
const MAX_FUNCTION_CHARS: usize = 2_000;

#[allow(dead_code)]
fn main() {
    // Tell Cargo to rerun this build script if src/ changes
    println!("cargo:rerun-if-changed=src");

    let src_dir = Path::new("src");
    if let Err(e) = check_dir(src_dir) {
        // Output compilation error and fail the build
        eprintln!("\n=== [Agent-Native Linter] Build Constraint Violation ===");
        eprintln!("{}\n", e);
        std::process::exit(1);
    }
}

#[allow(dead_code)]
fn check_dir(dir: &Path) -> Result<(), String> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            check_dir(&path)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            check_file(&path)?;
        }
    }
    Ok(())
}

#[allow(dead_code)]
fn check_file(path: &Path) -> Result<(), String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    check_source(path, &content)
}

#[allow(dead_code)]
pub fn check_source(path: &Path, content: &str) -> Result<(), String> {
    // 0. Parse file AST using syn.
    // If syntax parsing fails due to a code error or typo, gracefully pass through to rustc
    // so the compiler outputs rich, span-accurate diagnostic messages instead of masking them.
    let syn_file = match syn::parse_file(content) {
        Ok(file) => file,
        Err(_) => return Ok(()),
    };

    // Check if the file is an integration test or benchmark file
    let is_test_file = is_path_test_file(path);

    // Check if the entire file has an escape hatch attribute (e.g. macro-heavy UI / generated file)
    let file_has_skip = has_skip_attribute(&syn_file.attrs);

    // Pass 1: Extract test scopes (e.g. #[cfg(test)]) to preserve TDD incentives
    let mut scope_collector = TestScopeCollector {
        test_line_ranges: Vec::new(),
    };
    scope_collector.visit_file(&syn_file);

    // Pass 2: Calculate character metrics (Rules 2 & 3) with string/comment-aware scanning
    let lines: Vec<&str> = content.lines().collect();
    let (prod_logical_code_chars, doc_chars) =
        scan_code_and_comments(&lines, is_test_file, &scope_collector.test_line_ranges);

    // 1. Check production logical code limit (max 10,000 characters, Rule 2)
    if !file_has_skip && prod_logical_code_chars > MAX_LOGICAL_CODE_CHARS {
        let over = prod_logical_code_chars - MAX_LOGICAL_CODE_CHARS;
        return Err(format!(
            "Rule: AGENTS.md Rule 2 (Production Logical Code Limit)\nLocation: {:?}\nLimit: Maximum {} characters (excluding tests & comments)\nActual: {} characters (+{} over limit)\nAction: Refactor by splitting responsibilities into cohesive submodules, or use #![allow(clippy::too_many_lines)] for macro-heavy suites.",
            path, MAX_LOGICAL_CODE_CHARS, prod_logical_code_chars, over
        ));
    }

    // 2. Check documentation character limit (max 4,000 characters, Rule 3)
    if !file_has_skip && doc_chars > MAX_DOC_CHARS {
        let over = doc_chars - MAX_DOC_CHARS;
        return Err(format!(
            "Rule: AGENTS.md Rule 3 (File Documentation Limit)\nLocation: {:?}\nLimit: Maximum {} characters\nActual: {} characters (+{} over limit)\nAction: Keep documentation concise to maintain high signal-to-noise ratio for LLM context.",
            path, MAX_DOC_CHARS, doc_chars, over
        ));
    }

    // 3. Enforce File-Level Living Wiki Header (//! at least 100 characters) for production code (Rule 1)
    if !is_test_file && !file_has_skip {
        let mut module_doc_len = 0;
        for attr in &syn_file.attrs {
            if matches!(attr.style, syn::AttrStyle::Inner(_))
                && attr.path().is_ident("doc")
                && let syn::Meta::NameValue(syn::MetaNameValue {
                    value:
                        syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(s),
                            ..
                        }),
                    ..
                }) = &attr.meta
            {
                // Measure actual Unicode characters, not UTF-8 bytes (prevents CJK penalty)
                module_doc_len += s.value().trim().chars().count();
            }
        }

        if module_doc_len < MIN_MODULE_DOC_CHARS {
            let under = MIN_MODULE_DOC_CHARS - module_doc_len;
            return Err(format!(
                "Rule: AGENTS.md Rule 1 (File-Level Living Wiki Header)\nLocation: {:?}:1\nLimit: Minimum {} characters in module doc (//!)\nActual: {} characters (-{} under limit)\nAction: Every production source file must serve as a living Wiki entry detailing its purpose, responsibilities, and architecture.",
                path, MIN_MODULE_DOC_CHARS, module_doc_len, under
            ));
        }
    }

    // 4. Pass 3: AST Inspection for Function Physical Size (Rule 4)
    if !file_has_skip {
        let mut visitor = FunctionSizeChecker {
            path,
            source_lines: &lines,
            current_impl_has_skip: false,
            errors: Vec::new(),
        };
        visitor.visit_file(&syn_file);

        if let Some(err) = visitor.errors.into_iter().next() {
            return Err(err);
        }
    }

    Ok(())
}

/// Checks whether a path corresponds strictly to a test or benchmark file.
/// Avoids false positives on production files such as `src/contest.rs` or `src/attestation.rs`.
pub fn is_path_test_file(path: &Path) -> bool {
    let in_test_dir = path.components().any(|c| {
        let s = c.as_os_str().to_string_lossy();
        s == "tests" || s == "benches"
    });

    let is_test_filename = path.file_name().is_some_and(|f| {
        let name = f.to_string_lossy();
        name == "test.rs" || name.ends_with("_test.rs") || name.starts_with("test_")
    });

    in_test_dir || is_test_filename
}

/// Checks if an item or file has an escape hatch attribute (e.g. `#[allow(clippy::too_many_lines)]`, `#[agent_lint(skip)]`).
fn has_skip_attribute(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if attr.path().is_ident("allow")
            && let syn::Meta::List(list) = &attr.meta
        {
            let s = list.tokens.to_string();
            if s.contains("too_many_lines")
                || s.contains("clippy :: all")
                || s.contains("clippy::all")
            {
                return true;
            }
        }
        let segs: Vec<String> = attr
            .path()
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect();
        let path_str = segs.join("::");
        path_str == "agent_lint"
            || path_str == "agent_native"
            || path_str == "agent_lint::skip"
            || path_str == "agent_native::skip"
    })
}

/// Collects line ranges for items annotated with `#[cfg(test)]` or `#[test]`.
struct TestScopeCollector {
    test_line_ranges: Vec<RangeInclusive<usize>>,
}

impl<'ast> Visit<'ast> for TestScopeCollector {
    fn visit_item(&mut self, node: &'ast syn::Item) {
        let is_test = match node {
            syn::Item::Mod(m) => is_cfg_test(&m.attrs),
            syn::Item::Fn(f) => is_cfg_test(&f.attrs),
            syn::Item::Struct(s) => is_cfg_test(&s.attrs),
            syn::Item::Enum(e) => is_cfg_test(&e.attrs),
            syn::Item::Const(c) => is_cfg_test(&c.attrs),
            syn::Item::Static(s) => is_cfg_test(&s.attrs),
            syn::Item::Trait(t) => is_cfg_test(&t.attrs),
            syn::Item::Impl(i) => is_cfg_test(&i.attrs),
            _ => false,
        };

        if is_test {
            let span = node.span();
            let start_line = span.start().line.max(1);
            let end_line = span.end().line;
            if start_line <= end_line {
                self.test_line_ranges.push(start_line..=end_line);
            }
        }

        syn::visit::visit_item(self, node);
    }
}

/// Checks if attributes indicate a test configuration, avoiding token spacing artifacts.
fn is_cfg_test(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if attr.path().is_ident("test") {
            return true;
        }
        if attr.path().is_ident("cfg")
            && let syn::Meta::List(list) = &attr.meta
        {
            let compact = list.tokens.to_string().replace(" ", "");
            if compact == "test"
                || compact.starts_with("test,")
                || compact.ends_with(",test")
                || compact.contains(",test,")
            {
                return true;
            }
            if (compact.starts_with("all(") || compact.starts_with("any("))
                && !compact.contains("not(")
            {
                return compact
                    .split(|c: char| !c.is_alphanumeric() && c != '_')
                    .any(|w| w == "test");
            }
        }
        false
    })
}

/// Inspects functions across modules, impl blocks, and traits for physical character limits.
struct FunctionSizeChecker<'a> {
    path: &'a Path,
    source_lines: &'a [&'a str],
    current_impl_has_skip: bool,
    errors: Vec<String>,
}

impl<'a> FunctionSizeChecker<'a> {
    fn check_fn_size(
        &mut self,
        fn_name: &str,
        attrs: &[syn::Attribute],
        sig_span: proc_macro2::Span,
        body_span: proc_macro2::Span,
    ) {
        if self.current_impl_has_skip || has_skip_attribute(attrs) {
            return;
        }

        let start = sig_span.start();
        let end = body_span.end();

        if start.line == 0 || end.line == 0 || start.line > self.source_lines.len() {
            return;
        }

        let fn_chars = calculate_span_chars(self.source_lines, start, end);
        if fn_chars > MAX_FUNCTION_CHARS {
            let over = fn_chars - MAX_FUNCTION_CHARS;
            self.errors.push(format!(
                "Rule: AGENTS.md Rule 4 (Function Physical Size Limit)\nLocation: {:?}:{}\nItem: Function `{}`\nLimit: Maximum {} characters\nActual: {} characters (+{} over limit)\nAction: Refactor into smaller helper functions, or annotate with #[allow(clippy::too_many_lines)] for complex UI macros/state machines.",
                self.path, start.line, fn_name, MAX_FUNCTION_CHARS, fn_chars, over
            ));
        }
    }
}

impl<'ast, 'a> Visit<'ast> for FunctionSizeChecker<'a> {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let fn_name = node.sig.ident.to_string();
        self.check_fn_size(&fn_name, &node.attrs, node.sig.span(), node.block.span());
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        let prev = self.current_impl_has_skip;
        if has_skip_attribute(&node.attrs) {
            self.current_impl_has_skip = true;
        }
        syn::visit::visit_item_impl(self, node);
        self.current_impl_has_skip = prev;
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let fn_name = node.sig.ident.to_string();
        self.check_fn_size(&fn_name, &node.attrs, node.sig.span(), node.block.span());
        syn::visit::visit_impl_item_fn(self, node);
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        let fn_name = node.sig.ident.to_string();
        if let Some(block) = &node.default {
            self.check_fn_size(&fn_name, &node.attrs, node.sig.span(), block.span());
        }
        syn::visit::visit_trait_item_fn(self, node);
    }
}

/// Scans source lines and calculates code vs doc character counts.
/// Accurately counts Unicode characters (preventing multi-byte CJK penalty)
/// and strictly prioritizes comments before raw strings to prevent parser locks.
fn scan_code_and_comments(
    lines: &[&str],
    is_test_file: bool,
    test_ranges: &[RangeInclusive<usize>],
) -> (usize, usize) {
    let mut prod_logical_code_chars = 0;
    let mut doc_chars = 0;
    let mut in_block_comment = false;
    let mut in_raw_string = false;

    for (line_idx, line) in lines.iter().enumerate() {
        let line_num = line_idx + 1; // 1-indexed
        let trimmed = line.trim();
        // Use true Unicode character count (+1 for newline), not UTF-8 bytes
        let line_char_count = line.chars().count() + 1;

        if trimmed.is_empty() {
            continue;
        }

        let is_in_test_scope = is_test_file || test_ranges.iter().any(|r| r.contains(&line_num));

        if in_block_comment {
            doc_chars += line_char_count;
            if let Some(pos) = trimmed.find("*/") {
                in_block_comment = false;
                let remainder = trimmed[pos + 2..].trim();
                if !remainder.is_empty() && !is_in_test_scope {
                    prod_logical_code_chars += remainder.chars().count();
                }
            }
            continue;
        }

        // Check raw string transitions: r#" or r##"
        if in_raw_string {
            if trimmed.contains("\"#") || trimmed.contains("\"##") {
                in_raw_string = false;
            }
            if !is_in_test_scope {
                prod_logical_code_chars += line_char_count;
            }
            continue;
        }

        // [P1 Fix]: Line comments MUST be evaluated BEFORE raw string markers
        // to prevent comments like `// pattern r#"..."#` from falsely locking into raw string mode!
        if trimmed.starts_with("//") {
            doc_chars += line_char_count;
            continue;
        }

        // Check standard block comment start
        if trimmed.starts_with("/*") {
            doc_chars += line_char_count;
            if !trimmed.contains("*/") {
                in_block_comment = true;
            }
            continue;
        }

        // Check raw string start in actual code
        if trimmed.contains("r#\"") || trimmed.contains("r##\"") {
            if !(trimmed.ends_with("\"#")
                || trimmed.ends_with("\"##")
                || (trimmed.contains("\"#") && !trimmed.starts_with("r#\"")))
            {
                in_raw_string = true;
            }
            if !is_in_test_scope {
                prod_logical_code_chars += line_char_count;
            }
            continue;
        }

        // Normal line: if line contains inline comment `//` outside of quotes, split counts
        if let Some(slash_pos) = find_inline_comment(trimmed) {
            let code_part = trimmed[..slash_pos].trim();
            let comment_part = &trimmed[slash_pos..];
            if !is_in_test_scope {
                prod_logical_code_chars += code_part.chars().count() + 1;
            }
            doc_chars += comment_part.chars().count();
        } else if !is_in_test_scope {
            prod_logical_code_chars += line_char_count;
        }
    }

    (prod_logical_code_chars, doc_chars)
}

/// Finds the start position of an inline `//` comment that is not inside quotes.
fn find_inline_comment(line: &str) -> Option<usize> {
    let mut in_quote = false;
    let mut quote_char = ' ';
    let mut chars = line.char_indices().peekable();

    while let Some((idx, ch)) = chars.next() {
        if in_quote {
            if ch == '\\' {
                chars.next(); // Skip escaped character
            } else if ch == quote_char {
                in_quote = false;
            }
        } else if ch == '"' || ch == '\'' {
            in_quote = true;
            quote_char = ch;
        } else if ch == '/'
            && let Some(&(_, next_ch)) = chars.peek()
            && next_ch == '/'
        {
            return Some(idx);
        }
    }
    None
}

fn calculate_span_chars(
    lines: &[&str],
    start: proc_macro2::LineColumn,
    end: proc_macro2::LineColumn,
) -> usize {
    let start_line = start.line.max(1);
    let end_line = end.line.min(lines.len());

    if start_line > end_line {
        return 0;
    }

    let mut total_chars = 0;
    for (line_idx, line) in lines.iter().enumerate().take(end_line).skip(start_line - 1) {
        let is_first = line_idx == start_line - 1;
        let is_last = line_idx == end_line - 1;

        let start_col = if is_first { start.column } else { 0 };
        let end_col = if is_last { Some(end.column) } else { None };

        total_chars += char_range(line, start_col, end_col);
        if !is_last {
            total_chars += 1; // newline character
        }
    }

    total_chars
}

fn char_range(line: &str, start_byte: usize, end_byte: Option<usize>) -> usize {
    let start_idx = line
        .char_indices()
        .map(|(idx, _)| idx)
        .find(|&idx| idx >= start_byte)
        .unwrap_or(line.len());
    let end_idx = match end_byte {
        Some(eb) => line
            .char_indices()
            .map(|(idx, _)| idx)
            .find(|&idx| idx >= eb)
            .unwrap_or(line.len()),
        None => line.len(),
    };
    if start_idx <= end_idx {
        line[start_idx..end_idx].chars().count()
    } else {
        0
    }
}
