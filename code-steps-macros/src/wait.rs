//! The `wait!` proc macro — conditional or unconditional pause.
//!
//! `wait!` is the simplest non-trivial macro.  It expands to a call to
//! [`press_any_key_if`], which checks the global tag filter and either
//! prints a yellow `"    ..."` prompt and waits for Enter, or returns
//! immediately.
//!
//! ## Forms
//!
//! | Syntax                | Behaviour                     |
//! |-----------------------|-------------------------------|
//! | `wait![]`             | Always pauses, shows step path |
//! | `wait!["message"]`    | Always pauses, shows custom message |
//! | `wait!["t1", "t2"]`  | Pauses only if filter allows  |
//!
//! ## How it fits
//!
//! `wait!` is typically used *inside* a `step!` block to create an
//! interactive breakpoint.  When the user presses Enter, the step continues
//! and `print_step_done()` fires.
//!
//! [`press_any_key_if`]: https://docs.rs/code-steps/latest/code_steps/display/fn.press_any_key_if.html

use proc_macro::TokenStream;
use quote::quote;
use syn::{LitStr, Token, parse_macro_input, punctuated::Punctuated};

pub fn wait_impl(input: TokenStream) -> TokenStream {
    if input.is_empty() {
        return quote! { ::code_steps::display::press_any_key_if(&[], None) }.into();
    }

    let tags = parse_macro_input!(input with Punctuated::<LitStr, Token![,]>::parse_terminated);
    if tags.len() == 1 {
        let msg = &tags[0];
        quote! { ::code_steps::display::press_any_key_if(&[], Some(#msg)) }.into()
    } else {
        let tag_refs: Vec<_> = tags.iter().collect();
        quote! { ::code_steps::display::press_any_key_if(&[#(#tag_refs),*], None) }.into()
    }
}
