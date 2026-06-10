//! Comprehensive — all four macros + tag filtering in a realistic scenario.
//!
//! Run:
//!   cargo run --example comprehensive
//!   cargo run --example comprehensive -- --include basic
//!   cargo run --example comprehensive -- --include advanced --exclude debug
//!
//! Scenario: a data pipeline that loads → cleans → analyses → exports.
//! Each phase is a step.  Tags control which analysis phases run and
//! which debug/verbose content is shown.

use code_steps::{init_wait_filter, step};

fn main() {
    init_wait_filter();

    code_steps::display::print_file_header("Data Pipeline");

    // ═══════════════════════════════════════════════════════════════════
    // Phase 0 — Hidden setup
    // ═══════════════════════════════════════════════════════════════════

    step!("Initialisation", {
        code_steps::ignore!(("warm-cache") {
            println!("Warming cache… (hidden from display)");
            std::thread::sleep(std::time::Duration::from_millis(100));
        });

        code_steps::ignore!(("load-config") {
            println!("Loading config… (hidden from display)");
        });

        println!("Pipeline ready.");
    });

    // ═══════════════════════════════════════════════════════════════════
    // Phase 1 — Load data (always runs, shared by all)
    // ═══════════════════════════════════════════════════════════════════

    let raw_data: Vec<i32> = step!("Load input data", {
        let data = vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3];

        code_steps::skip!(("debug") {
            println!("DEBUG: raw data = {data:?}");
        });

        data
    });

    // ═══════════════════════════════════════════════════════════════════
    // Phase 2 — Clean (always runs)
    // ═══════════════════════════════════════════════════════════════════

    let clean_data = step!("Clean and normalise", {
        let mut data = raw_data.clone();
        data.sort();
        let data: Vec<i32> = data.into_iter().filter(|&x| x > 2).collect();

        code_steps::skip!(("debug") {
            println!("DEBUG: after clean = {data:?}");
            println!("DEBUG: removed {} outliers", raw_data.len() - data.len());
        });

        data
    });

    // ═══════════════════════════════════════════════════════════════════
    // Phase 3 — Basic analysis (only when "basic" is active)
    // ═══════════════════════════════════════════════════════════════════

    step!("Basic statistics", "basic", {
        let sum: i32 = clean_data.iter().sum();
        let avg = sum as f64 / clean_data.len() as f64;
        println!("Count : {}", clean_data.len());
        println!("Sum   : {sum}");
        println!("Avg   : {avg:.2}");
    });

    // ═══════════════════════════════════════════════════════════════════
    // Phase 4 — Advanced analysis (only when "advanced" is active)
    // ═══════════════════════════════════════════════════════════════════

    step!("Advanced analysis", "advanced", {
        let min = clean_data.iter().min().unwrap();
        let max = clean_data.iter().max().unwrap();
        let median = clean_data[clean_data.len() / 2];

        println!("Min    : {min}");
        println!("Max    : {max}");
        println!("Median : {median}");

        code_steps::skip!(("debug") {
            println!("DEBUG: variance calculation…");
            let mean = clean_data.iter().sum::<i32>() as f64 / clean_data.len() as f64;
            let var: f64 = clean_data
                .iter()
                .map(|&x| (x as f64 - mean).powi(2))
                .sum::<f64>()
                / clean_data.len() as f64;
            println!("DEBUG: variance = {var:.4}");
        });
    });

    // ═══════════════════════════════════════════════════════════════════
    // Phase 5 — Export (both basic + advanced)
    // ═══════════════════════════════════════════════════════════════════

    step!("Export results", "basic", "advanced", {
        println!("Writing report…");

        code_steps::ignore!(("verbose") {
            println!("  chapter 1: overview");
            println!("  chapter 2: methodology");
            println!("  chapter 3: results");
        });

        println!("Export complete.");
    });

    // ═══════════════════════════════════════════════════════════════════
    // Final — always runs
    // ═══════════════════════════════════════════════════════════════════

    step!("Done", {
        println!("Pipeline finished successfully.");
    });
}
