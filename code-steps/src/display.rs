//! Terminal display helpers for the [`step!`] macro.
//!
//! # Architecture
//!
//! All output goes to **stderr** so stdout stays clean for program output.
//! Three layers:
//!
//! | Layer          | What                               |
//! |----------------|------------------------------------|
//! | Highlighting   | syntect lazy-init, theme selection |
//! | Step display   | header / code / "ok" printing      |
//! | Wait filter    | global tag system for wait/skip/ignore |
//!
//! # Theme
//!
//! Selected at compile time via `[package.metadata.code-steps] theme`
//! in `Cargo.toml` (processed by `build.rs` → `env!("CODE_STEPS_THEME")`).
//!
//! Syntect bundles these themes (plus many more):
//!
//! ```text
//! ayu-dark              (bundled in this crate — syntect doesn't ship it)
//! Solarized (dark)      Solarized (light)
//! base16-ocean.dark     base16-ocean.light
//! base16-eighties.dark  base16-mocha.dark
//! InspiredGitHub
//! ```
//!
//! [`step!`]: https://docs.rs/code-steps/latest/code_steps/macro.step.html

use std::io::{self, Write};
use std::sync::OnceLock;

use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

// ── Theme ──────────────────────────────────────────────────────────────────
//
// build.rs reads Cargo.toml and emits `cargo:rustc-env=CODE_STEPS_THEME=<name>`.
// This module reads that env var at compile time.  Any syntect theme key works.

/// Theme key — set by `build.rs`, defaults to `"ayu-dark"`.
static THEME: &str = env!("CODE_STEPS_THEME");

/// Lazily load the syntax set and theme set.  The `ayu-dark` theme is
/// bundled as `themes/ayu-dark.tmTheme` because syntect 5 doesn't ship it;
/// all other themes come from [`ThemeSet::load_defaults`].
fn highlighting() -> &'static (SyntaxSet, ThemeSet) {
    use std::sync::OnceLock;
    static H: OnceLock<(SyntaxSet, ThemeSet)> = OnceLock::new();
    H.get_or_init(|| {
        let ss = SyntaxSet::load_defaults_newlines();
        let mut ts = ThemeSet::load_defaults();

        // Bundle ayu-dark — syntect dropped it in v5.
        let ayu = include_str!("themes/ayu-dark.tmTheme");
        if let Ok(theme) = ThemeSet::load_from_reader(&mut std::io::Cursor::new(ayu)) {
            ts.themes.insert("ayu-dark".into(), theme);
        }

        (ss, ts)
    })
}

/// Print each line of `code` with 3-space indent and 24-bit terminal escapes.
fn print_highlighted(code: &str) {
    let (ss, ts) = highlighting();
    let syntax = ss.find_syntax_by_name("Rust").expect("Rust syntax");
    let theme = &ts.themes[THEME];
    let mut h = HighlightLines::new(syntax, theme);

    for line in LinesWithEndings::from(code) {
        let ranges = h.highlight_line(line, ss).unwrap();
        let _ = write!(io::stderr(), "   ");
        for (style, text) in ranges {
            let _ = write!(
                io::stderr(),
                "{}",
                syntect::util::as_24_bit_terminal_escaped(&[(style, text)], false)
            );
        }
    }
}

// ── Step display API ──────────────────────────────────────────────────────
//
// Called by the `step!` macro expansion.  Print order:
//   1. print_step_header  — cyan "[comment]" line
//   2. print_code         — syntax-highlighted body
//   3. (user code runs, possibly with wait!/skip!/ignore!)
//   4. print_step_done    — green "   ok" line

/// Bold header with blank lines before and after.
pub fn print_file_header(filename: &str) {
    let _ = writeln!(io::stderr());
    let _ = writeln!(io::stderr(), "\x1b[1m──── {} ────\x1b[0m", filename);
    let _ = writeln!(io::stderr());
}

/// Cyan `[comment]` line introducing the next step.
pub fn print_step_header(comment: &str) {
    let _ = writeln!(io::stderr(), "\x1b[36m[{}]\x1b[0m", comment);
}

/// Print the dedented, comment-stripped, ignore-stripped source of the step
/// block with syntax highlighting.
pub fn print_code(code: &str) {
    print_highlighted(code);
}

/// Green `   ok` marker after the step body has finished executing.
pub fn print_step_done() {
    let _ = writeln!(io::stderr(), "\x1b[32m   ok\x1b[0m\n");
}

// ── Wait / skip / ignore filter ───────────────────────────────────────────
//
// A single global [`WaitFilter`] (stored in a `OnceLock`) controls which
// `wait!()` / `skip!()` / `ignore!()` calls are active at runtime.
//
// Initialised by calling [`init_wait_filter`] once at the start of `main()`.
// If never called, the filter is absent and **all** calls are active
// (backward-compatible default).
//
// CLI format (args after `--`):
//
// ```text
// cargo run --example demo -- --include a,b --exclude c
// ```
//
// Filter logic (applied to the tags of each call):
//
//   include empty? ──yes──→ pass
//          │
//          no: any tag ∈ include? ──yes──→ pass
//          │
//          no ──→ blocked
//
//   THEN
//
//   any tag ∈ exclude? ──yes──→ blocked
//          │
//          no ──→ pass

#[derive(Debug, Default)]
pub struct WaitFilter {
    include: Vec<String>,
    exclude: Vec<String>,
}

/// Global filter instance.  `None` means "not initialised → allow all".
static FILTER: OnceLock<WaitFilter> = OnceLock::new();

/// Parse `--include` and `--exclude` from [`std::env::args`] and store them.
/// Call once at the top of `main()`.
pub fn init_wait_filter() {
    let args: Vec<String> = std::env::args().collect();
    let mut filter = WaitFilter::default();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--include" if i + 1 < args.len() => {
                i += 1;
                filter.include = args[i].split(',').map(|s| s.trim().to_string()).collect();
            }
            "--exclude" if i + 1 < args.len() => {
                i += 1;
                filter.exclude = args[i].split(',').map(|s| s.trim().to_string()).collect();
            }
            _ => {}
        }
        i += 1;
    }

    let _ = FILTER.set(filter);
}

/// Called by the `wait!()` macro expansion.
/// Prints `"    ..."` in yellow and waits for Enter if the filter allows.
pub fn press_any_key_if(tags: &[&str]) {
    if let Some(filter) = FILTER.get() {
        if !filter.matches(tags) {
            return;
        }
    }
    let _ = writeln!(io::stderr(), "\x1b[33m    ...\x1b[0m");
    let mut buf = String::new();
    let _ = io::stdin().read_line(&mut buf);
}

/// Called by `skip!()` / `step!("…", "tag", …)`.  Returns `true` when the
/// filter considers `tags` active (or hasn't been initialised).
pub fn filter_matches(tags: &[&str]) -> bool {
    FILTER.get().map_or(true, |f| f.matches(tags))
}

impl WaitFilter {
    fn matches(&self, tags: &[&str]) -> bool {
        let passes_include =
            self.include.is_empty() || tags.iter().any(|t| self.include.iter().any(|i| i == t));
        let passes_exclude =
            tags.is_empty() || !tags.iter().any(|t| self.exclude.iter().any(|e| e == t));
        passes_include && passes_exclude
    }
}
