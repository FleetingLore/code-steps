//! Proc macros for the `code-steps` crate.
//!
//! # The four macros
//!
//! | Macro     | Shows code  | Executes code | Pauses  | Has tags |
//! |-----------|------------|---------------|---------|----------|
//! | `step!`   | yes        | yes           | no      | yes      |
//! | `wait!`   | yes        | —             | yes     | yes      |
//! | `skip!`   | yes        | conditional   | no      | yes      |
//! | `ignore!` | no (hidden)| yes           | no      | yes      |
//!
//! All four share the same tag system, controlled at runtime by
//! [`code_steps::display::init_wait_filter`].
//!
//! # The `step!` macro — dual-phase design
//!
//! `step!` does two independent things:
//!
//! **Phase 1 — compile-time source processing** (produces a display string):
//!
//! ```text
//! raw source  →  dedent  →  strip_comments  →  strip_ignores  →  display
//! ```
//!
//! This runs inside the proc macro at compile time.  The display string is
//! baked into the binary and printed verbatim by [`print_code`].
//!
//! **Phase 2 — runtime execution** (produces executable Rust):
//!
//! ```rust
//! {
//!     print_step_header(comment);
//!     print_code(display_str);     // from Phase 1
//!     let __result = { user code }; // the parsed AST, unmodified
//!     print_step_done();
//!     __result
//! }
//! ```
//!
//! The user's code block is compiled normally.  `wait!` / `skip!` / `ignore!`
//! inside it are independently-expanded proc macros — `step!` doesn't touch
//! them at the AST level.
//!
//! [`code_steps::display::init_wait_filter`]: https://docs.rs/code-steps/latest/code_steps/display/fn.init_wait_filter.html
//! [`print_code`]: https://docs.rs/code-steps/latest/code_steps/display/fn.print_code.html

use proc_macro::TokenStream;
use quote::quote;
use syn::{Block, LitStr, Token, parse_macro_input, punctuated::Punctuated};

// ── step! ─────────────────────────────────────────────────────────────────

/// Parsed form of `step!("description"[, "tag"…], { … })`.
struct StepInput {
    comment: LitStr,
    tags: Vec<LitStr>,
    block: Block,
}

impl syn::parse::Parse for StepInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let comment: LitStr = input.parse()?;
        let mut tags = Vec::new();

        // Tags are optional.  If there's a comma after the comment and the
        // next token isn't `{`, parse comma-separated string-literal tags.
        if input.peek(Token![,]) {
            let _comma: Token![,] = input.parse()?;
            while !input.peek(syn::token::Brace) {
                tags.push(input.parse()?);
                if input.peek(Token![,]) {
                    let _comma: Token![,] = input.parse()?;
                } else {
                    break;
                }
            }
        }

        let block: Block = input.parse()?;
        Ok(StepInput {
            comment,
            tags,
            block,
        })
    }
}

