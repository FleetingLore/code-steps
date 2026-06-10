//! Phase 1: compile-time source processing pipeline.
//!
//! This module is used **exclusively** by [`step!`](super::step) during
//! Phase 1 — it transforms the raw source text of a step block into the
//! display string that will be baked into the binary.
//!
//! # Pipeline
//!
//! ```text
//! raw source
//!   │
//!   ▼
//! dedent()           ── strip common leading whitespace
//!   │
//!   ▼
//! strip_comments()   ── remove // and /* */ comments
//!   │
//!   ▼
//! strip_ignores()    ── replace ignore!(…) { … } with // (ignored)
//!   │
//!   ▼
//! display string      ── printed verbatim by print_code()
//! ```
//!
//! All functions operate on `&str` → `String` and are independent of the
//! Rust AST — they work on raw source characters.

/// Remove the common leading whitespace from every line, keeping relative
/// indentation intact.
///
/// An empty/whitespace-only line is **not** counted when computing the
/// minimum indent, so trailing blank lines don't skew the result.
pub fn dedent(s: &str) -> String {
    let lines: Vec<&str> = s.lines().collect();
    let min = lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);
    let mut out = String::with_capacity(s.len());
    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        if line.len() > min {
            out.push_str(&line[min..]);
        }
    }
    out.push('\n');
    out
}

/// Strip `// …` and `/* … */` comments from the source.
///
/// Keeps newlines so line counts stay roughly aligned.  Does **not** handle
/// string-literals containing `//` — that's fine for typical step blocks.
pub fn strip_comments(src: &str) -> String {
    let chars: Vec<char> = src.chars().collect();
    let len = chars.len();
    let mut out = String::with_capacity(len);
    let mut i = 0;

    while i < len {
        if i + 1 < len && chars[i] == '/' && chars[i + 1] == '*' {
            i += 2;
            while i + 1 < len && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            i += 2;
            continue;
        }
        if i + 1 < len && chars[i] == '/' && chars[i + 1] == '/' {
            while i < len && chars[i] != '\n' {
                i += 1;
            }
            if i < len {
                out.push('\n');
                i += 1;
            }
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Find `ignore!(("…", …) { … })` blocks in the source and replace each
/// with a single `// (ignored)` placeholder.
///
/// The block's code still exists in the Rust AST and **will execute at
/// runtime** — this function only affects the *display* string that appears
/// in the terminal.
///
/// Brace-matching accounts for nesting (e.g. closures inside the ignore
/// block), so this works correctly even with deeply nested code.
pub fn strip_ignores(src: &str) -> String {
    let chars: Vec<char> = src.chars().collect();
    let len = chars.len();
    let mut out = String::with_capacity(len);
    let mut i = 0;

    while i < len {
        // Match `ignore!`
        if i + 8 < len && chars[i..].starts_with(&['i', 'g', 'n', 'o', 'r', 'e', '!', '(']) {
            let mut depth;
            i += 7;
            // Skip the tag list in parens: `("tag1", "tag2")`
            if i < len && chars[i] == '(' {
                depth = 1;
                i += 1;
                while i < len && depth > 0 {
                    match chars[i] {
                        '(' => depth += 1,
                        ')' => depth -= 1,
                        _ => {}
                    }
                    i += 1;
                }
            }
            // Skip whitespace before the block
            while i < len && chars[i].is_whitespace() {
                i += 1;
            }
            // Skip the braced block
            if i < len && chars[i] == '{' {
                depth = 1;
                i += 1;
                while i < len && depth > 0 {
                    match chars[i] {
                        '{' => depth += 1,
                        '}' => depth -= 1,
                        _ => {}
                    }
                    i += 1;
                }
            }
            out.push_str("// (ignored)\n");
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}
