//! Terminal display helpers for the [`step!`] macro.
//!
//! # Architecture
//!
//! All output goes to **stderr** so stdout stays clean for program output.
//! Three layers:
//!
//! | Layer          | What                               |
//! |----------------|------------------------------------|
//! | Highlighting   | syntect lazy-init, theme selection |
//! | Step display   | header / code / "ok" printing      |
//! | Wait filter    | global tag system for wait/skip/ignore |
//!
//! # Theme
//!
//! Selected at compile time via `[package.metadata.code-steps] theme`
//! in `Cargo.toml` (processed by `build.rs` → `env!("CODE_STEPS_THEME")`).
//!
//! Syntect bundles these themes (plus many more):
//!
//! ```text
//! ayu-dark              (bundled in this crate — syntect doesn't ship it)
//! Solarized (dark)      Solarized (light)
//! base16-ocean.dark     base16-ocean.light
//! base16-eighties.dark  base16-mocha.dark
//! InspiredGitHub
//! ```
//!
//! [`step!`]: https://docs.rs/code-steps/latest/code_steps/macro.step.html

use std::cell::RefCell;
use std::io::{self, Write};
use std::sync::OnceLock;

use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

// ── Theme ──────────────────────────────────────────────────────────────────
//
// build.rs reads Cargo.toml and emits `cargo:rustc-env=CODE_STEPS_THEME=<name>`.
// This module reads that env var at compile time.  Any syntect theme key works.

/// Theme key — set by `build.rs`, defaults to `"ayu-dark"`.
static THEME: &str = env!("CODE_STEPS_THEME");

/// Lazily load the syntax set and theme set.  The `ayu-dark` theme is
/// bundled as `themes/ayu-dark.tmTheme` because syntect 5 doesn't ship it;
/// all other themes come from [`ThemeSet::load_defaults`].
fn highlighting() -> &'static (SyntaxSet, ThemeSet) {
    use std::sync::OnceLock;
    static H: OnceLock<(SyntaxSet, ThemeSet)> = OnceLock::new();
    H.get_or_init(|| {
        let ss = SyntaxSet::load_defaults_newlines();
        let mut ts = ThemeSet::load_defaults();

        // Bundle ayu-dark — syntect dropped it in v5.
        let ayu = include_str!("themes/ayu-dark.tmTheme");
        if let Ok(theme) = ThemeSet::load_from_reader(&mut std::io::Cursor::new(ayu)) {
            ts.themes.insert("ayu-dark".into(), theme);
        }

        (ss, ts)
    })
}

/// Print each line of `code` with 3-space indent and 24-bit terminal escapes.
fn print_highlighted(code: &str) {
    let indent = step_indent();
    let (ss, ts) = highlighting();
    let syntax = ss.find_syntax_by_name("Rust").expect("Rust syntax");
    let theme = &ts.themes[THEME];
    let mut h = HighlightLines::new(syntax, theme);

    for line in LinesWithEndings::from(code) {
        let ranges = h.highlight_line(line, ss).unwrap();
        let _ = write!(io::stderr(), "{indent}   ");
        for (style, text) in ranges {
            let _ = write!(
                io::stderr(),
                "{}",
                syntect::util::as_24_bit_terminal_escaped(&[(style, text)], false)
            );
        }
    }
}

// ── Step display API ──────────────────────────────────────────────────────
//
// Called by the `step!` macro expansion.  Print order:
//   1. print_step_separator — cyan "====…" line (terminal width)
//   2. print_step_header   — cyan "[comment]" line
//   3. print_code          — syntax-highlighted body (typewriter if enabled)
//   4. (user code runs, possibly with wait!/skip!/ignore!)
//   5. press_any_key_if    — auto-pause, shows path + "waiting"

// ── Step nesting path ────────────────────────────────────────────────────
//
// When `step!` calls are nested, a thread-local stack tracks the path.
// `enter_step()` pushes the step name and returns a guard that pops on
// drop.  `press_any_key_if()` reads the stack to display the current
// nesting path (e.g. "Load : Parse : Validate waiting").

thread_local! {
    static STEP_PATH: RefCell<Vec<String>> = RefCell::new(Vec::new());
}

/// RAII guard: pushes the step name on creation, pops on drop.
/// Created by [`enter_step`] and embedded in the `step!` expansion.
pub struct StepGuard;

impl Drop for StepGuard {
    fn drop(&mut self) {
        STEP_PATH.with(|p| {
            p.borrow_mut().pop();
        });
    }
}

/// Push `name` onto the nesting stack and return a guard that pops on drop.
pub fn enter_step(name: &str) -> StepGuard {
    STEP_PATH.with(|p| p.borrow_mut().push(name.to_string()));
    StepGuard
}

