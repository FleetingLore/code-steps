//! step! — the primary macro: display + execute + auto-pause.
//!
//! Run: `cargo run --example step`
//!
//! Since `step!` now auto-pauses at the end of every step, you don't
//! need `wait![]` at step boundaries.  Use `wait![]` only for mid-step
//! pauses or conditional checkpoints.
//!
//! Demonstrates:
//! - `step!["desc", { … }]` — always runs, auto-pauses after
//! - `step!["desc", "tag", { … }]` — conditional via filter
//! - Comments are stripped from the display at compile time
//! - Nesting `wait!`, `skip!`, `ignore!` inside a step
//! - Return values: steps can produce results for later steps

use code_steps::step;

fn main() {
    code_steps::display::print_file_header("step!");

    // ── Basic step: always runs ────────────────────────────────────────
    step!["A simple calculation", {
        let x = 10;
        let y = 20;
        println!("{x} + {y} = {}", x + y);
    }];

    // ── Step with comments (comments are stripped from display) ────────
    step!["Processing with inline comments", {
        // This comment won't appear in the terminal
        let name = "Rust"; /* inline too */
        println!("Hello, {name}!");
    }];

    // ── Step that uses wait!, skip!, ignore! inside ────────────────────
    step!["Step with nested macros", {
        code_steps::ignore![("init") {
            println!("(silent init…)");
        }];

        println!("Visible output — pause here to explain.");
        code_steps::wait!["check"]; // mid-step pause

        code_steps::skip![("extra") {
            println!("Conditional extra output.");
        }];

        println!("Step continues after pause.");
        // auto-pause at end of step
    }];

    // ── Chaining: each step wraps its result ───────────────────────────
    let doubled = step!["Double a number", {
        let n = 21;
        n * 2
    }];

    step!["Use the result", {
        println!("Doubled value: {doubled}");
        assert_eq!(doubled, 42);
    }];
}
