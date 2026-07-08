# Changelog

All notable, user-facing changes to open-sessionbar are documented here. The
format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this
project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2026-07-08

- Added `--port` CLI flag to override the session port; the plugin now reads `OPENCODE_SESSIONBAR_PORT` environment variable and Home-Manager always exports the port.
- Added `plug` alias for the `bar` subcommand, CSS classes for styling, and `--help`/`--version` flags.
- Added Requirements section (OpenCode >= 1.3.14, Nerd font) and expanded install paths including `nix run`, Home-Manager, and prebuilt archives with `--locked`.
- Added Troubleshooting and Uninstall sections to documentation.
- Changed snapshot logic to prefer most-recent waiters and active sessions when capping the snapshot, aligning summary counts and headline mode with the listed set.
- Fixed version drift: `flake.nix` now reads version from `Cargo.toml` and `plugin/package.json` updated from 0.1.0 to 0.2.0.
- Fixed documentation: corrected spinner descriptions (braille is single-cell comet default, shimmer is full-cell rotation; documented dots/ring/ring-comet), decoupled color from spinner (busy label/glyph colored by agent mode, glyphs monochrome), and corrected Windows artifact name and added macOS prebuilt archives.
- Improved Home-Manager integration: warns on plugin install failure.

## 0.2.0

- Added tmux status-line formatter with mode-colored Working label and proper escaping.
- Added animated Working spinner with OpenCode mode colors (build/plan/custom).
- Added new spinner styles: ring and ring-comet.
- Added `--animate`, `--spinner`, and `--tick` command-line options to control animation.
- Changed default spinner to a gapless single-cell braille comet.
- Improved spinner animation to be gapless across braille cell seams.
- Fixed module disappearing on transient reconnect by holding the last snapshot during brief SSE drops.
- Fixed stdout buffering so piped consumers (waybar) receive animation frames in real time.
- Fixed top-edge gap in ring spinner by using full 4-column braille cells.

## 0.1.0

- Initial release: opencode session status bar serving live session state over HTTP/SSE for tmux and waybar.