/// Bold header with blank lines before and after.
pub fn print_file_header(filename: &str) {
    let _ = writeln!(io::stderr());
    let _ = writeln!(io::stderr(), "\x1b[1m──── {} ────\x1b[0m", filename);
    let _ = writeln!(io::stderr());
}

/// Cyan `[comment]` line introducing the next step.
pub fn print_step_header(comment: &str) {
    let indent = step_indent();
    let _ = writeln!(io::stderr(), "{indent}\x1b[36m[{}]\x1b[0m", comment);
}

/// Terminal-width separator line of `=` above each step header.
pub fn print_step_separator() {
    let indent = step_indent();
    let width = term_width().unwrap_or(80).saturating_sub(indent.len());
    let _ = writeln!(io::stderr(), "{indent}\x1b[36m{}\x1b[0m", "=".repeat(width));
}

fn term_width() -> Option<usize> {
    std::env::var("COLUMNS").ok().and_then(|s| s.parse().ok())
}

fn step_indent() -> String {
    let depth = STEP_PATH.with(|p| p.borrow().len());
    "   ".repeat(depth.saturating_sub(1))
}

/// Print the dedented, comment-stripped, ignore-stripped source of the step
/// block with syntax highlighting.
///
/// When typewriter mode is enabled (see [`set_typewriter`]), characters
/// appear one-by-one with a configurable delay.
pub fn print_code(code: &str) {
    init_typewriter_defaults();
    if TYPEWRITER.load(Ordering::Relaxed) {
        print_code_typewriter(code);
    } else {
        print_highlighted(code);
    }
}

// ── Typewriter mode ──────────────────────────────────────────────────────

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

static TYPEWRITER: AtomicBool = AtomicBool::new(false);
static TYPEWRITER_SPEED: AtomicU64 = AtomicU64::new(15);
static TYPEWRITER_LINE_PAUSE: AtomicU64 = AtomicU64::new(150);

fn init_typewriter_defaults() {
    use std::sync::OnceLock;
    static INIT: OnceLock<()> = OnceLock::new();
    INIT.get_or_init(|| {
        if option_env!("CODE_STEPS_TYPEWRITER").map_or(false, |v| v == "true") {
            TYPEWRITER.store(true, Ordering::Relaxed);
        }
        if let Some(n) = option_env!("CODE_STEPS_TYPEWRITER_SPEED").and_then(|v| v.parse().ok()) {
            TYPEWRITER_SPEED.store(n, Ordering::Relaxed);
        }
        if let Some(n) =
            option_env!("CODE_STEPS_TYPEWRITER_LINE_PAUSE").and_then(|v| v.parse().ok())
        {
            TYPEWRITER_LINE_PAUSE.store(n, Ordering::Relaxed);
        }
    });
}

/// Enable or disable typewriter display for all subsequent `step!` calls.
pub fn set_typewriter(on: bool) {
    TYPEWRITER.store(on, Ordering::Relaxed);
}

/// Set the typewriter character delay in milliseconds (default 15).
pub fn set_typewriter_speed(ms: u64) {
    TYPEWRITER_SPEED.store(ms, Ordering::Relaxed);
}

/// Set the typewriter line-end pause in milliseconds (default 150).
pub fn set_typewriter_line_pause(ms: u64) {
    TYPEWRITER_LINE_PAUSE.store(ms, Ordering::Relaxed);
}

fn print_code_typewriter(code: &str) {
    use std::io::Write;

    #[cfg(unix)]
    let _raw_guard = RawModeGuard::enter();

    let indent = step_indent();
    let (ss, ts) = highlighting();
    let syntax = ss.find_syntax_by_name("Rust").expect("Rust syntax");
    let theme = &ts.themes[THEME];
    let mut h = HighlightLines::new(syntax, theme);
    let speed = std::time::Duration::from_millis(TYPEWRITER_SPEED.load(Ordering::Relaxed));
    let line_pause =
        std::time::Duration::from_millis(TYPEWRITER_LINE_PAUSE.load(Ordering::Relaxed));

    for line in LinesWithEndings::from(code) {
        let _ = write!(io::stderr(), "{indent}   ");
        let _ = io::stderr().flush();
        let ranges = h.highlight_line(line, ss).unwrap();
        let mut fast_forward = false;
        let mut ri = 0;
        while ri < ranges.len() && !fast_forward {
            let (style_ref, text) = &ranges[ri];
            let style = *style_ref;
            let mut pos = 0;
            for ch in text.chars() {
                if try_read_enter() {
                    fast_forward = true;
                    break;
                }
                let ch_str = ch.to_string();
                let _ = write!(
                    io::stderr(),
                    "{}",
                    syntect::util::as_24_bit_terminal_escaped(&[(style, &ch_str)], false)
                );
                let _ = io::stderr().flush();
                std::thread::sleep(speed);
                pos += ch.len_utf8();
            }
            if fast_forward {
                let _ = write!(
                    io::stderr(),
                    "{}",
                    syntect::util::as_24_bit_terminal_escaped(&[(style, &text[pos..])], false)
                );
            }
            ri += 1;
        }
        if fast_forward {
            while ri < ranges.len() {
                let (style_ref, text) = &ranges[ri];
                let style = *style_ref;
                let _ = write!(
                    io::stderr(),
                    "{}",
                    syntect::util::as_24_bit_terminal_escaped(&[(style, text)], false)
                );
                ri += 1;
            }
            let _ = io::stderr().flush();
        }
        // Line pause — also check for Enter to skip
        if !fast_forward {
            sleep_or_skip(line_pause);
        }
    }
}

