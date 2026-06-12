//! The `step!` proc macro — display + execute a named code block.
//!
//! `step!` is the primary macro: it shows syntax-highlighted code in the
//! terminal, executes it, prints a green `ok`, and **automatically pauses**
//! for Enter.  Everything else — `wait!`, `skip!`, `ignore!` — is used
//! *inside* a `step!` block for finer control.
//!
//! The auto-pause means you rarely need `wait!()` at the end of a step.
//! Use `wait!()` *inside* a step only when you want to pause mid-step
//! (e.g. between two sub-operations), or `wait!("tag")` for a conditional
//! pause controlled by the filter.
//!
//! # Two forms
//!
//! | Form | Behaviour |
//! |------|-----------|
//! | `step!("desc", { … })` | Always displays and executes |
//! | `step!("desc", "tag", { … })` | Displays/executes **only if** the tag passes the global filter |
//!
//! The first form is for steps that should always run.  The second form lets
//! you make a step **optional** — controlled from the command line without
//! changing code.
//!
//! # Tag filtering — how `step!` connects to CLI args
//!
//! Tags bridge your source code to runtime command-line flags.  The workflow:
//!
//! 1. **In code** — attach string tags to `step!`, `wait!`, `skip!`, `ignore!`:
//!
//! ```rust,ignore
//! init_wait_filter();
//!
//! step!("Basic setup", "basic", {
//!     load_data();
//!     wait!("basic");
//! });
//!
//! step!("Advanced analysis", "advanced", {
//!     run_heavy_computation();
//!     wait!("advanced");
//! });
//! ```
//!
//! 2. **On the command line** — pass `--include` / `--exclude` after `--`:
//!
//! ```text
//! cargo run -- --include basic
//! cargo run -- --include advanced --exclude basic
//! ```
//!
//! 3. **At runtime** — the filter checks each step's tags against the CLI args.
//!    A step with a tag that **doesn't match** is completely skipped — no
//!    header, no code display, no execution.
//!
//! ## Filter rules
//!
//! For a step tagged `["basic"]` with `--include basic --exclude debug`:
//!
//! - `include` is non-empty → step runs only if **any** of its tags is in the
//!   include list.  Here `"basic"` is in `["basic"]`, so it passes.
//! - `exclude` blocks the step if **any** tag matches.  Here `"debug"` doesn't
//!   match, so it passes.
//! - Without `--include` (empty), **all tags pass** the include check.
//! - A step with **no tags** (first form) always runs — filter doesn't apply.
//!
//! ## Comparison with `skip!` and `ignore!`
//!
//! All three can respond to tags, but they do different things:
//!
//! | Macro   | Tagged behaviour |
//! |---------|------------------|
//! | `step!` | Entire step skipped: nothing shown, nothing run |
//! | `skip!` | Code **shown** but **not executed** when tag is active |
//! | `ignore!` | Code **executed** but **hidden** when tag is active |
//!
//! `step!` is the coarse-grained switch — turn entire sections on/off.
//! `skip!`/`ignore!` are fine-grained — control visibility vs execution
//! *within* a step.
//!
//! # How it works internally
//!
//! `step!` does two things at two different times:
//!
//! **Phase 1 — compile-time**: the raw source text is run through the
//! [`source`](super::source) pipeline (`dedent` → `restore_newlines` →
//! `collapse_continuations` → `strip_comments` → `strip_ignores` →
//! `strip_nested_steps`) and baked into the binary as a `&str`.  No
//! runtime cost — the terminal just prints a pre-computed string.
//!
//! **Phase 2 — runtime**: the parsed AST drives code generation.  A
//! cyan `=====` separator is printed, then the header, then the
//! display string (with typewriter effect if enabled), then the user's
//! code executes, then an auto-pause shows the nesting path.  Tags wrap
//! the body in `if filter_matches(&[...]) { … }` for conditional
//! execution.
//!
//! # Why two phases?
//!
//! `syn` (the Rust parser used by proc macros) discards whitespace and
//! comments.  But terminal display *needs* the original formatting.  So
//! `step!` processes the same input twice: once as raw text (Phase 1)
//! and once as a typed AST (Phase 2).
//!
//! [`quote`]: https://docs.rs/quote

use proc_macro::TokenStream;
use quote::quote;
use syn::{Block, LitStr, Token, parse_macro_input};

use crate::source;

// ── StepInput parser ───────────────────────────────────────────────────────

/// Parsed form of `step!("description"[, "tag"…], { … })`.
///
/// Tags are optional.  If present, they appear as comma-separated string
/// literals between the description and the block.
pub struct StepInput {
    pub comment: LitStr,
    pub tags: Vec<LitStr>,
    pub block: Block,
}

impl syn::parse::Parse for StepInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let comment: LitStr = input.parse()?;
        let mut tags = Vec::new();

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

// ── Implementation (called from lib.rs entry point) ────────────────────────

pub fn step_impl(input: TokenStream) -> TokenStream {
    // ── Phase 1: produce the display string ──

    let raw = input.to_string();
    let block_start = raw.find('{').unwrap();
    let block_end = raw.rfind('}').unwrap();
    let inner = &raw[block_start + 1..block_end];
    let code_str = inner.trim_start_matches('\n').to_string();

    let s = source::dedent(&code_str);
    let s = source::restore_newlines(&s);
    let s = source::collapse_continuations(&s);
    let s = source::strip_comments(&s);
    let s = source::strip_ignores(&s);
    let display_str = source::strip_nested_steps(&s);

    // ── Phase 2: produce the runtime expansion ──

    let parsed = parse_macro_input!(input as StepInput);
    let comment_str = parsed.comment.value();
    let block = &parsed.block;
    let tag_refs: Vec<_> = parsed.tags.iter().collect();

    let expanded = if tag_refs.is_empty() {
        quote! {{
            let __step_guard = ::code_steps::display::enter_step(#comment_str);
            ::code_steps::display::print_step_separator();
            ::code_steps::display::print_step_header(#comment_str);
            ::code_steps::display::print_code(#display_str);
            let __result = #block;
            ::code_steps::display::press_any_key_if(&[], None);
            __result
        }}
    } else {
        quote! {{
            if ::code_steps::display::filter_matches(&[#(#tag_refs),*]) {
                let __step_guard = ::code_steps::display::enter_step(#comment_str);
                ::code_steps::display::print_step_separator();
                ::code_steps::display::print_step_header(#comment_str);
                ::code_steps::display::print_code(#display_str);
                let __result = #block;
                ::code_steps::display::press_any_key_if(&[], None);
                __result
            }
        }}
    };

    expanded.into()
}
