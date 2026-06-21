# open-sessionbar — agent notes

Desktop-agnostic OpenCode session monitor. Two halves in one repo:

- **`src/`** — the `opensessions` Rust binary (consumer + formatter + installer).
- **`plugin/`** — the `opencode-sessionbar` OpenCode TUI plugin (producer),
  embedded into the binary via `include_str!` in `src/install.rs`.

## Architecture

```
OpenCode TUI ── plugin/ (Bun) ── serves 127.0.0.1:4098 ──HTTP/SSE──> opensessions (Rust)
                 /sessions, /sessions/stream, /health                  bar | watch | tui | json
```

The plugin reads `api.client.session.list()/status()` +
`api.state.session.permission/question/todo` and serves a snapshot. The binary
polls `/sessions` (bar) or subscribes to `/sessions/stream` (tui/watch) and
formats it. Wire format: `docs/PROTOCOL.md`.

## Source of truth & embedding

`plugin/` is the canonical plugin. `src/install.rs` embeds the five files
(`package.json`, `store.ts`, `snapshot.ts`, `server.ts`, `tui.tsx`) with
`include_str!("../plugin/...")`. **If you edit `plugin/`, the binary picks it up
on the next `cargo build`** — keep the file list in `install.rs::FILES` in sync
if you add/remove plugin files.

## Plugin file roles (plugin/)

| File | Role |
|------|------|
| `store.ts` | snapshot types + helpers (`timeAgo`, `truncate`) |
| `snapshot.ts` | builds the snapshot; headline precedence `permission > question > busy > idle`; filters sub-sessions + stale (>6h) + caps 12 |
| `server.ts` | `Bun.serve` (+ `node:http` fallback): `/health`, `/sessions`, `/sessions/stream` (SSE) |
| `tui.tsx` | plugin entry: port 4098, election, event-driven rebuild, `/sessionbar` command |

## Rust module roles (src/)

| File | Role |
|------|------|
| `model.rs` | serde snapshot types (camelCase rename) |
| `client.rs` | blocking HTTP `/sessions` + `/health`, SSE `/sessions/stream` reader |
| `format/*.rs` | one formatter per bar (`waybar`, `i3blocks`, `polybar`, `eww`, `plain`, `json`) |
| `tui.rs` | ratatui live popup; SSE in a thread → channel → redraw |
| `install.rs` | embedded plugin + `tui.json` patch (strict-JSON, `.bak`, JSONC-abort) |
| `main.rs` | `std::env::args` dispatch (open-usage idiom, no clap) |

## Conventions

- Keep comments factual: API fields, routes, ports, file roles. No policy framing.
- Plugin server: **127.0.0.1 only**, `GET /health` → `{ok,name}` for election.
- Port default **4098** (the OpenCode plugin port convention; `opencode-notify`
  owns 4097). Override via `tui.json` options tuple / `--port` /
  `OPENCODE_SESSIONBAR_PORT`.
- No npm: the binary is the plugin's distribution channel.

## Build & test

```sh
cargo build && cargo test          # app (5 integration tests in tests/format.rs)
cd plugin && bun test              # plugin (server + snapshot, 7 tests)
nix build .# && nix flake check    # nix package
```

## Packaging

Flake mirrors `open-usage`: `packages.default` (rustPlatform),
`apps.default`, `homeManagerModules.default` (`programs.open-sessionbar`:
`enable`, `package`, `port`, `opencodePlugin.enable`), `checks`, `devShells`,
cachix `nixConfig` (`0xfell.cachix.org`). CI pushes to cachix + attaches release
binaries (Linux/Windows) on `v*` tags. Published as `github:0xfell/open-sessionbar`.

## CONTRIBUTING

See `CONTRIBUTING.md`.
