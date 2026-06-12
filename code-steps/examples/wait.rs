//! wait! — mid-step pauses (end-of-step pauses are now automatic).
//!
//! Run: `cargo run --example wait`
//!
//! Since `step!` now auto-pauses after every step, you only need `wait![]`
//! **inside** a step — to break a step into sub-stages, or to create
//! conditional pauses controlled by the tag filter.
//!
//! Demonstrates:
//! - `wait![]` — pause mid-step between two sub-operations
//! - `wait!["tag"]` — pause only if filter allows the tag
//! - Multiple pauses within a single step

use code_steps::step;

fn main() {
    code_steps::display::print_file_header("wait!");

    // ── Mid-step pause: break one step into sub-stages ─────────────────
    step!["Two-stage setup", {
        println!("Stage 1: allocating buffers…");
        let _buf = vec![0u8; 1024];
        code_steps::wait![]; // pause before stage 2
        println!("Stage 2: initialising connections…");
        // step auto-pauses after this.
    }];

    // ── Conditional mid-step pause ─────────────────────────────────────
    step!["Processing with optional checkpoint", {
        println!("Computing…");
        let result = 42;
        // Only pauses mid-step if "check" tag is active.
        code_steps::wait!["check"];
        println!("Result: {result}");
    }];

    // ── Multiple mid-step pauses (sub-stage breakdown) ─────────────────
    step!["Pipeline in one step", {
        println!("1/3 — loading");
        code_steps::wait!["step-1"];
        println!("2/3 — transforming");
        code_steps::wait!["step-2"];
        println!("3/3 — saving");
    }];
}
