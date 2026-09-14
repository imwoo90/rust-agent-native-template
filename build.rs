use std::fs;
use std::path::Path;
use syn::spanned::Spanned;
use syn::visit::Visit;

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

    let lines: Vec<&str> = content.lines().collect();
    let mut logical_code_chars = 0;
    let mut doc_chars = 0;
    let mut in_block_comment = false;

    for line in &lines {
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

        logical_code_chars += line_len;
    }

    // 1. Check logical code limit (max 10,000 characters, Rule 2)
    if logical_code_chars > 10000 {
        return Err(format!(
            "File {:?} has {} logical code characters, exceeding the 10,000-character limit (AGENTS.md Rule 2).\nAction: Refactor by splitting responsibilities into smaller submodules.",
            path, logical_code_chars
        ));
    }

    // 2. Check documentation character limit (max 4,000 characters, Rule 3)
    if doc_chars > 4000 {
        return Err(format!(
            "File {:?} has {} documentation/comment characters, exceeding the 4,000-character limit (AGENTS.md Rule 3).\nAction: Keep documentation concise to maintain high signal-to-noise ratio for LLM context.",
            path, doc_chars
        ));
    }

    // 3. Enforce File-Level Documentation (//! at least 100 characters) for production code (Rule 1)
    let path_str = path.to_string_lossy();
    let is_test = path_str.contains("test");

    if !is_test {
        let mut module_doc_len = 0;
        for attr in &syn_file.attrs {
            if matches!(attr.style, syn::AttrStyle::Inner(_)) && attr.path().is_ident("doc") {
                if let syn::Meta::NameValue(syn::MetaNameValue {
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
        }

        if module_doc_len < 100 {
            return Err(format!(
                "File {:?} has a module/file level documentation comment (//!) of only {} characters (minimum 100 characters required, AGENTS.md Rule 1).\nAction: Every production source file must serve as a living Wiki entry detailing its purpose, responsibilities, and architecture.",
                path, module_doc_len
            ));
        }
    }

    // 4. Check function character size (max 2,000 physical characters, Rule 4)
    let mut visitor = FunctionChecker {
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

struct FunctionChecker<'a> {
    path: &'a Path,
    source_lines: &'a [&'a str],
    errors: Vec<String>,
}

impl<'a> FunctionChecker<'a> {
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
        if fn_chars > 2000 {
            self.errors.push(format!(
                "Function at {:?}:{} exceeds the 2,000-character limit ({} characters, AGENTS.md Rule 4).\nFunction: {}\nAction: Refactor into smaller, single-responsibility helper functions.",
                self.path, start.line, fn_chars, fn_name
            ));
        }
    }
}

impl<'ast, 'a> Visit<'ast> for FunctionChecker<'a> {
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
        if let Some(block) = &node.default {
            let fn_name = node.sig.ident.to_string();
            self.check_fn_size(&fn_name, node.sig.span(), block.span());
        }
        syn::visit::visit_trait_item_fn(self, node);
    }
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
