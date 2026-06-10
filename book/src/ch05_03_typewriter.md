# Typewriter Mode

When enabled, code is printed **character-by-character** with a small delay
between each character and a longer pause at the end of each line —
simulating a live-typing effect.

## Enabling

Enable at runtime, anywhere before the first `step!`:

```rust
fn main() {
    code_steps::display::set_typewriter(true);
    // … steps will now type out
}
```

Or set a compile-time default in `Cargo.toml`:

```toml
[package.metadata.code-steps]
typewriter = true
```

## Configuration

Three knobs, all settable at runtime:

| Function | Default | Effect |
|----------|---------|--------|
| `set_typewriter(on)` | `false` | Enable / disable |
| `set_typewriter_speed(ms)` | `15` | Delay between characters |
| `set_typewriter_line_pause(ms)` | `150` | Pause after each newline |

```rust
code_steps::display::set_typewriter(true);
code_steps::display::set_typewriter_speed(8);
code_steps::display::set_typewriter_line_pause(80);
```

## Fast-forward with Enter

During typewriter display, pressing **Enter** immediately finishes the
current line and continues typing the next line at normal speed.  If it's
the last line of the step, Enter jumps straight to the waiting prompt.

This means you can control the pacing interactively — let the typewriter
run at its own speed, or tap Enter to skip ahead one line at a time.

## How it works

The typewriter is implemented in [`print_code`] — when enabled, it calls
a character-by-character output loop instead of the normal bulk print.

### Non-blocking stdin

To detect Enter presses without blocking the typewriter, stdin is put into
**non-blocking mode** via `fcntl(F_SETFL, O_NONBLOCK)`.  Between each
character, a quick `read(2)` checks for pending input.  If a newline is
found, fast-forward triggers; otherwise the typewriter continues.

During typewriter display, the terminal is also switched to **raw mode**
(`tcsetattr` with `ICANON | ECHO | ISIG` cleared).  This:
- Disables line buffering (keypresses arrive immediately)
- Disables terminal echo (typed characters don't appear on screen)
- Disables signal generation (Ctrl+C is handled by the program, not the terminal)

Raw mode is restored via a `Drop` guard when the typewriter finishes.

### Per-character highlighting

Each character is wrapped in its own syntax-highlighting escape sequence
so the colour updates correctly as characters appear.  For performance,
the syntect [`ThemeSet`] and [`SyntaxSet`] are loaded once via `OnceLock`.

```rust,ignore
// Simplified: per-character output with highlighting
for ch in text.chars() {
    if try_read_enter() { fast_forward = true; break; }
    write!(stderr, "{}", highlight(&[(style, &ch.to_string())]));
    flush();
    sleep(speed);
}
```

[`print_code`]: https://docs.rs/code-steps/latest/code_steps/display/fn.print_code.html
[`ThemeSet`]: https://docs.rs/syntect/latest/syntect/highlighting/struct.ThemeSet.html
[`SyntaxSet`]: https://docs.rs/syntect/latest/syntect/parsing/struct.SyntaxSet.html
