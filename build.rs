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
    // 0. Parse file AST using syn
    let syn_file = syn::parse_file(content)
        .map_err(|e| format!("Failed to parse Rust syntax in {:?}: {}", path, e))?;

    let path_str = path.to_string_lossy();
    let is_test_file = path_str.contains("test");

    // Pass 1: Extract test scopes (e.g. #[cfg(test)]) to preserve TDD incentives
    let mut scope_collector = TestScopeCollector {
        test_line_ranges: Vec::new(),
    };
    scope_collector.visit_file(&syn_file);

    // Pass 2: Calculate character metrics (Rules 2 & 3)
    let lines: Vec<&str> = content.lines().collect();
    let mut prod_logical_code_chars = 0;
    let mut doc_chars = 0;
    let mut in_block_comment = false;

    for (line_idx, line) in lines.iter().enumerate() {
        let line_num = line_idx + 1; // 1-indexed
        let trimmed = line.trim();
        let line_len = line.len() + 1; // Count raw line length + newline character

        if trimmed.is_empty() {
            continue;
        }

        if in_block_comment {
            doc_chars += line_len;
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }

        if trimmed.starts_with("/*") {
            doc_chars += line_len;
            if !trimmed.contains("*/") {
                in_block_comment = true;
            }
            continue;
        }

        if trimmed.starts_with("//") {
            doc_chars += line_len;
            continue;
        }

        // Check if this line is part of a #[cfg(test)] or test file
        let is_in_test_scope = is_test_file
            || scope_collector
                .test_line_ranges
                .iter()
                .any(|r| r.contains(&line_num));

        if !is_in_test_scope {
            prod_logical_code_chars += line_len;
        }
    }

    // 1. Check production logical code limit (max 10,000 characters, Rule 2)
    if prod_logical_code_chars > MAX_LOGICAL_CODE_CHARS {
        let over = prod_logical_code_chars - MAX_LOGICAL_CODE_CHARS;
        return Err(format!(
            "Rule: AGENTS.md Rule 2 (Production Logical Code Limit)\nLocation: {:?}\nLimit: Maximum {} characters (excluding tests & comments)\nActual: {} characters (+{} over limit)\nAction: Refactor by splitting responsibilities into cohesive submodules.",
            path, MAX_LOGICAL_CODE_CHARS, prod_logical_code_chars, over
        ));
    }

    // 2. Check documentation character limit (max 4,000 characters, Rule 3)
    if doc_chars > MAX_DOC_CHARS {
        let over = doc_chars - MAX_DOC_CHARS;
        return Err(format!(
            "Rule: AGENTS.md Rule 3 (File Documentation Limit)\nLocation: {:?}\nLimit: Maximum {} characters\nActual: {} characters (+{} over limit)\nAction: Keep documentation concise to maintain high signal-to-noise ratio for LLM context.",
            path, MAX_DOC_CHARS, doc_chars, over
        ));
    }

    // 3. Enforce File-Level Living Wiki Header (//! at least 100 characters) for production code (Rule 1)
    if !is_test_file {
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
                module_doc_len += s.value().trim().len();
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
    let mut visitor = FunctionSizeChecker {
        path,
        source_lines: &lines,
        errors: Vec::new(),
    };
    visitor.visit_file(&syn_file);

    if let Some(err) = visitor.errors.into_iter().next() {
        return Err(err);
    }

    Ok(())
}

/// Collects line ranges for items annotated with `#[cfg(test)]`.
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

/// Inspects functions across modules, impl blocks, and traits for physical character limits.
struct FunctionSizeChecker<'a> {
    path: &'a Path,
    source_lines: &'a [&'a str],
    errors: Vec<String>,
}

impl<'a> FunctionSizeChecker<'a> {
    fn check_fn_size(
        &mut self,
        fn_name: &str,
        sig_span: proc_macro2::Span,
        body_span: proc_macro2::Span,
    ) {
        let start = sig_span.start();
        let end = body_span.end();

        if start.line == 0 || end.line == 0 || start.line > self.source_lines.len() {
            return;
        }

        let fn_chars = calculate_span_chars(self.source_lines, start, end);
        if fn_chars > MAX_FUNCTION_CHARS {
            let over = fn_chars - MAX_FUNCTION_CHARS;
            self.errors.push(format!(
                "Rule: AGENTS.md Rule 4 (Function Physical Size Limit)\nLocation: {:?}:{}\nItem: Function `{}`\nLimit: Maximum {} characters\nActual: {} characters (+{} over limit)\nAction: Refactor into smaller, single-responsibility helper functions.",
                self.path, start.line, fn_name, MAX_FUNCTION_CHARS, fn_chars, over
            ));
        }
    }
}

impl<'ast, 'a> Visit<'ast> for FunctionSizeChecker<'a> {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let fn_name = node.sig.ident.to_string();
        self.check_fn_size(&fn_name, node.sig.span(), node.block.span());
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let fn_name = node.sig.ident.to_string();
        self.check_fn_size(&fn_name, node.sig.span(), node.block.span());
        syn::visit::visit_impl_item_fn(self, node);
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        let fn_name = node.sig.ident.to_string();
        if let Some(block) = &node.default {
            self.check_fn_size(&fn_name, node.sig.span(), block.span());
        }
        syn::visit::visit_trait_item_fn(self, node);
    }
}

fn is_cfg_test(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        if attr.path().is_ident("test") {
            return true;
        }
        if attr.path().is_ident("cfg")
            && let syn::Meta::List(list) = &attr.meta
        {
            return list.tokens.to_string().contains("test");
        }
        false
    })
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
