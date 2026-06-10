# Auto-Pause & Path Display

`step!` automatically pauses at the end of every step.  No need to
manually insert `wait!()` at step boundaries.

## How it looks

```text
==========================================
[Load data]
   let data = vec![1, 2, 3];
   println!("{data:?}");
[1, 2, 3]
   Load data waiting        ← green prompt, press Enter to continue
```

The pause prompt shows the step name.  For nested steps, it shows the full
path (see [5.2 — Nested Steps](../ch05_02_nesting.md)).

## Implementation

Under the hood, each `step!` expansion includes:

```rust,ignore
{
    let __step_guard = enter_step("Step Name");  // push onto path stack
    print_step_header(…);
    print_code(…);
    let __result = { /* user code */ };
    print_step_done();
    press_any_key_if(&[]);  // prints "{path} waiting", waits for Enter
    __result
    // __step_guard drops here → pops path
}
```

The `StepGuard` uses RAII: it pushes the step name onto a thread-local
stack on creation, and pops it on drop.  This means even if the step body
contains `return` or `?` the path is correctly restored.

## Why auto-pause?

Before auto-pause, every step needed an explicit `wait!()` at the end.
This was repetitive and error-prone.  Auto-pause makes `step!` self-contained:
each step naturally stops so you can explain it.  Use `wait!()` only when
you need mid-step pauses or conditional checkpoints.
