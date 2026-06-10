# Chapter 2: The Three Basic Macros

Before diving into the main `step!` macro, let's understand the three
supporting macros.  They are always used **inside** a `step!` block and
give you fine-grained control over visibility and interactivity.

| Macro     | Displayed in terminal | Executed at runtime | Pauses |
|-----------|-----------------------|---------------------|--------|
| `wait!`   | Yes (yellow prompt)   | —                   | Yes    |
| `skip!`   | Yes                   | Conditional         | No     |
| `ignore!` | No (hidden)           | Yes                 | No     |

Think of them as operating on two independent axes:

- **Display axis**: `skip!` shows the code in terminal, `ignore!` hides it.
- **Execution axis**: `skip!` may skip execution, `ignore!` always runs.
- **Pause axis**: `wait!` is the only one that pauses for input.

The next three sections explore each macro in detail with runnable examples.
