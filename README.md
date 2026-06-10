# code-steps

Notebook-style code step display for Rust. Show syntax-highlighted code in the terminal, step by step, with `step!` / `wait!` / `skip!` / `ignore!` control — ideal for examples, tutorials, and educational crates.

## Quick Start

```toml
[dependencies]
code-steps = "0.1"
```

```rust
use code_steps::{step, wait};

fn main() {
    step!("Create and save an image", {
        let img = Image::new(128, 128);
        img.save("output.png")?;
        wait!();
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
      ...                                           ← wait! pause
   ok                                               ← green

[Load it back]
   let img = Image::load("output.png")?;
   assert_eq!(img.width(), 128);
   ok
```

## Features

- **`step!`** — display + execute a code block as a named step
- **`wait!`** — pause until Enter
- **`skip!(("tag") { … })`** — shown, not executed (when tag active via `--include`)
- **`ignore!(("tag") { … })`** — executed, hidden from display
- **Tag filter** — control which calls are active from the command line (`--include a,b --exclude c`)
- **Syntax highlighting** — powered by [syntect](https://crates.io/crates/syntect), output to stderr

## Filtering

Call `init_wait_filter()` once at the top of `main()`:

```rust
use code_steps::{init_wait_filter, step, wait};

fn main() {
    init_wait_filter();

    step!("demo", {
        ignore!(("setup") { heavy_init(); })
        wait!("check");
    });
}
```

```text
cargo run --example demo -- --include check --exclude setup
```

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
| `step!("desc", { … })`       | yes   | yes      | no     |
| `step!("desc", "tag", { … })`| yes   | cond.    | no     |
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
