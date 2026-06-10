//! `build.rs` — compile-time theme selection.
//!
//! Reads `[package.metadata.code-steps] theme` from `Cargo.toml` and
//! exposes it as the `CODE_STEPS_THEME` environment variable so
//! [`display`] can pick the matching syntect theme at compile time.
//!
//! Any theme key known to [`ThemeSet::load_defaults`] works (case-sensitive).
//! Additionally, `ayu-dark` is bundled as `themes/ayu-dark.tmTheme`.
//!
//! ```toml
//! [package.metadata.code-steps]
//! theme = "base16-ocean.dark"
//! ```
//!
//! [`display`]: crate::display
//! [`ThemeSet::load_defaults`]: syntect::highlighting::ThemeSet::load_defaults

use std::fs;

fn main() {
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let toml: toml::Value =
        toml::from_str(&fs::read_to_string(format!("{dir}/Cargo.toml")).unwrap()).unwrap();
    let theme = toml["package"]["metadata"]["code-steps"]["theme"]
        .as_str()
        .unwrap_or("ayu-dark");
    println!("cargo:rustc-env=CODE_STEPS_THEME={theme}");
}
