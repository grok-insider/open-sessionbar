# Contributing

## Layout

- `src/` — the `opensessions` Rust binary.
- `plugin/` — the OpenCode TUI plugin (TypeScript/Bun), embedded into the binary.
- `tests/` — Rust integration tests; `plugin/tests/` — Bun tests.
- `contrib/` — copy-paste bar snippets; `docs/` — protocol + install.

## Develop

```sh
cargo build && cargo test
cd plugin && bun test
nix build .# && nix flake check
```

The plugin is embedded via `include_str!` in `src/install.rs`. After editing
`plugin/`, rebuild the binary to embed the changes. If you add/remove a plugin
file, update `FILES` in `src/install.rs`.

## Testing the plugin live

Run the plugin's server standalone and point the binary at it:

```sh
# a tiny harness that serves a sample snapshot on :4098 (see tests for shape)
opensessions bar --format waybar
opensessions tui
```

Or install into OpenCode and run it for real:

```sh
opensessions plugin install
# restart opencode; opensessions plugin status  -> server: live
```

## Style

- Rust: `cargo fmt`, `cargo clippy`. No `unwrap()` on external I/O.
- Comments are factual (fields, routes, ports, file roles).
- Keep the server **127.0.0.1-only** and the JSON shapes in `model.rs` and
  `plugin/store.ts` in sync (see `docs/PROTOCOL.md`).

## Releases

Patch releases are automatic; major/minor are manual (same model as open-recorder):

| Stream | How |
|--------|-----|
| **Patch** (`x.y.z → x.y.z+1`) | On push to `master` with `feat`/`fix` commits since the last tag, CI opens a release PR. Merge it to tag + GitHub Release + attach binaries. |
| **Minor / major** | Actions → **Manual Version Bump** (repo admin only). Opens a release PR for `x.y.0` / `x.0.0`. |
| **Force patch PR** | Actions → Release → `workflow_dispatch` with `force` (no feat/fix needed). |

`release-plz` only cuts the `vX.Y.Z` tag + GitHub Release (`release_always`).
Version bumps live in the hand-rolled release PR. Nothing is published to crates.io.
CI also builds via Nix (cachix `grok-insider`) and runs `bun test`.
