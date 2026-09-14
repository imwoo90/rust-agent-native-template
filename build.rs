use std::fs;
use std::path::Path;

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

fn check_file(path: &Path) -> Result<(), String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let lines: Vec<&str> = content.lines().collect();

    let mut logical_code_chars = 0;
    let mut doc_chars = 0;
    let mut in_block_comment = false;
    let mut module_docs = Vec::new();

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
            // Collect module-level docs
            if trimmed.starts_with("//!") {
                let doc_line = if trimmed.starts_with("//! ") {
                    &trimmed[4..]
                } else {
                    &trimmed[3..]
                };
                module_docs.push(doc_line.to_string());
            }
            continue;
        }

        logical_code_chars += line_len;
    }

    // 1. Check logical code limit (max 10,000 characters)
    if logical_code_chars > 10000 {
        return Err(format!(
            "File {:?} has {} logical code characters, exceeding the 10,000-character limit (AGENTS.md Rule 2).\nAction: Refactor by splitting responsibilities into smaller submodules.",
            path, logical_code_chars
        ));
    }

    // 2. Check documentation character limit (max 4,000 characters)
    if doc_chars > 4000 {
        return Err(format!(
            "File {:?} has {} documentation/comment characters, exceeding the 4,000-character limit (AGENTS.md Rule 3).\nAction: Keep documentation concise to maintain high signal-to-noise ratio for LLM context.",
            path, doc_chars
        ));
    }

    // 3. Enforce File-Level Documentation (//! at least 100 characters) for production code
    let path_str = path.to_string_lossy();
    let is_test = path_str.contains("test");

    if !is_test {
        let module_doc_len = module_docs
            .iter()
            .map(|line| line.trim().len())
            .sum::<usize>();
        if module_doc_len < 100 {
            return Err(format!(
                "File {:?} has a module/file level documentation comment (//!) of only {} characters (minimum 100 characters required, AGENTS.md Rule 1).\nAction: Every production source file must serve as a living Wiki entry detailing its purpose, responsibilities, and architecture.",
                path, module_doc_len
            ));
        }
    }

    // 4. Check function character size (max 2,000 physical characters)
    let mut in_fn = false;
    let mut seen_opening_brace = false;
    let mut fn_start_line = 0;
    let mut brace_depth = 0;
    let mut fn_chars = 0;
    let mut fn_name = String::new();

    for (idx, line) in lines.iter().enumerate() {
        let line_trimmed = line.trim();
        let line_len = line.len() + 1;

        // Detect function start
        if !in_fn && brace_depth == 0 && is_fn_declaration(line_trimmed) {
            in_fn = true;
            seen_opening_brace = false;
            fn_start_line = idx + 1;
            fn_chars = 0;
            fn_name = line_trimmed.to_string();
        }

        if in_fn {
            fn_chars += line_len;

            for c in line.chars() {
                if c == '{' {
                    brace_depth += 1;
                    seen_opening_brace = true;
                } else if c == '}' {
                    brace_depth -= 1;
                }
            }

            // Function ends when brace depth returns to 0 after opening brace,
            // or if line ends with semicolon before opening brace (e.g. trait declaration)
            if (seen_opening_brace && brace_depth == 0)
                || (!seen_opening_brace && line_trimmed.ends_with(';'))
            {
                if seen_opening_brace && fn_chars > 2000 {
                    return Err(format!(
                        "Function at {:?}:{} exceeds the 2,000-character limit ({} characters, AGENTS.md Rule 4).\nFunction: {}\nAction: Refactor into smaller, single-responsibility helper functions.",
                        path, fn_start_line, fn_chars, fn_name
                    ));
                }
                in_fn = false;
            }
        }
    }

    Ok(())
}

fn is_fn_declaration(line: &str) -> bool {
    let prefixes = [
        "fn ",
        "pub fn ",
        "pub(crate) fn ",
        "async fn ",
        "pub async fn ",
        "pub(crate) async fn ",
        "const fn ",
        "pub const fn ",
        "pub(crate) const fn ",
        "unsafe fn ",
        "pub unsafe fn ",
        "pub(crate) unsafe fn ",
    ];
    prefixes.iter().any(|prefix| line.starts_with(prefix))
}
