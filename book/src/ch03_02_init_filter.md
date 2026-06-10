# 3.2 — `init_wait_filter` & The CLI

`init_wait_filter()` reads `--include` and `--exclude` from
[`std::env::args`] and installs a **global filter** that all macros
consult at runtime.

## Calling it

Call `init_wait_filter()` exactly once, at the top of `main()`:

```rust
use code_steps::init_wait_filter;

fn main() {
    init_wait_filter();
    // … rest of your program
}
```

If you never call it, the filter is absent and **all** tags are active
(backward-compatible default).

## CLI syntax

Pass flags after `--` so Cargo forwards them to your program:

```text
$ cargo run -- --include basic
$ cargo run -- --include basic,advanced --exclude debug
$ cargo run -- --exclude verbose
```

- `--include a,b` — comma-separated list of tags to activate.
- `--exclude c` — comma-separated list of tags to block.
- Both can be used together.
- Without flags, all tags are active (everything runs).

## Parsing logic

The function scans `std::env::args()` positionally.  The value immediately
after `--include` (or `--exclude`) is split on commas, trimmed, and stored.

```text
args: ["--include", "basic,advanced", "--exclude", "debug"]
                              ↑                     ↑
                    include = ["basic", "advanced"]   exclude = ["debug"]
```

[`std::env::args`]: https://doc.rust-lang.org/std/env/fn.args.html
