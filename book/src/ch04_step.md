# Chapter 4: The `step!` Macro

`step!` is the **primary macro** — everything else is used inside it.

Every `step!` does four things:

1. Prints a cyan `[description]` header.
2. Prints the source code with syntax highlighting.
3. Executes the code block.
4. Prints a green `ok` and auto-pauses.

The next three sections cover `step!` in detail:

- **4.1** — Two forms: with and without tags.
- **4.2** — Auto-pause and how the path is displayed.
- **4.3** — Return values and chaining steps together.
