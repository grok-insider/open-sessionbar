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

Tag `vX.Y.Z` on `master`. CI builds via Nix (pushes to `grok-insider.cachix.org`),
runs `bun test`, and attaches Linux/Windows binaries to the GitHub Release.
