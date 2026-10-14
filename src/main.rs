//! Like cowsay, but with cute little robots.

mod balloon;
mod bot;
mod cli;
mod rng;

use std::path::Path;
use std::process::ExitCode;

use balloon::Mode;

fn main() -> ExitCode {
    // ╭──────────────────────────────────────────────────╮
    // │ Call me botthink and I'll think instead of talk. │
    // ╰──────────────────────────────────────────────────╯
    //     ○
    //      ○
    //       ▗▄▄▖
    //       ▟▛▜▙
    //       ▜██▛
    //       ▝▌▐▘
    let thinking = std::env::args_os()
        .next()
        .is_some_and(|arg0| Path::new(&arg0).file_stem().is_some_and(|name| name == "botthink"));
    cli::run(if thinking { Mode::Think } else { Mode::Say })
}
