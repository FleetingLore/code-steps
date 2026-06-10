//! The `step!` proc macro — dual-phase: display prep + code generation.
//!
//! `step!` is the most complex macro in this crate.  It does two things in
//! two distinct phases, at two different times:
//!
//! ## Phase 1 — compile-time source processing
//!
//! The raw source text between the block's braces is run through the
//! [`source`](super::source) pipeline:
//!
//! ```text
//! raw source  →  [dedent] → [strip_comments] → [strip_ignores] → display
//! ```
//!
//! The resulting `display_str` is a `&str` literal baked into the binary.
//! No runtime cost for processing — the terminal just prints a pre-computed
//! string.  This is why you can use comments freely in step blocks: they get
//! stripped at compile time.
//!
//! ## Phase 2 — runtime expansion
//!
//! The parsed AST ([`StepInput`]) drives code generation via [`quote`].
//! The expansion wraps the user's block with display calls:
//!
//! ```rust,ignore
//! {
//!     print_step_header(comment);
//!     print_code(display_str);      // from Phase 1
//!     let __result = { user code };  // the original block, unmodified
//!     print_step_done();
//!     __result
//! }
//! ```
//!
//! If the step has **tags**, the body is additionally wrapped in a runtime
//! filter check (`filter_matches`) so the entire step can be skipped via
//! `--include` / `--exclude` on the command line.
//!
//! ## Why two phases?
//!
//! Phase 1 cannot run on the parsed AST because `syn` doesn't preserve
//! whitespace and comments.  But we *need* both: the raw source for display
//! (Phase 1) and the typed AST for code generation (Phase 2).  So `step!`
//! processes the same input twice, through two different representations.
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
    let s = source::strip_comments(&s);
    let display_str = source::strip_ignores(&s);

    // ── Phase 2: produce the runtime expansion ──

    let parsed = parse_macro_input!(input as StepInput);
    let comment_str = parsed.comment.value();
    let block = &parsed.block;
    let tag_refs: Vec<_> = parsed.tags.iter().collect();

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
