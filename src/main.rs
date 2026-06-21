//! open-sessionbar: desktop-agnostic OpenCode session monitor.
//!
//! Consumes the opencode-sessionbar plugin's localhost server (127.0.0.1:4098)
//! and renders it for any status bar or a live TUI. The plugin itself is
//! embedded in this binary and installed via `opensessions plugin install`.
//!
//! Subcommands:
//!   opensessions bar [--format F]  One-shot status-bar line (default: plain)
//!   opensessions watch [--format F] Stream (SSE) -> repeated lines
//!   opensessions tui               Live fullscreen popup
//!   opensessions json              Raw snapshot JSON (one shot)
//!   opensessions plugin <...>      Manage the embedded OpenCode plugin
//!   opensessions help

mod client;
mod format;
mod install;
mod model;
mod tui;

use std::process::ExitCode;

use client::Client;
use format::Format;

const DEFAULT_PORT: u16 = 4098;

fn reset_sigpipe() {
    // Exit quietly on a closed downstream pipe (e.g. `opensessions json | head`)
    // instead of panicking from the print macros.
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}

fn main() -> ExitCode {
    reset_sigpipe();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().map(String::as_str).unwrap_or("bar");
    let rest = if args.is_empty() { &[][..] } else { &args[1..] };

    match cmd {
        "bar" => cmd_bar(rest),
        "watch" => cmd_watch(rest),
        "tui" => cmd_tui(rest),
        "json" => cmd_json(rest),
        "plugin" | "plug" => match install::cmd(rest) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("error: {e}");
                ExitCode::FAILURE
            }
        },
        "help" | "-h" | "--help" => {
            print_help();
            ExitCode::SUCCESS
        }
        "-v" | "--version" => {
            println!("open-sessionbar {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("unknown command: {other}\n");
            print_help();
            ExitCode::FAILURE
        }
    }
}

fn print_help() {
    println!(
        "open-sessionbar — desktop-agnostic OpenCode session monitor\n\n\
         USAGE:\n\
         \topensessions bar [--format F]    One-shot status-bar line\n\
         \topensessions watch [--format F]  Stream (SSE) -> repeated lines\n\
         \topensessions tui                 Live fullscreen popup\n\
         \topensessions json                Raw snapshot JSON (one shot)\n\
         \topensessions plugin <cmd>        install|update|uninstall|status\n\n\
         FORMATS (for bar/watch): {}\n\
         OPTIONS:\n\
         \t--format F   bar output format (default: plain)\n\
         \t--port N     plugin port (default: {DEFAULT_PORT}; env OPENCODE_SESSIONBAR_PORT)\n\
         \t--global|--project DIR   (plugin) install target",
        Format::all().join(", "),
    )
}

fn parse_port(args: &[String]) -> u16 {
    if let Ok(env) = std::env::var("OPENCODE_SESSIONBAR_PORT") {
        if let Ok(p) = env.parse::<u16>() {
            if p > 0 {
                return p;
            }
        }
    }
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--port" {
            if let Some(v) = args.get(i + 1).and_then(|s| s.parse::<u16>().ok()) {
                if v > 0 {
                    return v;
                }
            }
        }
        i += 1;
    }
    DEFAULT_PORT
}

fn parse_format(args: &[String], default: Format) -> Result<Format, String> {
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--format" || args[i] == "-f" {
            let name = args.get(i + 1).ok_or("--format requires a value")?;
            return Format::parse(name)
                .ok_or_else(|| format!("unknown format '{name}' (try: {})", Format::all().join(", ")));
        }
        i += 1;
    }
    Ok(default)
}

fn cmd_bar(args: &[String]) -> ExitCode {
    let fmt = match parse_format(args, Format::Plain) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    let snap = Client::new(parse_port(args)).snapshot();
    println!("{}", format::render(fmt, snap.as_ref()));
    ExitCode::SUCCESS
}

fn cmd_watch(args: &[String]) -> ExitCode {
    let fmt = match parse_format(args, Format::Plain) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    let port = parse_port(args);
    let client = Client::new(port);
    // Reconnect loop so `watch` survives opencode restarts.
    loop {
        let res = client.stream(|snap| {
            println!("{}", format::render(fmt, Some(&snap)));
            true
        });
        // Print an empty/hidden line on disconnect so a bar clears, then retry.
        println!("{}", format::render(fmt, None));
        if res.is_err() {
            std::thread::sleep(std::time::Duration::from_secs(2));
        } else {
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    }
}

fn cmd_tui(args: &[String]) -> ExitCode {
    match tui::run(parse_port(args)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn cmd_json(args: &[String]) -> ExitCode {
    let snap = Client::new(parse_port(args)).snapshot();
    println!("{}", format::render(Format::Json, snap.as_ref()));
    ExitCode::SUCCESS
}
