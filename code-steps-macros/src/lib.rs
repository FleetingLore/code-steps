//! Proc macros for the `code-steps` crate.
//!
//! # Architecture overview
//!
//! This crate contains four proc macros.  Because Rust requires
//! `#[proc_macro]` functions to live in the crate root, this file contains
//! only thin entry points that delegate to the corresponding modules.  All
//! documentation and logic lives in the modules:
//!
//! | Module              | Role                                            |
//! |---------------------|-------------------------------------------------|
//! | [`step`]            | `step!` — dual-phase: display prep + code gen   |
//! | [`source`]          | Phase 1 text pipeline (`dedent` → `strip_comments` → `strip_ignores`) |
//! | [`wait`]            | `wait!` — conditional or unconditional pause    |
//! | [`skip`]            | `skip!` — show code, execute conditionally      |
//! | [`ignore`]          | `ignore!` — hide code, always execute           |
//!
//! # The four macros at a glance
//!
//! | Macro     | Shows code  | Executes code | Pauses  |
//! |-----------|------------|---------------|---------|
//! | `step!`   | yes        | yes           | auto    |
//! | `wait!`   | yes        | —             | yes     |
//! | `skip!`   | yes        | conditional   | no      |
//! | `ignore!` | no (hidden)| yes           | no      |
//!
//! All four share the same tag system, controlled at runtime by
//! [`code_steps::display::init_wait_filter`].
//!
//! # Design highlights
//!
//! ## The `step!` dual-phase design
//!
//! `step!` is the most complex macro.  It does two independent things:
//!
//! **Phase 1 — compile-time source processing** ([`source`]):
//!
//! ```text
//! raw source  →  dedent  →  strip_comments  →  strip_ignores  →  strip_nested_steps  →  display string
//! ```
//!
//! The display string is baked into the binary.  No runtime cost for
//! processing — the terminal just prints a pre-computed `&str`.
//!
//! **Phase 2 — runtime expansion** ([`step`]):
//!
//! ```rust,ignore
//! // Input:  step!["description", { /* user code */ }]
//! //         step!["description", "tag"…, { /* user code */ }]
//! //
//! // Expands to (simplified):
//! {
//!     pause_if_nested(comment);      // if nesting, shows path & pauses
//!     let __step_guard = enter_step(comment);
//!     print_step_separator();
//!     print_step_header(comment);
//!     print_code(display_str);       // from Phase 1, typewriter if enabled
//!     let __result = { /* user code */ };
//!     press_any_key_if(&[]);         // auto-pause, shows path if nested
//!     __result
//!     // __step_guard drops here → pops path
//! }
//! ```
//!
//! ## Parser sharing
//!
//! [`skip!`] and [`ignore!`] share the same input syntax `("tag"…) { body }`
//! — they use the same [`SkipInput`] parser defined in [`skip`].
//!
//! ## Display vs. execution
//!
//! `skip!` and `ignore!` are symmetric complements:
//!
//! - `skip!`: **shown** in terminal, **not** executed (when tag active)
//! - `ignore!`: **hidden** from terminal, **always** executed
//!
//! Together they let you control the visible narrative without changing
//! what actually runs.
//!
//! [`code_steps::display::init_wait_filter`]: https://docs.rs/code-steps/latest/code_steps/display/fn.init_wait_filter.html
//! [`print_code`]: https://docs.rs/code-steps/latest/code_steps/display/fn.print_code.html
//! [`SkipInput`]: skip::SkipInput
//! [`skip!`]: skip
//! [`ignore!`]: ignore

use proc_macro::TokenStream;

mod ignore;
mod skip;
mod source;
mod step;
mod wait;

// ── Entry points ───────────────────────────────────────────────────────────
//
// Each #[proc_macro] function must live in the crate root (Rust requirement).
// They delegate immediately to the corresponding module — the real logic and
// documentation lives there.

#[proc_macro]
pub fn step(input: TokenStream) -> TokenStream {
    step::step_impl(input)
}

#[proc_macro]
pub fn wait(input: TokenStream) -> TokenStream {
    wait::wait_impl(input)
}

#[proc_macro]
pub fn skip(input: TokenStream) -> TokenStream {
    skip::skip_impl(input)
}

#[proc_macro]
pub fn ignore(input: TokenStream) -> TokenStream {
    ignore::ignore_impl(input)
}
