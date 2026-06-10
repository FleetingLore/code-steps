//! `build.rs` — compile-time theme selection.
//!
//! Reads `[package.metadata.code-steps] theme` from this crate's
//! `Cargo.toml` and exposes it as `CODE_STEPS_THEME`.  Users configure
//! the theme in their own `Cargo.toml` under the same key.
//!
//! Typewriter mode and speed are configured at runtime via
//! [`display::set_typewriter`], [`display::set_typewriter_speed`], and
//! [`display::set_typewriter_line_pause`].
//!
//! ```toml
//! [package.metadata.code-steps]
//! theme = "base16-ocean.dark"
//! ```
//!
//! [`display`]: crate::display
//! [`display::set_typewriter`]: crate::display::set_typewriter
//! [`display::set_typewriter_speed`]: crate::display::set_typewriter_speed
//! [`display::set_typewriter_line_pause`]: crate::display::set_typewriter_line_pause

use std::fs;

fn main() {
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let toml: toml::Value =
        toml::from_str(&fs::read_to_string(format!("{dir}/Cargo.toml")).unwrap()).unwrap();

    let theme = toml["package"]["metadata"]["code-steps"]["theme"]
        .as_str()
        .unwrap_or("ayu-dark");
    println!("cargo:rustc-env=CODE_STEPS_THEME={theme}");

    let meta = &toml["package"]["metadata"]["code-steps"];

    if let Some(tw) = meta.get("typewriter").and_then(|v| v.as_bool()) {
        println!("cargo:rustc-env=CODE_STEPS_TYPEWRITER={tw}");
    }
    if let Some(n) = meta.get("typewriter-speed").and_then(|v| v.as_integer()) {
        println!("cargo:rustc-env=CODE_STEPS_TYPEWRITER_SPEED={n}");
    }
    if let Some(n) = meta
        .get("typewriter-line-pause")
        .and_then(|v| v.as_integer())
    {
        println!("cargo:rustc-env=CODE_STEPS_TYPEWRITER_LINE_PAUSE={n}");
    }
}
