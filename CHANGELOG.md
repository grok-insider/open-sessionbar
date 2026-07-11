# Changelog

All notable, user-facing changes to open-sessionbar are documented here. The
format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this
project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.0.1

Initial public line of the open-sessionbar OpenCode session monitor:

- `opensessions` Rust binary: bar / watch / tui / json formatters for waybar, tmux, and related bars
- Embedded OpenCode TUI plugin (`opencode-sessionbar`) served over HTTP/SSE on localhost
- Animated Working spinner with OpenCode mode colors; multiple spinner styles
- Install helpers and Home-Manager / Nix flake packaging
- Distributed via GitHub Releases (static musl + darwin + windows) and Cachix — not crates.io
