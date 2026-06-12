//! The `skip!` proc macro — show code, execute conditionally.
//!
//! `skip!` is the complement of [`ignore!`](super::ignore): the code is
//! **shown** in the terminal display, but **executed only when the tag is
//! NOT active**.
//!
//! ## Expansion
//!
//! ```rust,ignore
//! // User writes:
//! skip![("debug") { dbg!(x); }]
//!
//! // Expands to:
//! if !::code_steps::display::filter_matches(&["debug"]) {
//!     { dbg!(x); }
//! }
//! ```
//!
//! The display string still contains the original `skip![(…) { … }]` because
//! `step!`'s Phase 1 works on the raw source — it doesn't see this expanded `if`.
//!
//! ## Parser (shared with `ignore!`)
//!
//! Uses [`SkipInput`] to parse the `("tag", …) { body }` syntax.  Note the
//! outer `[` `]` belong to the macro invocation; the parser receives tokens
//! *inside* them, starting with another `(` `)` pair for the tags, then the
//! block.

use proc_macro::TokenStream;
use quote::quote;
use syn::{Block, LitStr, Token, parse_macro_input, punctuated::Punctuated};

// ── SkipInput parser (shared with `ignore!`) ───────────────────────────────

/// Parsed form of `("tag", …) { … }`, shared by [`skip!`] and [`ignore!`].
///
/// [`skip!`]: crate::skip
/// [`ignore!`]: crate::ignore
pub struct SkipInput {
    pub tags: Punctuated<LitStr, Token![,]>,
    pub block: Block,
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

// ── Implementation (called from lib.rs entry point) ────────────────────────

pub fn skip_impl(input: TokenStream) -> TokenStream {
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
