//! Notebook-style code step display for Rust.
//!
//! Show syntax-highlighted code in the terminal, step by step,
//! with `wait!()` / `skip!()` / `ignore!()` control.
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use code_steps::{step, wait};
//!
//! step!("1. Create an image", {
//!     let img = Image::new(128, 128);
//!     img.save("output.png")?;
//!     wait!();
//! });
//! ```
//!
//! # Filtering
//!
//! ```rust,ignore
//! code_steps::display::init_wait_filter();
//! step!("render", {
//!     ignore!(("setup") { expensive_init(); })  // runs, hidden
//!     skip!(("debug")   { dbg!(x); })           // shown, not run
//!     wait!("check");                          // pause if active
//! });
//! ```
//!
//! ```text
//! cargo run --example demo -- --include check --exclude debug
//! ```

pub mod display;
pub use code_steps_macros::{ignore, skip, step, wait};
pub use display::init_wait_filter;
