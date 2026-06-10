# What Are Tags?

A **tag** is a string label attached to a macro invocation.  All four macros
support tags: `step!`, `wait!`, `skip!`, and `ignore!`.

## Where tags go

```rust
// step! — tags between description and block
step!("Description", "tag1", "tag2", { … });

// wait! — tags inside parens
wait!("tag1", "tag2");

// skip! and ignore! — tags in inner parens before the block
skip!(("tag1", "tag2") { … });
ignore!(("tag1", "tag2") { … });
```

## What tags do

Tags control whether a macro invocation is **active** or **blocked** at
runtime.  The decision is made by a global filter (see
[3.2](ch03_02_init_filter.md)) that reads command-line arguments.

| If the tag is…      | `step!` does…                | `wait!` does…    | `skip!` does…          | `ignore!` does…     |
|---------------------|------------------------------|------------------|------------------------|---------------------|
| Active (passes)     | Shows & executes             | Pauses           | Shows, **skips** exec  | Hides, always exec  |
| Blocked (fails)     | Entirely skipped             | No pause         | Shows & executes       | Hides, always exec  |

## Choosing tag names

Tag names are arbitrary strings — choose whatever makes sense for your
use case.  Common patterns:

| Pattern        | Example tags              | Use case |
|----------------|---------------------------|----------|
| Difficulty     | `"basic"`, `"advanced"`   | One example, different audience levels |
| Debug level    | `"debug"`, `"verbose"`    | Show/hide diagnostic output |
| Feature flag   | `"charts"`, `"export"`    | Conditional features in a demo |
| Step identity  | `"load"`, `"clean"`       | Coarse-grained step control |
