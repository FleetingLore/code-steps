//! playground — explore different `code-steps` macros interactively.
//!
//! Run: `cargo run --example playground`
//!
//! Each step demonstrates a different macro or feature.
//! Press Enter to advance through the steps.

use code_steps::step;

fn main() {
    code_steps::display::print_file_header("playground");

    // ── Basic step with wait! pause ──
    step!("Basic step + pause", {
        let greeting = "Hello, code-steps!";
        println!("{greeting}");
        code_steps::wait!();
    });

    // ── step! without tags shows & executes unconditionally ──
    step!("Unconditional step (always runs and shows)", {
        let answer = 42;
        println!("The answer is {answer}");
    });

    // ── skip! is always shown but runs conditionally ──
    step!("Demo: skip! macro", {
        code_steps::skip!(("demo-skip") {
            println!("This only runs when demo-skip is NOT active.");
        });
        println!("This line always runs.");
    });

    // ── ignore! always runs but is hidden from display ──
    step!("Demo: ignore! macro", {
        code_steps::ignore!(("demo-ignore") {
            println!("Boilerplate init — runs but hidden from terminal.");
        });
        println!("Visible code after init.");
    });

    // ── wait! with tags ──
    step!("Conditional wait", {
        println!("Before conditional pause...");
        code_steps::wait!("demo-check");
        println!("After pause!");
    });

    // ── Nested skip + ignore inside one step ──
    step!("Nested skip + ignore inside one step", {
        code_steps::ignore!(("nested-setup") {
            std::thread::sleep(std::time::Duration::from_millis(200));
        });

        code_steps::skip!(("nested-debug") {
            println!("Debug info (shown, runs only when tag inactive).");
        });

        code_steps::wait!("nested-check");
        println!("Main logic here.");
    });
}
