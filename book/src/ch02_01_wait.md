# `wait!`: Interactive Pauses

`wait!` pauses execution and waits for the user to press Enter.  It prints a
yellow prompt showing the current step path.

## Two forms

| Syntax              | Behaviour                       |
|---------------------|---------------------------------|
| `wait![]`           | Always pauses                   |
| `wait!["t1", "t2"]` | Pauses only if filter allows    |

## When to use `wait!`

Since `step!` now **auto-pauses** at the end of every step, you rarely need
`wait![]` at step boundaries.  Use `wait!` **mid-step** to break a step into
sub-stages:

```rust
step!["Two-stage setup", {
    println!("Stage 1: allocating…");
    wait![];  // pause before stage 2
    println!("Stage 2: initialising…");
    // step auto-pauses here
}];
```

Or use `wait!["tag"]` for conditional checkpoints controlled by the tag
filter (see [Chapter 3](../ch03_tag_system.md)).

## Full example

```rust
{{#include ../../code-steps/examples/wait.rs}}
```

Run it:

```text
$ cargo run --example wait
```
