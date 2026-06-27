# Changelog

All notable, user-facing changes to open-sessionbar are documented here. The
format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this
project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.2.1

- Added requirements: OpenCode >= 1.3.14 and a Nerd Font.
- Added nix run, Home-Manager, and prebuilt install paths (with `--locked`).
- Added default `bar` subcommand, `help`/`--version`, and the `plug` alias.
- Added CSS classes for styling.
- Added Troubleshooting and Uninstall sections to documentation.
- Added badges to the README.
- Added macOS prebuilt archives to the install documentation.
- Changed spinner descriptions: braille is single-cell comet default, shimmer is full-cell rotation; documented dots, ring, ring-comet.
- Changed color behavior: busy label/glyph are colored by agent mode (build #034cff, plan #a753ae), glyphs are monochrome.
- Fixed version drift: flake.nix now reads version from Cargo.toml (was pinned 0.1.0); plugin/package.json updated from 0.1.0 to 0.2.0.
- Fixed Windows artifact name in install docs (now `open-sessionbar-<ver>-x86_64-pc-windows-msvc.zip`).

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
