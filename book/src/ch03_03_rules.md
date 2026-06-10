# Filter Rules & Macro Behaviour

The filter evaluates each macro invocation's tags against the global
include/exclude lists.  The logic has two stages:

## Stage 1 — Include check

```
include list empty? ──yes──→ PASS (allow all)
       │
       no: any of the invocation's tags ∈ include list?
              │
              yes ──→ PASS
              │
              no  ──→ BLOCKED
```

## Stage 2 — Exclude check (only if Stage 1 passed)

```
any of the invocation's tags ∈ exclude list?
       │
       yes ──→ BLOCKED
       │
       no  ──→ PASS
```

## Concrete examples

Assume `--include basic,advanced --exclude debug`:

| Invocation tags     | Include check               | Exclude check       | Result  |
|---------------------|-----------------------------|---------------------|---------|
| `["basic"]`         | "basic" ∈ include → PASS    | "basic" ∉ exclude → PASS | **PASS** |
| `["advanced"]`      | "advanced" ∈ include → PASS | "advanced" ∉ exclude → PASS | **PASS** |
| `["debug"]`         | "debug" ∉ include → BLOCKED | —                   | **BLOCKED** |
| `["basic", "debug"]`| "basic" ∈ include → PASS    | "debug" ∈ exclude → BLOCKED | **BLOCKED** |
| `[]` (no tags)      | (empty → PASS)              | (empty → PASS)      | **PASS** |

Key insight: a step with **no tags** always runs — the filter doesn't apply
to it.

## Per-macro behaviour

| Macro     | When PASS                          | When BLOCKED               |
|-----------|------------------------------------|----------------------------|
| `step!`   | Shows header + code, executes, auto-pauses | Entirely skipped — nothing shown, nothing run |
| `wait!`   | Pauses for Enter                   | No pause                   |
| `skip!`   | Code **shown**, block **not executed** | Code shown & executed  |
| `ignore!` | Code **hidden**, block executed    | Code **hidden**, block executed (always runs) |

## Full example

The `filter.rs` example demonstrates all of this in action:

```rust
{{#include ../../code-steps/examples/filter.rs}}
```

Try different invocations:

```text
$ cargo run --example filter
$ cargo run --example filter -- --include basic
$ cargo run --example filter -- --include advanced --exclude debug
$ cargo run --example filter -- --exclude verbose
```