/// Non-blocking check: returns `true` if the user pressed Enter.
/// stdin must first be set to non-blocking via `setup_nonblocking_stdin`.
fn try_read_enter() -> bool {
    setup_nonblocking_stdin();
    let mut buf = [0u8; 32];
    loop {
        let n = unsafe { raw::read(0, buf.as_mut_ptr(), buf.len()) };
        if n <= 0 {
            return false;
        }
        if buf[..n as usize].contains(&b'\n') || buf[..n as usize].contains(&b'\r') {
            return true;
        }
    }
}

/// Sleep for `dur`, but return early if Enter is pressed.
fn sleep_or_skip(dur: std::time::Duration) {
    let step = std::time::Duration::from_millis(10);
    let start = std::time::Instant::now();
    while start.elapsed() < dur {
        if try_read_enter() {
            return;
        }
        std::thread::sleep(step.min(dur.saturating_sub(start.elapsed())));
    }
}

// ── Non-blocking stdin (Unix) ───────────────────────────────────────────

#[cfg(unix)]
mod raw {
    use std::os::raw::c_int;
    unsafe extern "C" {
        pub fn fcntl(fd: c_int, cmd: c_int, ...) -> c_int;
        pub fn read(fd: c_int, buf: *mut u8, count: usize) -> isize;
        pub fn tcgetattr(fd: c_int, termios: *mut Termios) -> c_int;
        pub fn tcsetattr(fd: c_int, action: c_int, termios: *const Termios) -> c_int;
    }
    pub const F_GETFL: c_int = 3;
    pub const F_SETFL: c_int = 4;
    #[cfg(target_os = "macos")]
    pub const O_NONBLOCK: c_int = 0x0004;
    #[cfg(not(target_os = "macos"))]
    pub const O_NONBLOCK: c_int = 0x0800;
    pub const TCSANOW: c_int = 0;

    #[repr(C)]
    #[derive(Clone)]
    pub struct Termios {
        pub c_iflag: u64,
        pub c_oflag: u64,
        pub c_cflag: u64,
        pub c_lflag: u64,
        pub c_cc: [u8; 20],
        pub c_ispeed: u64,
        pub c_ospeed: u64,
    }
}

#[cfg(not(unix))]
mod raw {
    pub unsafe fn read(_fd: i32, _buf: *mut u8, _count: usize) -> isize {
        0
    }
}

static STDIN_NONBLOCK: OnceLock<()> = OnceLock::new();

fn setup_nonblocking_stdin() {
    STDIN_NONBLOCK.get_or_init(|| {
        #[cfg(unix)]
        unsafe {
            let flags = raw::fcntl(0, raw::F_GETFL, 0);
            raw::fcntl(0, raw::F_SETFL, flags | raw::O_NONBLOCK);
        }
    });
}

#[cfg(unix)]
fn enter_raw_mode() -> Option<raw::Termios> {
    use std::mem::MaybeUninit;
    unsafe {
        let mut orig: MaybeUninit<raw::Termios> = MaybeUninit::zeroed();
        if raw::tcgetattr(0, orig.as_mut_ptr()) != 0 {
            return None;
        }
        let orig = orig.assume_init();
        let mut raw = orig.clone();
        // Disable canonical mode, echo, and signal chars
        raw.c_lflag &= !(0x0000_0002 | 0x0000_0008 | 0x0000_0001); // ICANON | ECHO | ISIG
        // Read 1 byte at a time, no timeout
        raw.c_cc[6] = 1; // VMIN = 1
        raw.c_cc[5] = 0; // VTIME = 0
        raw::tcsetattr(0, raw::TCSANOW, &raw);
        Some(orig)
    }
}

#[cfg(unix)]
fn leave_raw_mode(orig: raw::Termios) {
    unsafe {
        raw::tcsetattr(0, raw::TCSANOW, &orig);
    }
}

