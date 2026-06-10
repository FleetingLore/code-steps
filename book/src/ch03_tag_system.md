# The Tag System

Tags are the bridge between your source code and the command line.  They let
you write **one program** that can be run in **many different ways** just by
changing CLI arguments — no code changes needed.

The next three sections cover:

- **Tags** — what they are and where you attach them.
- **`init_wait_filter`** — how it parses `--include` / `--exclude` from the command line.
- **Filter rules** — the exact logic and how each macro responds.
