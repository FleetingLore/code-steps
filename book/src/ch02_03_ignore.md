# `ignore!`: Execute Without Showing

`ignore!` is the complement of [`skip!`](../ch02_02_skip.md): code **always
executes** at runtime but is **hidden** from the terminal display.

## Syntax

```rust
ignore!(("tag") { /* code runs but is hidden */ })
```

## How it works

`ignore!` operates at two levels:

| Level     | What happens                                        |
|-----------|-----------------------------------------------------|
| Runtime   | Block always executes (tags NOT checked)            |
| Display   | Replaced with `// (ignored)` by compile-time Phase 1 |

The expansion is intentionally minimal — the block runs as-is with zero
overhead:

```rust,ignore
// User writes:
ignore!(("setup") { heavy_init(); })

// Expands to:
{ { heavy_init(); } }
```

Meanwhile, `step!`'s Phase 1 scans the raw source text and replaces every
`ignore!(…) { … }` block with `// (ignored)`.  This is pure text replacement
at compile time.

## Use case: hiding repeated setup

When you have a series of steps that all need the same expensive setup
(loading a config, building an index), showing that setup in *every* step
creates visual noise.  `ignore!` hides the repetition — the setup still runs
every time, but only appears in the terminal once.

## Full example

```rust
{{#include ../../code-steps/examples/ignore.rs}}
```

Run it:

```text
$ cargo run --example ignore
```
