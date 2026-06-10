# 4.1 — Two Forms of `step!`

`step!` has two forms, distinguished by the presence of tags:

## Form 1 — No tags (always runs)

```rust
step!("Description", {
    // This always displays and executes.
});
```

The expansion wraps the block with display calls and auto-pause.  No filter
check — it runs unconditionally.

## Form 2 — With tags (conditional)

```rust
step!("Description", "tag1", "tag2", {
    // This only runs if the filter allows these tags.
});
```

The expansion adds a runtime `if filter_matches(&["tag1", "tag2"])` guard.
If the tags don't pass the filter, the entire step is skipped — nothing
shown, nothing executed.

## When to use each

| Situation | Use |
|-----------|-----|
| Steps that should always appear in every run | Form 1 (no tags) |
| Steps that are optional, controlled from CLI | Form 2 (with tags) |
| Loading data, cleanup, final messages | Form 1 |
| Analysis phases, debug sections, optional demos | Form 2 |

## Full example

```rust
{{#include ../../code-steps/examples/step.rs}}
```

Run it:

```text
$ cargo run --example step
```
