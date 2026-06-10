# `skip!`: Show Without Executing

`skip!` displays code in the terminal but **conditionally skips** its
execution at runtime.

## Syntax

```rust
skip!(("tag1", "tag2") { /* code shown, conditionally skipped */ })
```

Note the double parentheses: `(("tag") { body })`.  The outer `()` belong to
the macro invocation; the inner `("tag")` wraps the tags, followed by the
block.

## How it works

When a tag is **active** (passes the filter), `skip!` shows the code in the
terminal but does **not** execute it.  When the tag is **inactive**, the code
runs normally.

```rust,ignore
// User writes:
skip!(("debug") { dbg!(x); })

// Expands to:
if !filter_matches(&["debug"]) {
    { dbg!(x); }
}
```

## Use case: teaching Rust's borrow rules

When giving a talk about ownership, you might want to show both a "clean"
version and a verbose "debug trace" version side by side.  The debug version
clutters the output — you want the audience to **see it** in the source but
not run it every time.  `skip!` does exactly that.

## Full example

```rust
{{#include ../../code-steps/examples/skip.rs}}
```

Run it:

```text
$ cargo run --example skip

# Activate debug blocks (shown, not executed):
$ cargo run --example skip -- --include debug-borrow

# Activate unsafe demo (shown, not executed):
$ cargo run --example skip -- --include debug-unsafe
```
