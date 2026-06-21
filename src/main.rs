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
mod spinner;
mod tui;

use std::process::ExitCode;

use client::Client;
use format::Format;
use spinner::{Anim, AnimateMode, SpinnerStyle};

const DEFAULT_PORT: u16 = 4098;
const DEFAULT_TICK_MS: u64 = 100;

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
         \t--format F     bar output format (default: plain)\n\
         \t--animate M    off|glyph|pulse (default: off). glyph needs `watch`.\n\
         \t--spinner S    braille|shimmer|dots|ring|ring-comet (default: braille)\n\
         \t--tick MS      glyph frame interval under watch (default: {DEFAULT_TICK_MS})\n\
         \t--port N       plugin port (default: {DEFAULT_PORT}; env OPENCODE_SESSIONBAR_PORT)\n\
         \t--global|--project DIR   (plugin) install target",
        Format::all().join(", "),
    )
}

fn parse_anim(args: &[String]) -> Result<Anim, String> {
    let mut anim = Anim::default();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--animate" | "-a" => {
                let v = args.get(i + 1).ok_or("--animate requires off|glyph|pulse")?;
                anim.mode = AnimateMode::parse(v)
                    .ok_or_else(|| format!("unknown --animate '{v}' (off|glyph|pulse)"))?;
            }
            "--spinner" => {
                let v = args.get(i + 1).ok_or("--spinner requires braille|shimmer")?;
                anim.spinner = SpinnerStyle::parse(v)
                    .ok_or_else(|| format!("unknown --spinner '{v}' (braille|shimmer)"))?;
            }
            _ => {}
        }
        i += 1;
    }
    Ok(anim)
}

fn parse_tick(args: &[String]) -> u64 {
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--tick" {
            if let Some(v) = args.get(i + 1).and_then(|s| s.parse::<u64>().ok()) {
                if v >= 16 {
                    return v;
                }
            }
        }
        i += 1;
    }
    DEFAULT_TICK_MS
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
    let anim = match parse_anim(args) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    let snap = Client::new(parse_port(args)).snapshot();
    println!("{}", format::render(fmt, snap.as_ref(), anim));
    ExitCode::SUCCESS
}

fn cmd_watch(args: &[String]) -> ExitCode {
    use std::sync::mpsc::{self, RecvTimeoutError};
    use std::time::Duration;

    let fmt = match parse_format(args, Format::Plain) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut anim = match parse_anim(args) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    let port = parse_port(args);
    let tick_ms = parse_tick(args);
    let animate_glyph = anim.mode == AnimateMode::Glyph;

    // Background thread keeps the SSE stream open (reconnecting), forwarding
    // each snapshot to the render loop via a channel.
    let (tx, rx) = mpsc::channel::<Option<model::Snapshot>>();
    std::thread::spawn(move || {
        let client = Client::new(port);
        loop {
            let tx2 = tx.clone();
            let res = client.stream(move |snap| tx2.send(Some(snap)).is_ok());
            if tx.send(None).is_err() {
                return; // render loop gone
            }
            let _ = res;
            std::thread::sleep(Duration::from_secs(2));
        }
    });

    // Render loop: re-emit on each snapshot, and (when animating a glyph and a
    // session is busy) on a frame timer so the spinner advances smoothly.
    //
    // CRITICAL: stdout is block-buffered when piped (waybar pipes the exec), so
    // each line must be flushed immediately or waybar sees frames in bursts and
    // the spinner looks frozen. We write + flush explicitly.
    use std::io::Write;
    let mut out = std::io::stdout();
    let emit = |out: &mut std::io::Stdout, line: String| {
        let _ = writeln!(out, "{line}");
        let _ = out.flush();
    };

    let mut last: Option<model::Snapshot> = None;
    let frame_dt = Duration::from_millis(tick_ms);
    loop {
        let busy = last.as_ref().map(format::is_busy_opt).unwrap_or(false);
        let timeout = if animate_glyph && busy { frame_dt } else { Duration::from_secs(3600) };
        match rx.recv_timeout(timeout) {
            Ok(snap) => {
                last = snap;
                anim.tick = 0;
                emit(&mut out, format::render(fmt, last.as_ref(), anim));
            }
            Err(RecvTimeoutError::Timeout) => {
                // Frame tick: advance the spinner and re-render the same snapshot.
                anim.tick = anim.tick.wrapping_add(1);
                emit(&mut out, format::render(fmt, last.as_ref(), anim));
            }
            Err(RecvTimeoutError::Disconnected) => return ExitCode::SUCCESS,
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
    println!("{}", format::render(Format::Json, snap.as_ref(), Anim::default()));
    ExitCode::SUCCESS
}