#[cfg(unix)]
struct RawModeGuard {
    orig: raw::Termios,
}

#[cfg(unix)]
impl RawModeGuard {
    fn enter() -> Option<Self> {
        enter_raw_mode().map(|orig| RawModeGuard { orig })
    }
}

#[cfg(unix)]
impl Drop for RawModeGuard {
    fn drop(&mut self) {
        leave_raw_mode(self.orig.clone());
    }
}

/// Green `   ok` marker after the step body has finished executing.
pub fn print_step_done() {
    let _ = writeln!(io::stderr(), "\x1b[32m   ok\x1b[0m\n");
}

// ── Wait / skip / ignore filter ───────────────────────────────────────────
//
// A single global [`WaitFilter`] (stored in a `OnceLock`) controls which
// `wait!()` / `skip!()` / `ignore!()` calls are active at runtime.
//
// Initialised by calling [`init_wait_filter`] once at the start of `main()`.
// If never called, the filter is absent and **all** calls are active
// (backward-compatible default).
//
// CLI format (args after `--`):
//
// ```text
// cargo run --example demo -- --include a,b --exclude c
// ```
//
// Filter logic (applied to the tags of each call):
//
//   include empty? ──yes──→ pass
//          │
//          no: any tag ∈ include? ──yes──→ pass
//          │
//          no ──→ blocked
//
//   THEN
//
//   any tag ∈ exclude? ──yes──→ blocked
//          │
//          no ──→ pass

#[derive(Debug, Default)]
pub struct WaitFilter {
    include: Vec<String>,
    exclude: Vec<String>,
}

/// Global filter instance.  `None` means "not initialised → allow all".
static FILTER: OnceLock<WaitFilter> = OnceLock::new();

/// Parse `--include` and `--exclude` from [`std::env::args`] and store them.
/// Call once at the top of `main()`.
pub fn init_wait_filter() {
    let args: Vec<String> = std::env::args().collect();
    let mut filter = WaitFilter::default();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--include" if i + 1 < args.len() => {
                i += 1;
                filter.include = args[i].split(',').map(|s| s.trim().to_string()).collect();
            }
            "--exclude" if i + 1 < args.len() => {
                i += 1;
                filter.exclude = args[i].split(',').map(|s| s.trim().to_string()).collect();
            }
            _ => {}
        }
        i += 1;
    }

    let _ = FILTER.set(filter);
}

/// Called by the `wait!()` macro expansion and `step!`'s auto-pause.
/// Prints the current nesting path (or a custom message) and waits for
/// Enter if the filter allows.
pub fn press_any_key_if(tags: &[&str], msg: Option<&str>) {
    if let Some(filter) = FILTER.get() {
        if !filter.matches(tags) {
            return;
        }
    }
    let segments: Vec<String> = STEP_PATH.with(|p| p.borrow().clone());
    let indent = step_indent();
    if let Some(msg) = msg {
        let _ = writeln!(io::stderr(), "{indent}\x1b[33m    {msg}\x1b[0m");
    } else if let Some((last, parents)) = segments.split_last() {
        if parents.is_empty() {
            let _ = writeln!(io::stderr(), "{indent}\x1b[32m    {last} waiting\x1b[0m");
        } else {
            let _ = write!(io::stderr(), "{indent}\x1b[33m    {}", parents.join(" : "));
            let _ = write!(io::stderr(), " : ");
            let _ = writeln!(io::stderr(), "\x1b[32m{last} waiting\x1b[0m");
        }
    } else {
        let _ = writeln!(io::stderr(), "{indent}\x1b[33m    ...\x1b[0m");
    }
    // Non-blocking wait for Enter (stdin may be in non-blocking mode
    // after typewriter display).
    setup_nonblocking_stdin();
    let mut byte = [0u8; 1];
    loop {
        let n = unsafe { raw::read(0, byte.as_mut_ptr(), 1) };
        if n == 1 && (byte[0] == b'\n' || byte[0] == b'\r') {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

/// Called by `skip!()` / `step!("…", "tag", …)`.  Returns `true` when the
/// filter considers `tags` active (or hasn't been initialised).
pub fn filter_matches(tags: &[&str]) -> bool {
    FILTER.get().map_or(true, |f| f.matches(tags))
}

impl WaitFilter {
    fn matches(&self, tags: &[&str]) -> bool {
        let passes_include =
            self.include.is_empty() || tags.iter().any(|t| self.include.iter().any(|i| i == t));
        let passes_exclude =
            tags.is_empty() || !tags.iter().any(|t| self.exclude.iter().any(|e| e == t));
        passes_include && passes_exclude
    }
}
