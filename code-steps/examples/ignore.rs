//! ignore! — execute code, hide from display.
//!
//! Run: `cargo run --example ignore`
//!
//! # What this demonstrates
//!
//! When you have a series of steps that all need the same setup (loading a
//! config, building an index, opening a connection), showing that setup in
//! *every* step creates visual noise.  `ignore!` solves this: the setup
//! still runs every time, but it only appears in the terminal once — or
//! not at all.
//!
//! # How tagging changes behaviour
//!
//! Without filter init, all `ignore!` blocks run and are hidden.
//! With `--exclude setup`, the block under `ignore!(("setup") { … })`
//! still executes (ignore always runs), but the placeholder `// (ignored)`
//! disappears from the display because the surrounding `step!` is tagged
//! `"setup"`, and `--exclude setup` skips the entire step.
//!
//! Try:
//!   cargo run --example ignore
//!   cargo run --example ignore -- --exclude setup
//!   cargo run --example ignore -- --exclude analytics

use code_steps::step;

fn main() {
    code_steps::display::print_file_header("ignore! — hiding repeated setup");

    // ── Scenario: three analysis steps that all need the same index ───
    //
    // Without ignore!, the build_index() call would appear in every step's
    // display, making the output repetitive and hard to follow.

    // ── Step 1: build once, reuse in analysis 1 ────────────────────────
    step!("Build search index", {
        // Expensive one-time build — shown here to explain what's happening.
        let index = build_fake_index();
        println!("Index built: {index} entries.");
    });

    // ── Step 2–4: reuse index via ignore! — visual noise eliminated ───

    step!("Query 1: exact match", {
        // The rebuild is necessary (this is a demo, no global state),
        // but showing it three times would be repetitive.
        code_steps::ignore!(("rebuild-index") {
            let _index = build_fake_index(); // runs, hidden from display
        });

        let results = vec!["rustic", "rust", "trust"];
        println!("Results for 'rust': {results:?}");
    });

    step!("Query 2: fuzzy match", {
        code_steps::ignore!(("rebuild-index") {
            let _index = build_fake_index(); // runs, hidden
        });

        let results = vec!["hello", "help", "held"];
        println!("Results for 'hel~': {results:?}");
    });

    step!("Query 3: regex", {
        code_steps::ignore!(("rebuild-index") {
            let _index = build_fake_index(); // runs, hidden
        });

        let results = vec!["apple", "apply", "app"];
        println!("Results for '^app': {results:?}");
    });
}

fn build_fake_index() -> usize {
    std::thread::sleep(std::time::Duration::from_millis(150));
    42
}
