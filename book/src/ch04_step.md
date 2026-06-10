# The `step!` Macro

`step!` is the **primary macro** — everything else is used inside it.

Every `step!` does four things:

1. Prints a cyan `[description]` header.
2. Prints the source code with syntax highlighting.
3. Executes the code block.
4. Prints a green `ok` and auto-pauses.

The next three sections cover `step!` in detail:

- **Two forms** — with and without tags.
- **Auto-pause** — how steps pause automatically and show the path.
- **Return values** — chaining steps together.
