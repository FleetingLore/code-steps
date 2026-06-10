//! basics — minimal `code-steps` usage.
//!
//! Run: `cargo run --example basics`

use code_steps::display::{print_code, print_file_header, print_step_done, print_step_header};

fn main() {
    print_file_header("basics");

    // ── Step 1: using the high-level `step!` macro ──
    code_steps::step!("A simple step with a code block", {
        let name = "Rust";
        println!("Hello from {name}!");

        // pause for Enter
        code_steps::wait!();
    });

    // ── Step 2: manual display API ──
    print_step_header("Manual display calls (no macro)");
    print_code("let x = 42;\nlet y = x * 2;\nprintln!(\"{y}\");");
    let x = 42;
    let y = x * 2;
    println!("{y}");
    print_step_done();
}
