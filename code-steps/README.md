# code-steps

Notebook-style code step display for Rust. Show syntax-highlighted code in the terminal, step by step, with `step!` / `wait!` / `skip!` / `ignore!` control — ideal for examples, tutorials, and educational crates.

## Quick Start

```toml
[dependencies]
code-steps = "0.4"
```

```rust
use code_steps::step;

fn main() {
    step!["Create and save an image", {
        let img = Image::new(128, 128);
        img.save("output.png")?;
    }];

    step!["Load it back", {
        let img = Image::load("output.png")?;
        assert_eq!(img.width(), 128);
    }];
}
```

Terminal output:

```
==========================================
[Create and save an image]
   let img = Image::new(128, 128);                  ← syntax-highlighted
   img.save("output.png")?;
   Create and save an image waiting                 ← auto-pause

==========================================
[Load it back]
   let img = Image::load("output.png")?;
   assert_eq!(img.width(), 128);
   Load it back waiting
```

Each step shows a cyan `=====` separator, a cyan `[description]` header,
syntax-highlighted code, and an auto-pause prompt.  No `ok` line — the
separator signals the next step.

## Features

- **`step!`** — display + execute, with auto-pause and nested path tracking
- **`wait!`** — mid-step pause (conditional via tags)
- **`skip![("tag") { … }]`** — code shown, conditionally skipped
- **`ignore![("tag") { … }]`** — code hidden from display, always executed
- **Tag filter** — `--include` / `--exclude` on the command line
- **Typewriter mode** — characters appear one-by-one; press Enter to fast-forward
- **Syntax highlighting** — powered by [syntect](https://crates.io/crates/syntect), output to stderr

## Filtering

### Tagging steps

`step!` has two forms:

```rust
step!["Always runs", {
    // no tags → always displays and executes
}];

step!["Conditional step", "tag1", "tag2", {
    // only runs if filter allows these tags
}];
```

Tags connect your source code to command-line flags.  Call
`init_wait_filter()` once at the top of `main()`, then control which
steps run from the command line:

```rust
use code_steps::{init_wait_filter, step, ignore};

fn main() {
    init_wait_filter();

    step!["Setup", "basic", "advanced", {
        ignore![("setup") { heavy_init(); }]
        println!("Ready.");
    }];

    step!["Basic analysis", "basic", {
        let data = load_sample();
        println!("Analysing {data:?}…");
    }];

    step!["Advanced analysis", "advanced", {
        run_heavy_computation();
        println!("Done.");
    }];

    step!["Cleanup", {
        save_results();
    }];
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

## Typewriter mode

Enable typewriter display so code appears character-by-character:

```rust
fn main() {
    code_steps::display::set_typewriter(true);         // enable
    code_steps::display::set_typewriter_speed(8);      // ms per char (default 15)
    code_steps::display::set_typewriter_line_pause(60); // ms after newline (default 150)

    step!["Demo", { /* types out character by character */ }];
}
```

Press **Enter** during typing to fast-forward the current line.  The next
line continues at normal speed.  On the last line, Enter jumps straight to
the waiting prompt.

Configure defaults in `Cargo.toml`:

```toml
[package.metadata.code-steps]
typewriter = true
typewriter-speed = 5
typewriter-line-pause = 60
```

## Nested steps

`step!` can be nested — inner steps are visually indented and the auto-pause
shows the full path:

```rust
step!["Compile", {
    step!["Tokenise", { /* … */ }];
    step!["Parse",    { /* … */ }];
    step!["Type-check", {
        step!["Check main", { /* … */ }];
    }];
}];
```

Terminal output:

```
==========================================
[Compile]
   let source = …;
   // Tokenise
   // Parse
   // Type-check
   Compile waiting

   ===========================================
   [Tokenise]
      let tokens = …;
      Compile : Tokenise waiting

   ===========================================
   [Type-check]
      // Check main signature

      ===========================================
      [Check main signature]
         println!(…);
         Compile : Type-check : Check main signature waiting
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
| `step!["desc", { … }]`       | yes   | yes      | auto   |
| `step!["desc", "tag", { … }]`| yes   | cond.    | auto   |
| `wait![]`                    | yes   | —        | yes    |
| `wait!["t1", "t2"]`          | yes   | —        | cond.  |
| `skip![("t1") { … }]`        | yes   | cond. neg| no     |
| `ignore![("t1") { … }]`      | no    | yes      | no     |

### Display functions (stderr)

| Function | Purpose |
|----------|---------|
| `init_wait_filter()` | Parse CLI args, install global filter |
| `print_file_header(path)` | Bold header with surrounding blank lines |
| `print_step_separator()` | Cyan `=====` separator the width of the terminal |
| `print_step_header(comment)` | Cyan `[comment]` line |
| `print_code(code)` | Syntax-highlighted code (typewriter if enabled) |
| `set_typewriter(on)` | Enable / disable typewriter display |
| `set_typewriter_speed(ms)` | Character delay in ms |
| `set_typewriter_line_pause(ms)` | Line-end pause in ms |
| `press_any_key_if(tags)` | Path prompt + wait for Enter |
| `filter_matches(tags)` | Query whether tags pass the current filter |

## License

MIT OR Apache-2.0
