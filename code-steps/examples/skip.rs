//! skip! — show code, execute conditionally.
//!
//! Run:
//!   cargo run --example skip
//!   cargo run --example skip -- --include 'debug-*'
//!   cargo run --example skip -- --exclude verbose
//!
//! # What this demonstrates
//!
//! When teaching Rust's borrow rules, you often want to show both the
//! "correct" pattern and a verbose "debug" version that traces every
//! borrow and move.  The debug version clutters the output — you want the
//! audience to see it in the source, but not run it every time.
//!
//! `skip!` solves this: the debug code is **displayed** in the terminal
//! (so the audience sees the pattern), but **skipped at runtime**.  The
//! clean version executes instead.
//!
//! # How tagging changes behaviour
//!
//! | Command                                    | Effect |
//! |--------------------------------------------|--------|
//! | `cargo run --example skip`                 | All skip blocks execute (no filter → all inactive) |
//! | `cargo run --example skip -- --include 'debug-*'` | Debug blocks are SHOWN but NOT executed |
//! | `cargo run --example skip -- --exclude verbose` | Verbose blocks show & execute (not skipped) |

use code_steps::step;

fn main() {
    code_steps::display::print_file_header("skip! — borrow rules with optional debug traces");

    // ═══════════════════════════════════════════════════════════════════
    // Demo 1: mutable borrow — clean vs verbose debug trace
    // ═══════════════════════════════════════════════════════════════════
    //
    // The rule: you cannot mutate a Vec while an immutable reference to
    // it is still live.  The clean version drops the reference before
    // pushing.  The debug version traces every step — useful to show
    // the concept, noisy to run every time.

    step!("Mutable borrow: clean vs debug", {
        let mut data = vec![10, 20, 30];

        // ── Clean version (always runs) ──
        let first = &data[0];
        println!("First element: {first}");
        // `first` is dropped here (last use).
        data.push(40);
        println!("After push: {data:?}");

        // ── Debug version (shown, skipped when tag is active) ──
        // Expands each step into a trace: "borrow here → drop here → mutate".
        code_steps::skip!(("debug-borrow") {
            let mut data2 = vec![10, 20, 30];
            println!("  [trace] allocating vec at {:p}", &data2);
            {
                let first2 = &data2[0];
                println!("  [trace] borrowed data2[0] = {first2}");
                println!("  [trace] immutable borrow ends here");
            }
            println!("  [trace] borrow dropped, safe to mutate");
            data2.push(40);
            println!("  [trace] pushed 40 → {data2:?}");
        });
    });

    // ═══════════════════════════════════════════════════════════════════
    // Demo 2: ownership & clone — minimal vs detailed
    // ═══════════════════════════════════════════════════════════════════
    //
    // The rule: moving a String transfers ownership.  The clean version
    // clones before the move.  The debug version traces every allocation.

    step!("Use after move: clean vs debug", {
        let name = String::from("Rust");

        // ── Clean version (always runs) ──
        let greeting = format!("Hello, {name}!");
        println!("{greeting}");

        // ── Debug version (shown, skipped when tag is active) ──
        code_steps::skip!(("debug-borrow") {
            let name2 = String::from("Rust");
            println!("  [trace] name2 allocated at {:p} (len={})",
                     name2.as_ptr(), name2.len());
            let greeting2 = name2; // move
            println!("  [trace] moved into greeting2 at {:p} (len={})",
                     greeting2.as_ptr(), greeting2.len());
            // name2 is now invalid — can't use it.
            println!("  [trace] greeting = \"{greeting2}\"");
        });
    });

    // ═══════════════════════════════════════════════════════════════════
    // Demo 3: unsafe vs safe — concept demo
    // ═══════════════════════════════════════════════════════════════════

    step!("Raw pointer vs reference", {
        // ── Safe version (always runs) ──
        let x = 42;
        let r = &x;
        println!("Safe reference: {r}");

        // ── Unsafe version (shown, skipped when tag is active) ──
        code_steps::skip!(("debug-unsafe") {
            let x = 42;
            let r: *const i32 = &x;
            let addr = r as usize;
            println!("  [trace] raw pointer: 0x{addr:X}");
            unsafe {
                println!("  [trace] dereferenced: {}", *r);
            }
        });
    });
}
