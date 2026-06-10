# code-steps

Notebook-style code step display for Rust. Show syntax-highlighted code in the terminal, step by step, with `step!` / `wait!` / `skip!` / `ignore!` control — ideal for examples, tutorials, and educational crates.

## Quick Start

```toml
[dependencies]
code-steps = "0.1"
```

```rust
use code_steps::step;

fn main() {
    step!("Create and save an image", {
        let img = Image::new(128, 128);
        img.save("output.png")?;
    });

    step!("Load it back", {
        let img = Image::load("output.png")?;
        assert_eq!(img.width(), 128);
    });
}
```

Terminal output:

```
[Create and save an image]                         ← cyan comment
   let img = Image::new(128, 128);                  ← syntax-highlighted
   img.save("output.png")?;
   ok                                               ← green
      ...                                           ← auto-pause

[Load it back]
   let img = Image::load("output.png")?;
   assert_eq!(img.width(), 128);
   ok
      ...                                           ← auto-pause
```

## Features

- **`step!`** — display + execute a code block, auto-pauses at the end
- **`wait!`** — pause mid-step (or conditional pause via tags)
- **`skip!(("tag") { … })`** — shown, not executed (when tag active via `--include`)
- **`ignore!(("tag") { … })`** — executed, hidden from display
- **Tag filter** — control which calls are active from the command line (`--include a,b --exclude c`)
- **Syntax highlighting** — powered by [syntect](https://crates.io/crates/syntect), output to stderr

## Filtering

### Tagging steps

`step!` has two forms:

```rust
step!("Always runs", {
    // no tags → always displays and executes
});

step!("Conditional step", "tag1", "tag2", {
    // only runs if filter allows these tags
});
```

Tags connect your source code to command-line flags.  Call
`init_wait_filter()` once at the top of `main()`, then control which
steps run from the command line:

```rust
use code_steps::{init_wait_filter, step, wait, skip, ignore};

fn main() {
    init_wait_filter();

    step!("Setup", "basic", "advanced", {
        ignore!(("setup") { heavy_init(); })
        println!("Ready.");
    });

    step!("Basic analysis", "basic", {
        let data = load_sample();
        wait!("basic");
    });

    step!("Advanced analysis", "advanced", {
        run_heavy_computation();
        wait!("advanced");
    });

    step!("Cleanup", {
        // no tags — always runs
        save_results();
    });
}
```

### Controlling from the command line

Pass `--include` / `--exclude` after `--`:

```text
# Only show steps tagged "basic"
cargo run -- --include basic

# Show steps tagged "basic" or "advanced", but hide "debug" content
cargo run -- --include basic,advanced --exclude debug

# No flags → all steps run, all pauses active (default)
cargo run
```

### Filter rules

- **`--include a,b`** — only steps/tags matching `a` or `b` are active.
  If omitted, **all tags pass**.
- **`--exclude c`** — steps/tags matching `c` are blocked, even if they
  passed `--include`.
- A step with **no tags** always runs — the filter doesn't apply.

### How each macro responds to the filter

| Macro   | When tag is active (`--include`) | When tag is blocked |
|---------|----------------------------------|---------------------|
| `step!` | Shows + executes                 | Entirely skipped    |
| `wait!` | Pauses for Enter                 | No pause            |
| `skip!` | Code shown, **not** executed     | Code executed       |
| `ignore!` | Code executed, **hidden**      | Code executed       |

## Nested steps

`step!` can be nested — each inner step shows its path in the auto-pause:

```rust
step!("Compile", {
    step!("Tokenise", { /* … */ });
    step!("Parse",    { /* … */ });
    step!("Type-check", {
        step!("Check main", { /* … */ });
    });
});
```

Terminal output:

```
[Compile]
   …
   ok
   Compile waiting

   [Tokenise]
      …
      ok
      Compile : Tokenise waiting       ← path shows nesting

   [Type-check]
      [Check main]
         …
         ok
         Compile : Type-check : Check main waiting   ← three levels deep
```

See `examples/nested.rs` for a full demo.

## Themes

Set in `Cargo.toml`:

```toml
[package.metadata.code-steps]
theme = "base16-ocean.dark"
```

Any syntect theme key works.  Commonly used ones:

| Theme               | Key                    |
|---------------------|------------------------|
| Ayu Dark (default)  | `ayu-dark`             |
| Solarized Dark      | `Solarized (dark)`     |
| Solarized Light     | `Solarized (light)`    |
| Base16 Ocean Dark   | `base16-ocean.dark`    |
| Base16 Ocean Light  | `base16-ocean.light`   |
| Base16 Eighties     | `base16-eighties.dark` |
| Base16 Mocha        | `base16-mocha.dark`    |
| Inspired GitHub     | `InspiredGitHub`       |

## API

### Macros

| Macro                        | Shows | Executes | Pauses |
|------------------------------|-------|----------|--------|
| `step!("desc", { … })`       | yes   | yes      | auto    |
| `step!("desc", "tag", { … })`| yes   | cond.    | auto    |
| `wait!()`                    | yes   | —        | yes    |
| `wait!("t1", "t2")`          | yes   | —        | cond.  |
| `skip!(("t1") { … })`        | yes   | cond. neg| no     |
| `ignore!(("t1") { … })`      | no    | yes      | no     |

### Display functions (stderr)

| Function | Purpose |
|----------|---------|
| `init_wait_filter()` | Parse CLI args, install global filter |
| `print_file_header(path)` | Bold header with surrounding blank lines |
| `print_step_header(comment)` | Cyan `[comment]` line |
| `print_code(code)` | Syntax-highlighted code |
| `print_step_done()` | Green `ok` |
| `press_any_key_if(tags)` | Yellow prompt + wait for Enter (if filter allows) |
| `filter_matches(tags)` | Query whether tags pass the current filter |

## License

MIT OR Apache-2.0
