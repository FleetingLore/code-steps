# Return Values & Chaining

`step!` preserves the return value of its block, so you can **chain** steps
together — the output of one step feeds into the next.

## Basic chaining

```rust
let doubled = step!("Double a number", {
    let n = 21;
    n * 2       // ← returned from the step
});

step!("Use the result", {
    println!("Doubled value: {doubled}");
    assert_eq!(doubled, 42);
});
```

The first `step!` evaluates to `42`, which is bound to `doubled`.  The
second step uses it.  The display still works — header, code, ok, and
auto-pause fire normally around each block.

## Important: chaining only works with untagged steps

A tagged `step!` (Form 2) expands to an `if` expression without an `else`
branch.  If the filter blocks it, the expression evaluates to `()` — which
won't match the expected type.

```rust,ignore
// ❌ Won't compile — tagged step may return () when filtered out
let data = step!("Load", "basic", { vec![1, 2, 3] });
```

**Rule**: use Form 1 (no tags) for steps that produce values.  Use Form 2
(tagged) only for side-effect steps that don't return anything.

## Pattern: extract data in untagged steps, analyse in tagged steps

```rust
// Always runs, produces data
let raw = step!("Load data", {
    load_from_file("input.csv")
});

// Always runs, transforms
let clean = step!("Clean", {
    raw.into_iter().filter(|r| r.is_valid()).collect()
});

// Only runs when "basic" is active
step!("Basic stats", "basic", {
    println!("Mean: {}", clean.mean());
});

// Only runs when "advanced" is active
step!("Advanced stats", "advanced", {
    println!("Std dev: {}", clean.std_dev());
});
```

This pattern keeps the pipeline running regardless of which analysis
phases are active.

## Full example

```rust
{{#include ../../code-steps/examples/comprehensive.rs}}
```

Run it:

```text
$ cargo run --example comprehensive
$ cargo run --example comprehensive -- --include basic
```
