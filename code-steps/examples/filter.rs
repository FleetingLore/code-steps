//! Tag filter — control which steps run from the command line.
//!
//! Run:
//!   cargo run --example filter
//!   cargo run --example filter -- --include basic
//!   cargo run --example filter -- --include advanced --exclude debug
//!   cargo run --example filter -- --exclude verbose
//!
//! Demonstrates:
//! - `init_wait_filter()` parsing `--include` / `--exclude` CLI args
//! - `step!` with tags: only runs when filter allows
//! - `wait!` with tags: only pauses when filter allows
//! - Steps without tags always run

use code_steps::{init_wait_filter, step};

fn main() {
    init_wait_filter();

    code_steps::display::print_file_header("Tag filter demo");

    // ── Tagged step: only runs if "basic" passes the filter ────────────
    step!["Basic analysis", "basic", {
        println!("Running basic analysis…");
        let data = vec![1, 2, 3];
        println!("Data: {data:?}");
    }];

    // ── Tagged step: only runs if "advanced" passes the filter ─────────
    step!["Advanced analysis", "advanced", {
        println!("Running advanced analysis…");
        let sum: i32 = (1..100).sum();
        println!("Sum 1..100 = {sum}");

        code_steps::skip![("debug") {
            // Shown but won't execute when "debug" is active (--include debug)
            println!("DEBUG: internal state dump");
        }];
    }];

    // ── Tagged step with multiple tags ─────────────────────────────────
    step!["Visualisation", "basic", "advanced", {
        println!("Generating charts…");
        println!("Done.");
    }];

    // ── No tags — always runs ──────────────────────────────────────────
    step!["Cleanup", {
        code_steps::ignore![("verbose") {
            println!("Verbose: freeing temporary buffers…");
        }];
        println!("All done.");
    }];
}
