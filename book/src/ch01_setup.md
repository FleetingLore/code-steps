# Chapter 1: Environment Setup

## What is code-steps?

`code-steps` is a Rust crate that turns your terminal into a **notebook-style
presentation tool**.  It shows syntax-highlighted code step by step, pauses
for input, and lets you control what runs from the command line — ideal for
tutorials, live demos, and educational crates.

## Adding the dependency

Add `code-steps` to your `Cargo.toml`:

```toml
[dependencies]
code-steps = "0.2"
```

## Your first program

Create `src/main.rs`:

```rust
use code_steps::step;

fn main() {
    step!("Hello, code-steps!", {
        println!("This is my first step.");
    });
}
```

Run it:

```text
$ cargo run

──── Hello, code-steps! ────

[Hello, code-steps!]
   println!("This is my first step.");
This is my first step.
   ok

   Hello, code-steps! waiting
```

Every `step!` block does four things:

1. Prints a cyan `[description]` header.
2. Prints the source code with **syntax highlighting**.
3. Executes the code.
4. Prints a green `ok` and **auto-pauses** — press Enter to continue.

All display output goes to **stderr**, so your `println!` output (stdout) stays
clean.

## Project structure

If you're following along, your project should look like this:

```text
my-demo/
├── Cargo.toml
└── src/
    └── main.rs
```

That's it — you're ready to start using code-steps.
