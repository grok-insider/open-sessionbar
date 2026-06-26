# open-sessionbar

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
  or plain text) plus a live fullscreen TUI popup. Runs on Linux, macOS, Windows.
- **`opencode-sessionbar`** — the OpenCode TUI plugin that serves session state
  over `127.0.0.1:4098` (HTTP + SSE). It's **embedded in the binary** — install
  it with `opensessions plugin install`. No npm.

## Quick start

```sh
# 1. install the app (NixOS: flake module; else cargo / release binary)
cargo install --git https://github.com/grok-insider/open-sessionbar

# 2. install the OpenCode plugin (drops files + registers in tui.json)
opensessions plugin install
#    …restart opencode…

# 3. wire it into your bar
opensessions bar --format waybar
```

See [`docs/INSTALL.md`](docs/INSTALL.md) for NixOS/Home-Manager, Windows, and
per-bar setup, and [`contrib/`](contrib/) for copy-paste bar snippets.

## Commands

| Command | What |
|---------|------|
| `opensessions bar --format <F>` | one-shot status-bar line (`waybar`, `i3blocks`, `polybar`, `eww`, `plain`, `json`) |
| `opensessions watch --format <F>` | stream (SSE) → one line per change (use for animated spinner) |
| `opensessions tui` | live fullscreen popup |
| `opensessions json` | raw snapshot JSON |
| `opensessions plugin install\|update\|uninstall\|status` | manage the embedded OpenCode plugin |

Options:
- `--port N` / `OPENCODE_SESSIONBAR_PORT` (default 4098)
- `--animate off\|glyph\|pulse` (default off). `glyph` animates the bar text
  (needs `watch`); `pulse` emits a CSS class to opacity-pulse (works with `bar`).
- `--spinner braille\|shimmer\|dots\|ring\|ring-comet` (default braille) — frame
  set for `glyph`. `ring`/`ring-comet` orbit a dot around a hollow "0".
- `--tick MS` (default 100) — glyph frame interval under `watch`.
- `--global`/`--project DIR` (plugin install target).

### Animation & colors

While a session is busy, the bar shows a "Working" headline. With
`--animate glyph` it gets an OpenCode-style spinner (`shimmer` = amber dot
gradient, `braille` = `⠋⠙⠹…`). The label (and spinner glyph) is colored by the
session's **agent mode**, matching OpenCode's dark theme: **build = `#034cff`**
(blue), **plan = `#a753ae`** (purple). Status-bar formats emit the mode as a CSS
class (`busy build`/`busy plan`) and/or a resolved hex; see
[`contrib/`](contrib/) for per-bar setup.

## How it works

The plugin runs inside the OpenCode TUI, reads live session state
(`api.client.session.*` + `api.state.session.permission/question/todo`), and
serves a compact snapshot on `127.0.0.1:4098`:

- `GET /sessions` — snapshot (the bar polls this)
- `GET /sessions/stream` — SSE push (the TUI popup subscribes)
- `GET /health` — liveness + primary/follower election

The `opensessions` binary is a pure consumer + formatter, so it works under any
desktop or none. Wire format → [`docs/PROTOCOL.md`](docs/PROTOCOL.md).

Multiple OpenCode instances are handled by primary/follower election: one binds
the port, the rest stay silent; if the primary exits a follower takes over.

## Build

```sh
cargo build && cargo test        # app
cd plugin && bun test            # plugin
nix build .# && nix flake check  # nix
```

## License

MIT © Grok Insider