/// Remove the common leading whitespace from every line, keeping relative
/// indentation intact.  An empty/whitespace-only line is not counted when
/// computing the minimum.
fn dedent(s: &str) -> String {
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

/// Strip `// …` and `/* … */` comments.  Keeps newlines so line counts
/// stay roughly aligned.  (Does NOT handle string-literals containing `//`;
/// that's fine for typical step blocks.)
fn strip_comments(src: &str) -> String {
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

/// Find `ignore!(("…", …) { … })` blocks and replace them with a single
/// `// (ignored)` placeholder.  The block's code still exists in the AST and
/// will execute at runtime — this only affects the *display* string.
///
/// Brace-matching accounts for nesting (e.g. closures inside the ignore block).
fn strip_ignores(src: &str) -> String {
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

/// The `step!` proc macro.
///
/// See [the module-level documentation](self) for the dual-phase design.
#[proc_macro]
pub fn step(input: TokenStream) -> TokenStream {
    // ── Phase 1: produce the display string ──

    // Grab the raw source text between the outer braces.
    let raw = input.to_string();
    let block_start = raw.find('{').unwrap();
    let block_end = raw.rfind('}').unwrap();
    let inner = &raw[block_start + 1..block_end];
    let code_str = inner.trim_start_matches('\n').to_string();

    // Pipeline: normalise whitespace → remove comments → hide ignore blocks.
    let s = dedent(&code_str);
    let s = strip_comments(&s);
    let display_str = strip_ignores(&s);

    // ── Phase 2: produce the runtime expansion ──

    let parsed = parse_macro_input!(input as StepInput);
    let comment_str = parsed.comment.value();
    let block = &parsed.block;
    let tag_refs: Vec<_> = parsed.tags.iter().collect();

    // If the step has tags, wrap the body in a runtime filter check so the
    // entire step can be skipped via `--include` / `--exclude`.
    let expanded = if tag_refs.is_empty() {
        quote! {{
            ::code_steps::display::print_step_header(#comment_str);
            ::code_steps::display::print_code(#display_str);
            let __result = #block;
            ::code_steps::display::print_step_done();
            __result
        }}
    } else {
        quote! {{
            if ::code_steps::display::filter_matches(&[#(#tag_refs),*]) {
                ::code_steps::display::print_step_header(#comment_str);
                ::code_steps::display::print_code(#display_str);
                let __result = #block;
                ::code_steps::display::print_step_done();
                __result
            }
        }}
    };

    expanded.into()
}

// ── wait!("tag1", "tag2") ────────────────────────────────────────────────
//
// Expands to `press_any_key_if(&["tag1", "tag2"])` (or `&[]` for the
// unconditional `wait!()` form).  The runtime function checks the global
// filter and either prints `"    ..."` + waits, or returns immediately.

#[proc_macro]
pub fn wait(input: TokenStream) -> TokenStream {
    if input.is_empty() {
        return quote! { ::code_steps::display::press_any_key_if(&[]) }.into();
    }

    let tags = parse_macro_input!(input with Punctuated::<LitStr, Token![,]>::parse_terminated);
    let tag_refs: Vec<_> = tags.iter().collect();

    quote! { ::code_steps::display::press_any_key_if(&[#(#tag_refs),*]) }.into()
}

// ── skip!(("tag1", "tag2") { … }) ────────────────────────────────────────
//
// Parses `("tag") { body }` (inside the macro's own parens).  Expands to:
//
// ```ignore
// if !filter_matches(&["tag"]) {
//     { body }
// }
// ```
//
// So the body only runs when the tag is NOT active.  The display string
// still contains `skip!(("tag") { … })` because `step!`'s Phase 1 works on
// the raw source — it doesn't see the expanded `if`.

/// Shared input shape for `skip!((…) { … })` and `ignore!((…) { … })`.
struct SkipInput {
    tags: Punctuated<LitStr, Token![,]>,
    block: Block,
}

impl syn::parse::Parse for SkipInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        syn::parenthesized!(content in input);
        let tags = content.parse_terminated(|input| input.parse(), Token![,])?;
        Ok(SkipInput {
            tags,
            block: input.parse()?,
        })
    }
}

#[proc_macro]
pub fn skip(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as SkipInput);
    let tag_refs: Vec<_> = parsed.tags.iter().collect();
    let block = &parsed.block;

    quote! {
        if !::code_steps::display::filter_matches(&[#(#tag_refs),*]) {
            #block
        }
    }
    .into()
}

// ── ignore!(("tag1", "tag2") { … }) ──────────────────────────────────────
//
// Always executes the block — ignore operates at the *display* level.
// `step!`'s Phase 1 (`strip_ignores`) removes the source text before it
// reaches the terminal.  The tags on `ignore!` are currently NOT checked
// at runtime (the block always runs).

#[proc_macro]
pub fn ignore(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as SkipInput);
    let block = &parsed.block;
    quote! { { #block } }.into()
}
