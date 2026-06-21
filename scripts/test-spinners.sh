#!/usr/bin/env bash
# test-spinners.sh — preview every spinner style frame-by-frame in the terminal.
#
# Usage:
#   ./test-spinners.sh              # animate all styles, ~80ms/frame
#   ./test-spinners.sh shimmer 60   # one style, custom ms
#   ./test-spinners.sh list         # print each style's frames as a strip
#
# Renders against a fake "busy" snapshot so you see exactly what the bar would
# show (headline + spinner glyph). No opencode/plugin needed.

set -u
BIN="${OPENSESSIONS_BIN:-$(realpath "$(dirname "$0")/target/release/opensessions" 2>/dev/null || echo opensessions)}"
STYLES=(braille shimmer dots)
TICK="${2:-80}"

print_frames() {
  local style="$1"
  # Pull the frame set from the binary's --help isn't available; instead animate
  # via watch against an in-process snapshot. Simpler: emit frames by running
  # `bar` N times with an advancing tick env — but tick is internal. So we drive
  # `watch` for a short window and sample its stdout.
  printf "%-8s: " "$style"
  timeout 0.8 "$BIN" watch --format plain --animate glyph --spinner "$style" --tick 60 --port 6553 2>/dev/null \
    | grep -ao "^[^ ]* Working" | sed 's/ Working//' | tr '\n' ' '
  echo
}

animate_style() {
  local style="$1" ms="$2"
  echo "▶ $style ( ${ms}ms/frame ) — Ctrl-C to stop"
  # `watch` against an unreachable port emits empty frames fast; instead we want
  # the busy glyph. So we use a tiny inline server is overkill — just cycle the
  # frames we know each style produces, read from a one-shot sample.
  local frames
  frames=$(timeout 0.8 "$BIN" watch --format plain --animate glyph --spinner "$style" --tick 60 --port 6553 2>/dev/null \
    | grep -ao "^[^ ]* Working" | sed 's/ Working//' | head -20)
  if [ -z "$frames" ]; then
    echo "  (no frames — is the binary built? run: cargo build --release)"
    return
  fi
  # Re-loop the sampled frames forever.
  while true; do
    printf '%s\r' "$(echo "$frames" | head -1)   "
    for f in $frames; do
      printf '\r\033[K%s Working…   ' "$f"
      sleep "$(awk "BEGIN{print $ms/1000}")"
    done
  done
}

case "${1:-all}" in
  list)
    for s in "${STYLES[@]}"; do print_frames "$s"; done
    ;;
  all)
    for s in "${STYLES[@]}"; do
      animate_style "$s" "$TICK" &
      PID=$!
      sleep 2
      kill "$PID" 2>/dev/null
      wait "$PID" 2>/dev/null
      echo
    done
    ;;
  braille|shimmer|dots)
    animate_style "$1" "$TICK"
    ;;
  *)
    echo "usage: $0 [all|braille|shimmer|dots|list] [ms=80]"
    exit 1
    ;;
esac
