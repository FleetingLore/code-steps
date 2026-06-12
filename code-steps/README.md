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
   let img = Image::new(128, 128);
   img.save("output.png")?;
    < Create and save an image

==========================================
[Load it back]
   let img = Image::load("output.png")?;
   assert_eq!(img.width(), 128);
    < Load it back
```

## Features

- **`step!`** — display + execute, auto-pause with transition prompts
- **`wait!["msg"]`** — mid-step pause, typewriter pauses at the exact point
- **`skip![("tag") { … }]`** — code shown, conditionally skipped
- **`ignore![("tag") { … }]`** — code hidden from display, always executed
- **Tag filter** — `--include` / `--exclude` on the command line
- **Typewriter mode** — characters appear one-by-one; press Enter to fast-forward
- **Nested steps** — visual indent, path tracking, sibling transitions
- **Syntax highlighting** — powered by [syntect](https://crates.io/crates/syntect), output to stderr

## Filtering

`step!` has two forms:

```rust
step!["Always runs", {
    // no tags → always displays and executes
}];

step!["Conditional", "tag1", "tag2", {
    // only runs if filter allows these tags
}];
```

Call `init_wait_filter()` once at the top of `main()`, then control from CLI:

```bash
cargo run -- --include basic
cargo run -- --include basic,advanced --exclude debug
```

## Typewriter mode

```rust
fn main() {
    code_steps::display::set_typewriter(true);          // enable
    code_steps::display::set_typewriter_speed(8);       // ms per char
    code_steps::display::set_typewriter_line_pause(60); // ms after newline

    step!["Demo", {
        let x = 1;
        wait!["checkpoint"];  // typewriter pauses here
        let y = x + 1;
    }];
}
```

Press **Enter** to fast-forward the current line.

Configure defaults in `Cargo.toml`:

```toml
[package.metadata.code-steps]
typewriter = true
typewriter-speed = 5
typewriter-line-pause = 60
```

## Nested steps

```rust
step!["Compile", {
    step!["Tokenise", { /* … */ }];
    step!["Parse",    { /* … */ }];
    step!["Type-check", {
        step!["Check main", { /* … */ }];
    }];
}];
```

Transitions:
- `Compile: Tokenise` — entering child
- `Tokenise > Parse` — sibling switch
- `Compile < Type-check` — last child exits to parent
- `< Compile` — parent exits

## API

### Macros

| Macro | Shows | Executes | Pauses |
|-------|-------|----------|--------|
| `step!["desc", { … }]` | yes | yes | auto |
| `step!["desc", "tag", { … }]` | yes | cond. | auto |
| `wait![]` | — | — | yes |
| `wait!["msg"]` | — | — | yes |
| `wait!["t1", "t2"]` | — | — | cond. |
| `skip![("t1") { … }]` | yes | cond. neg | no |
| `ignore![("t1") { … }]` | no | yes | no |

### Display functions

| Function | Purpose |
|----------|---------|
| `init_wait_filter()` | Parse CLI args |
| `set_typewriter(on)` | Enable typewriter |
| `set_typewriter_speed(ms)` | Character delay |
| `set_typewriter_line_pause(ms)` | Line-end pause |
| `print_step_separator()` | Cyan `=====` separator |
| `print_step_header(comment)` | Cyan `[comment]` header |
| `print_code(code)` | Syntax-highlighted code |
| `press_any_key_if(tags, msg)` | Path/message + wait for Enter |

## License

MIT OR Apache-2.0
