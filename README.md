# open-sessionbar

[![CI](https://github.com/grok-insider/open-sessionbar/actions/workflows/ci.yml/badge.svg)](https://github.com/grok-insider/open-sessionbar/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/grok-insider/open-sessionbar?sort=semver)](https://github.com/grok-insider/open-sessionbar/releases/latest)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A desktop-environment-agnostic monitor for your [OpenCode](https://opencode.ai)
sessions. See — at a glance, from your status bar — which sessions are working,
which are done, and which are **waiting for your permission**, then pop open a
live list with one click.

```
★ search online about foo      Waiting for your permission · WebSearch   waiting <1m
● Build the login feature       Working…                                 13m
✓ do the xp test                Done                                     54m
```

Two pieces, one repo:

- **`opensessions`** — a single Rust binary. Talks to the plugin over localhost
  and renders your sessions for any status bar (waybar, i3blocks, polybar, eww,
  tmux, or plain text) plus a live fullscreen TUI popup. Runs on Linux, macOS,
  Windows.
- **`opencode-sessionbar`** — the OpenCode TUI plugin that serves session state
  over `127.0.0.1:4098` (HTTP + SSE). It's **embedded in the binary** — install
  it with `opensessions plugin install`. No npm.

## Requirements

- **OpenCode ≥ 1.3.14** (for the plugin).
- A **Nerd/braille-capable font** in your bar/terminal for the status glyphs
  (★ ● ✓ ○) and the animated spinner.

## Install

```sh
# cargo (any platform)
cargo install --git https://github.com/grok-insider/open-sessionbar --locked

# Nix — run without installing
nix run github:grok-insider/open-sessionbar -- bar --format waybar
```

**Prebuilt binaries:** each [Release](https://github.com/grok-insider/open-sessionbar/releases)
attaches `opensessions` for Linux (x86_64/aarch64 musl), macOS (x86_64/arm64), and
Windows (x86_64) — `open-sessionbar-<version>-<target>.tar.gz` (`.zip` on Windows),
each with a `.sha256`.

**Nix / Home Manager:**

```nix
imports = [ inputs.open-sessionbar.homeManagerModules.default ];
programs.open-sessionbar = {
  enable = true;
  # port = 4098;                 # OPENCODE_SESSIONBAR_PORT
  opencodePlugin.enable = true;  # also drop + register the OpenCode plugin
};
```

Then install the plugin (unless Home Manager did it) and wire your bar:

```sh
opensessions plugin install   # writes files + registers in tui.json; restart opencode
opensessions bar --format waybar
```

See [`docs/INSTALL.md`](docs/INSTALL.md) for per-platform detail and
[`contrib/`](contrib/) for copy-paste bar snippets.

## Commands

`opensessions` with no subcommand defaults to `bar`.

| Command | What |
|---------|------|
| `opensessions bar [--format F]` | one-shot status-bar line (`waybar`, `i3blocks`, `polybar`, `eww`, `tmux`, `plain`, `json`) |
| `opensessions watch [--format F]` | stream (SSE) → one line per change; also re-emits on the spinner frame timer under `--animate glyph` |
| `opensessions tui` | live fullscreen popup |
| `opensessions json` | raw snapshot JSON |
| `opensessions plugin install\|update\|uninstall\|status` | manage the embedded OpenCode plugin (`plug` is an alias) |
| `opensessions help` · `--version` | usage / version |

Options:
- `--format F` — bar output format (default `plain`).
- `--animate off\|glyph\|pulse` / `-a` (default off). `glyph` animates the bar text
  (needs `watch`); `pulse` emits a CSS class to opacity-pulse (works with `bar`).
- `--spinner braille\|shimmer\|dots\|ring\|ring-comet` (default braille) — frame set
  for `glyph`.
- `--tick MS` (default 100) — glyph frame interval under `watch`.
- `--port N` / `OPENCODE_SESSIONBAR_PORT` (default 4098).
- `--global` / `--project DIR` — plugin install target.

### Animation & colors

While a session is busy the bar shows a "Working" headline; with `--animate glyph`
it gets an OpenCode-style spinner. The styles (all monochrome — color comes from
the mode, below):

- `braille` (default) — a gapless single-cell 3-dot comet.
- `shimmer` — a full-cell rotation (the classic smooth "loading" spinner).
- `dots` — two opposite orbiting dots.
- `ring` / `ring-comet` — a dot / 3-dot comet orbiting a hollow "0".

The busy label and glyph are colored by the session's **agent mode**, matching
OpenCode's dark theme: **build = `#034cff`** (blue), **plan = `#a753ae`** (purple).
Status-bar formats expose the headline state and mode as CSS classes
(`permission` / `question` / `busy build` / `busy plan` / `idle`, plus `pulse`)
and/or a resolved hex; see [`contrib/`](contrib/) for per-bar styling.

## How it works

The plugin runs inside the OpenCode TUI, reads live session state
(`api.client.session.list()/status()` + `api.state.session.permission/question/todo`,
and the agent mode from the latest message), and serves a compact snapshot on
`127.0.0.1:4098`:

- `GET /sessions` — snapshot (the bar polls this)
- `GET /sessions/stream` — SSE push (the TUI popup / `watch` subscribe)
- `GET /health` — liveness + primary/follower election

The `opensessions` binary is a pure consumer + formatter, so it works under any
desktop or none. Wire format → [`docs/PROTOCOL.md`](docs/PROTOCOL.md).

Multiple OpenCode instances are handled by primary/follower election: one binds
the port, the rest stay silent; if the primary exits a follower takes over.

## Troubleshooting

- **Bar is blank** — is OpenCode running with the plugin installed? Check
  `opensessions plugin status` (installed / registered / server live). On a brief
  SSE drop the last snapshot is held for a few seconds rather than blanking.
- **Port already in use** — that's the primary/follower election working; another
  OpenCode instance owns `:4098`. Override with `--port` /
  `OPENCODE_SESSIONBAR_PORT` if it collides with something else.
- **Remove it** — `opensessions plugin uninstall` (un-registers from `tui.json`
  and deletes the plugin files).

## Build

```sh
cargo build && cargo test        # app
cd plugin && bun test            # plugin
nix build .# && nix flake check  # nix
```

## License

MIT © Grok Insider
