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
//! dedent()            ── strip common leading whitespace
//!   │
//!   ▼
//! strip_comments()    ── remove // and /* */ comments
//!   │
//!   ▼
//! strip_ignores()     ── replace ignore![…] { … } with // (ignored)
//!   │
//!   ▼
//! strip_nested_steps()── replace step![…] { … } with // (nested step)
//!   │
//!   ▼
//! display string       ── printed verbatim by print_code()
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

/// Find `ignore![("…", …) { … }]` blocks in the source and replace each
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
        // Match `ignore!(` — possibly prefixed with `crate::` or `code_steps::`
        if i + 8 < len && chars[i..].starts_with(&['i', 'g', 'n', 'o', 'r', 'e', '!', '(']) {
            // Capture the indentation from the line being replaced.
            // Extract only the leading whitespace (not the `code_steps::` prefix).
            let line_prefix = match out.rfind('\n') {
                Some(pos) => &out[pos + 1..],
                None => &out[..],
            };
            let indent: String = line_prefix
                .chars()
                .take_while(|c| c.is_whitespace())
                .collect();
            // Strip the entire line prefix from `out`.
            out.truncate(out.len().saturating_sub(line_prefix.len()));

            i += 7; // skip "ignore!"
            let mut depth;
            // Skip the tag list in parens: `("tag1", "tag2")` — also capture tags
            let mut tags = Vec::new();
            if i < len && chars[i] == '(' {
                depth = 1;
                i += 1;
                // Read tag strings inside the parens
                while i < len && depth > 0 {
                    match chars[i] {
                        '(' | '{' => depth += 1,
                        ')' | '}' => {
                            depth -= 1;
                            if depth == 0 {
                                i += 1;
                                break;
                            }
                        }
                        '"' => {
                            i += 1;
                            let mut tag = String::new();
                            while i < len && chars[i] != '"' {
                                if chars[i] == '\\' && i + 1 < len {
                                    i += 1;
                                }
                                tag.push(chars[i]);
                                i += 1;
                            }
                            if i < len {
                                i += 1;
                            }
                            tags.push(tag);
                            continue; // skip the `i += 1` at end of loop
                        }
                        _ => {}
                    }
                    if depth > 0 {
                        i += 1;
                    }
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
            // Consume closing `)` and `;` of the ignore!(…) call
            while i < len && chars[i].is_whitespace() {
                i += 1;
            }
            if i < len && chars[i] == ')' {
                i += 1;
            }
            while i < len && chars[i].is_whitespace() {
                i += 1;
            }
            if i < len && chars[i] == ';' {
                i += 1;
            }
            if tags.is_empty() {
                out.push_str(&format!("{indent}// (ignored)"));
            } else {
                let tag_list = tags.join(", ");
                out.push_str(&format!("{indent}// (ignored {tag_list})"));
            }
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

/// After `TokenStream::to_string()` normalises whitespace, consecutive
/// statements end up on one line with stray spaces.  This inserts a
/// newline after each `;` and strips any following spaces, so every
/// line starts at the same indentation level.
pub fn restore_newlines(src: &str) -> String {
    let mut out = String::with_capacity(src.len() + 16);
    let chars: Vec<char> = src.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        out.push(chars[i]);
        if chars[i] == ';' {
            // Skip spaces after `;` up to the next non-space char
            let mut j = i + 1;
            while j < chars.len() && chars[j] == ' ' {
                j += 1;
            }
            if j < chars.len() && chars[j] != '\n' && chars[j] != '}' {
                out.push('\n');
                i = j - 1; // skip the spaces we consumed
            }
        }
        i += 1;
    }
    out
}

/// Collapse multi-line expressions into single lines.  Lines that don't
/// end with `;`, `{`, or `}` are joined with the next line with a space.
pub fn collapse_continuations(src: &str) -> String {
    let mut out = String::new();
    for line in src.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            out.push('\n');
        } else if trimmed.ends_with(';') || trimmed.ends_with('}') || trimmed.ends_with('{') {
            out.push_str(line);
            out.push('\n');
        } else {
            out.push_str(line.trim_end());
            out.push(' ');
        }
    }
    out
}

/// Find `step!["…", …, { … }]` calls in the source and replace each
/// with a `// description` placeholder that shows the step's title.
///
/// When a `step!` contains nested `step!` calls, the outer block's display
/// would otherwise repeat the inner step's source — which is confusing
/// because the inner step handles its own display.  This function prevents
/// that by hiding nested `step!` invocations from the outer step's output.
///
/// The placeholder includes the nested step's description so the reader
/// can see *what* happens next without the full code.
///
/// Depth-tracking handles `(`, `)`, `{`, `}` so nested braces and
/// parenthesised expressions inside the nested step don't break matching.
pub fn strip_nested_steps(src: &str) -> String {
    let chars: Vec<char> = src.chars().collect();
    let len = chars.len();
    let mut out = String::with_capacity(len);
    let mut i = 0;

    while i < len {
        // Match `step!(`
        if i + 6 < len && chars[i..].starts_with(&['s', 't', 'e', 'p', '!', '(']) {
            let start = i;
            i += 6; // skip "step!("

            // ── Extract the step description ──
            let mut description = String::new();
            // Skip whitespace before the opening quote
            while i < len && chars[i].is_whitespace() {
                i += 1;
            }
            if i < len && chars[i] == '"' {
                i += 1; // skip opening `"`
                while i < len {
                    if chars[i] == '\\' && i + 1 < len {
                        i += 1; // skip backslash
                        description.push(chars[i]);
                    } else if chars[i] == '"' {
                        i += 1; // skip closing `"`
                        break;
                    } else {
                        description.push(chars[i]);
                    }
                    i += 1;
                }
            }

            // ── Skip the rest of the step!(…) invocation ──
            let mut depth: i32 = 1; // inside the outer `(`
            let mut in_string = false;
            let mut in_char = false;

            while i < len && depth > 0 {
                match chars[i] {
                    '"' if !in_char => in_string = !in_string,
                    '\'' if !in_string => in_char = !in_char,
                    '\\' if in_string || in_char => {
                        i += 1; // skip escaped char
                    }
                    '(' if !in_string && !in_char => depth += 1,
                    ')' if !in_string && !in_char => {
                        depth -= 1;
                        if depth == 0 {
                            i += 1; // consume the closing `)`
                            break;
                        }
                    }
                    '{' if !in_string && !in_char => depth += 1,
                    '}' if !in_string && !in_char => depth -= 1,
                    _ => {}
                }
                i += 1;
            }
            // Consume trailing semicolon and whitespace
            while i < len && chars[i].is_whitespace() {
                i += 1;
            }
            if i < len && chars[i] == ';' {
                i += 1;
            }
            if i > start {
                if description.is_empty() {
                    out.push_str("// (nested step)\n");
                } else {
                    out.push_str(&format!("// {description}\n"));
                }
            } else {
                out.push_str("step!(");
            }
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}
