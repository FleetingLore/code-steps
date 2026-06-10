# Nested Steps

`step!` can be nested — an inner `step!` inside an outer `step!`'s block.
The auto-pause shows the full nesting path so you always know where you are.

## Basic nesting

```rust
step!("Compile", {
    step!("Tokenise", {
        println!("Breaking into tokens…");
    });
    step!("Parse", {
        println!("Building AST…");
    });
});
```

## Path display

When nested, the yellow pause prompt shows the full path:

```text
==========================================
[Compile]
   Compile waiting                    ← depth 1

   ===========================================
   [Tokenise]
      Compile : Tokenise waiting      ← depth 2: "Outer : Inner"

   ===========================================
   [Parse]
      Compile : Parse waiting
```

## Three-level nesting

```text
==========================================
[Compile]
   ===========================================
   [Type-check]
      ===========================================
      [Check main]
         Compile : Type-check : Check main waiting   ← depth 3
```

## How it works

Each `step!` pushes its name onto a thread-local stack via `enter_step()`.
The auto-pause reads the stack and joins with `" : "`.  A `StepGuard` (RAII)
ensures the name is popped even if the step body returns early.

This means:

- **Early returns** (`return`, `?`) are safe — the guard cleans up.
- **Panics** unwind the stack correctly — guards drop during unwinding.
- **Deep nesting** works — any number of levels.

## Use case: compiler pipeline demo

When teaching a compiler course, you can nest steps to reflect the actual
compiler phases:

```rust
step!("Compile source.rs", {
    step!("Lexing",    { /* tokenise */ });
    step!("Parsing",   { /* build AST */ });
    step!("Type-check", {
        step!("Check main signature", { … });
        step!("Infer types",          { … });
    });
    step!("Codegen",   { /* emit LLVM IR */ });
});
```

Each phase is a step; sub-checks are nested steps.  The terminal output
mirrors the logical structure of the compiler.

## Full example

```rust
{{#include ../../code-steps/examples/nested.rs}}
```

Run it:

```text
$ cargo run --example nested
```
