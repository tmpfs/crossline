//! Helper functions for gracefully handling panics.
//!
//! Because we enter terminal raw mode when showing a prompt
//! we need to disable raw mode before exiting the program otherwise
//! the TTY behavior may be incorrect.
//!
//! We do not know which stream you are writing to so call the appropriate
//! function before initializing any prompts to disable raw mode
//! when a panic happens.
//!
use backtrace::Backtrace;
use std::panic::PanicInfo;

use crossterm::{cursor, execute, terminal::disable_raw_mode};

fn handle_panic_hook(info: &PanicInfo) {
    let _ = disable_raw_mode();
    let thread = std::thread::current();
    let thread_name = if let Some(name) = thread.name() {
        name.to_string()
    } else {
        thread.id().as_u64().to_string()
    };
    eprintln!("thread '{}' {}", thread_name, info);
    if let Ok(_) = std::env::var("RUST_BACKTRACE") {
        let backtrace = Backtrace::new();
        eprintln!("{:?}", backtrace);
    } else {
        eprintln!("note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace")
    }
}

#[cfg(any(feature = "panic", doc))]
#[doc(cfg(feature = "panic"))]
/// Set a panic hook writing terminal commands to stdout.
pub fn stdout_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let _ = execute!(std::io::stdout(), cursor::MoveToNextLine(1));
        handle_panic_hook(info);
    }));
}

#[cfg(any(feature = "panic", doc))]
#[doc(cfg(feature = "panic"))]
/// Set a panic hook writing terminal commands to stderr.
pub fn stderr_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let _ = execute!(std::io::stderr(), cursor::MoveToNextLine(1));
        handle_panic_hook(info);
    }));
}
