//! demo — full-featured showcase of `code-steps`.
//!
//! Run:
//!   cargo run --example demo
//!   cargo run --example demo -- --include step2,check --exclude setup
//!
//! CLI flags (after `--`):
//!   --include a,b     only show / activate steps with these tags
//!   --exclude c       hide / deactivate steps with these tags
//!
//! Tags used: setup, step1, check, step2

use code_steps::{ignore, init_wait_filter, skip, step, wait};

fn main() {
    // Initialise the global tag filter from CLI args.
    init_wait_filter();

    code_steps::display::print_file_header("demo");

    // ── ignore! — runs but never shown in terminal ──
    step!("Heavy setup (hidden via ignore)", "setup", {
        ignore!(("setup") {
            // Simulate expensive init — this code executes but won't appear.
            std::thread::sleep(std::time::Duration::from_millis(300));
            let _data = vec![0u8; 1024];
        });
        println!("Setup complete!");
        wait!("setup");
    });

    // ── skip! — shown but only executed when tag is NOT active ──
    step!("Step 1: Process data", "step1", {
        let data = vec![1, 2, 3, 4, 5];
        let sum: i32 = data.iter().sum();
        println!("Sum = {sum}");

        skip!(("debug") {
            // This prints only when "debug" tag is excluded / not active.
            dbg!(&data);
        });

        wait!("check");
    });

    // ── Step 2: conditional via tag ──
    step!("Step 2: Transform results", "step2", {
        let doubled: Vec<i32> = (0..5).map(|x| x * 2).collect();
        println!("Doubled: {doubled:?}");

        wait!("check");
    });

    // ── Unconditional step (no tags — always runs) ──
    step!("Final step", {
        println!("All done!");
    });
}
