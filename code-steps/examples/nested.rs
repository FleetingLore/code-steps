//! Nested steps — step inside step, with path display.
//!
//! Run: `cargo run --example nested`
//!
//! # What this demonstrates
//!
//! `step!` can be nested: an inner `step!` inside an outer `step!`.
//! The auto-pause shows the full nesting path so you always know where
//! you are in the execution:
//!
//! ```
//! [Outer Step]
//!    ... code ...
//!    ok
//!    Outer Step waiting          ← auto-pause for outer
//!
//!    [Inner Step]
//!       ... code ...
//!       ok
//!       Outer Step : Inner Step waiting  ← auto-pause shows path
//! ```
//!
//! # Use case
//!
//! When demonstrating a multi-layered algorithm — e.g. a compiler pass
//! that tokenises, then parses, then type-checks — you can nest steps
//! so the terminal output reflects the logical structure of your code.

use code_steps::step;

fn main() {
    code_steps::display::print_file_header("Nested steps");

    // ═══════════════════════════════════════════════════════════════════
    // Scenario: a compiler pipeline with sub-passes
    // ═══════════════════════════════════════════════════════════════════

    step!["Compile source", {
        let source = "fn main() { println!(\"hi\"); }";
        println!("Source: {source}");

        // ── Nested step: tokenisation ──
        step!["Tokenise", {
            let tokens = vec![
                "fn", "main", "(", ")", "{", "println!", "(", "\"hi\"", ")", ";", "}",
            ];
            println!("Tokens: {tokens:?}");
        }];

        // ── Nested step: parsing ──
        step!["Parse", {
            println!("Building AST…");
            let _ast = "Program { functions: [main] }";
            println!("AST built.");
        }];

        // ── Nested step: type-check — can nest further ──
        step!["Type-check", {
            println!("Checking types…");

            step!["Check main signature", {
                println!("  fn main() → () — ok");
            }];

            step!["Check println! call", {
                println!("  println!(\"hi\") → () — ok");
            }];

            println!("All checks passed.");
        }];

        println!("Compilation successful.");
    }];

    // ═══════════════════════════════════════════════════════════════════
    // Three-level nesting
    // ═══════════════════════════════════════════════════════════════════

    step!["Data pipeline", {
        let data = vec![5, 3, 8, 1];

        step!["Sort", {
            let mut sorted = data.clone();
            sorted.sort();

            step!["Verify order", {
                assert!(sorted.windows(2).all(|w| w[0] <= w[1]));
                println!("Order verified: {sorted:?}");
            }];

            println!("Sorting done.");
        }];

        step!["Summarise", {
            let data = vec![1, 3, 5, 8];
            let sum: i32 = data.iter().sum();
            println!("Sum = {sum}");
        }];
    }];
}
