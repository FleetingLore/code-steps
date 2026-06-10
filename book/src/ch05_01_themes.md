# Theme Switching

code-steps uses [syntect] for syntax highlighting.  You can change the
colour theme by setting a single value in your `Cargo.toml`.

## Picking a theme

Add this to your **own** `Cargo.toml` (the crate that depends on code-steps):

```toml
[package.metadata.code-steps]
theme = "base16-ocean.dark"
```

The theme is selected at **compile time** — no runtime cost.  `build.rs`
reads the value and passes it to the highlighting engine.

## Available themes

Any [syntect theme key][syntect-themes] works.  Commonly used ones:

| Theme               | Key                    | Vibe |
|---------------------|------------------------|------|
| Ayu Dark (default)  | `ayu-dark`             | Warm, modern dark |
| Solarized Dark      | `Solarized (dark)`     | Classic low-contrast dark |
| Solarized Light     | `Solarized (light)`    | Classic low-contrast light |
| Base16 Ocean Dark   | `base16-ocean.dark`    | Blue-tinted dark |
| Base16 Ocean Light  | `base16-ocean.light`   | Blue-tinted light |
| Base16 Eighties     | `base16-eighties.dark` | Retro warm dark |
| Base16 Mocha        | `base16-mocha.dark`    | Brown-tinted dark |
| Inspired GitHub     | `InspiredGitHub`       | GitHub-like light |

## How it works

1. Your `Cargo.toml` sets `[package.metadata.code-steps] theme`.
2. `code-steps`'s `build.rs` reads it at compile time and emits
   `cargo:rustc-env=CODE_STEPS_THEME=<name>`.
3. `display.rs` reads `env!("CODE_STEPS_THEME")` at compile time.
4. At runtime, the syntect [`ThemeSet`] is queried with that key.

> **Note**: `ayu-dark` is bundled with code-steps because syntect v5
> dropped it.  All other themes come from syntect's built-in set.

[syntect]: https://crates.io/crates/syntect
[syntect-themes]: https://docs.rs/syntect/latest/syntect/highlighting/struct.ThemeSet.html#method.load_defaults
[`ThemeSet`]: https://docs.rs/syntect/latest/syntect/highlighting/struct.ThemeSet.html
