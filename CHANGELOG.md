# Changelog

All notable, user-facing changes to open-sessionbar are documented here. The
format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this
project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1](https://github.com/grok-insider/open-sessionbar/compare/v0.2.0...v0.2.1) - 2026-06-27

### Other

- fix spinner/color docs, expand install, sync version drift ([#4](https://github.com/grok-insider/open-sessionbar/pull/4))

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
