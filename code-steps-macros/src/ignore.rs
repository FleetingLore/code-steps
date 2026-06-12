//! The `ignore!` proc macro — hide code, always execute.
//!
//! `ignore!` is the complement of [`skip!`](super::skip): the code is
//! **hidden** from the terminal display, but **always runs** at runtime.
//!
//! It operates at two levels:
//!
//! | Level     | What happens                                        |
//! |-----------|-----------------------------------------------------|
//! | Runtime   | Block always executes (tags NOT checked)            |
//! | Display   | Replaced with `// (ignored)` by Phase 1             |
//!
//! ## Expansion
//!
//! ```rust,ignore
//! // User writes:
//! ignore![("setup") { heavy_init(); }]
//!
//! // Expands to:
//! { { heavy_init(); } }
//! ```
//!
//! The outer `{}` is the block expression — this is intentionally minimal.
//! The code runs as-is, with zero added overhead.
//!
//! ## Hidden from display
//!
//! `step!`'s Phase 1 ([`strip_ignores`]) scans the raw source text and
//! replaces every `ignore![…] { … }` block with `// (ignored)`.
//! pure text replacement at compile time — the proc macro expansion
//! (this file) is only concerned with runtime execution.
//!
//! [`strip_ignores`]: super::source::strip_ignores

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

use crate::skip::SkipInput;

pub fn ignore_impl(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as SkipInput);
    let block = &parsed.block;
    quote! { { #block } }.into()
}
