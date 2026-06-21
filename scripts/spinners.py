#!/usr/bin/env python3
"""Preview/generate open-sessionbar spinner frame sets in the terminal.

Usage:
  scripts/spinners.py            # animate every spinner side by side
  scripts/spinners.py ring       # animate just one
  scripts/spinners.py --list     # print frame arrays (copy into src/spinner.rs)
  scripts/spinners.py --grid ring  # show the ring's dot grid as ASCII

The frame sets here MUST match src/spinner.rs. `--list` regenerates the Rust
arrays so the two never drift.
"""
import sys
import time

# Braille dot bit values within a 2-col x 4-row cell.
BRAILLE = 0x2800
DOTBIT = {
    (0, 0): 0x01, (0, 1): 0x02, (0, 2): 0x04, (0, 3): 0x40,
    (1, 0): 0x08, (1, 1): 0x10, (1, 2): 0x20, (1, 3): 0x80,
}


def cell(dots):
    v = 0
    for d in dots:
        v |= DOTBIT[d]
    return chr(BRAILLE + v)


def render_grid(lit):
    """Render a set of (col,row) over cols 0..3 as adjacent braille cells."""
    left = [(c, r) for (c, r) in lit if c < 2]
    right = [(c - 2, r) for (c, r) in lit if c >= 2]
    return cell(left) + cell(right)


# Hollow-0 perimeter over a 4-wide x 4-tall grid (center 2x2 empty), clockwise
# from top-left. Full 4 columns keep the top/bottom edges continuous across the
# braille cell seam (a 3-wide ring left a gap at the top-middle).
RING_PATH = [
    (0, 0), (1, 0), (2, 0), (3, 0),  # top
    (3, 1), (3, 2), (3, 3),          # right + bottom-right
    (2, 3), (1, 3), (0, 3),          # bottom
    (0, 2), (0, 1),                  # left
]


def ring_frames():
    return [render_grid([p]) for p in RING_PATH]


def ring_comet_frames(trail=4):
    n = len(RING_PATH)
    return [render_grid([RING_PATH[(i - k) % n] for k in range(trail)]) for i in range(n)]


SPINNERS = {
    # single-cell 3-dot comet orbit — gapless (no inter-cell seam)
    "braille": ["⠇", "⠋", "⠙", "⠸", "⢰", "⣠", "⣄", "⡆"],
    # filled cell with a rotating gap
    "shimmer": ["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"],
    # 2-3 dot arc rotating
    "dots": ["⠋", "⠙", "⠚", "⠞", "⠖", "⠦", "⠴", "⠲", "⠳", "⠓"],
    # hollow "0": a dot orbiting a 3x4 ring (center empty)
    "ring": ring_frames(),
    # same ring, 3-dot comet for more visibility
    "ring-comet": ring_comet_frames(),
}


def list_rust():
    def arr(fs):
        return ", ".join(f'"{f}"' for f in fs)
    print("// paste into src/spinner.rs frames()/ring_frames()")
    for name, fs in SPINNERS.items():
        print(f"{name}: &[{arr(fs)}]")
    print(f"ring outline (all perimeter): {render_grid(RING_PATH)}")


def show_grid(name):
    if name != "ring":
        print(f"(grid view only meaningful for 'ring'); frames: {' '.join(SPINNERS[name])}")
        return
    lit = set(RING_PATH)
    print("ring is a 3-wide x 4-tall hollow 0 (perimeter lit, center empty):")
    for r in range(4):
        print("  " + "".join("#" if (c, r) in lit else "." for c in range(3)))
    print("full outline:", render_grid(RING_PATH))


def dimmed_ring_frame(i):
    """Ring comet with a colored fade: head bright white, tail dim grey.
    Truecolor only (terminals/TUI). Renders the head layer and the tail layer
    as two separately-colored braille pairs are not mergeable per-cell, so we
    color the whole comet glyph and step brightness over time for a pulse-fade.
    Here we approximate: bright head glyph + a dim full-ring backdrop."""
    n = len(RING_PATH)
    head = [RING_PATH[(i - k) % n] for k in range(2)]      # 2 bright dots
    tail = [RING_PATH[(i - k) % n] for k in range(2, 4)]   # 2 dim dots
    # Backdrop: full ring in very dim grey so the "0" is always visible.
    backdrop = render_grid(RING_PATH)
    bright = render_grid(head)
    dim = render_grid(tail)
    # Layer via ANSI: dim ring backdrop, then we just show bright head + grey tail
    # side-by-side is wrong (width); instead overlay isn't possible in mono, so
    # we present: [dim full ring] with brightness pulsing — plus a separate
    # bright/grey two-glyph readout for clarity.
    return (
        f"\x1b[38;5;240m{backdrop}\x1b[0m"          # always-on dim "0"
        f"  head:\x1b[97m{bright}\x1b[0m"
        f" tail:\x1b[38;5;244m{dim}\x1b[0m"
    )


def animate(names, fps=10, seconds=6):
    delay = 1.0 / fps
    width = max(len(n) for n in names)
    ticks = int(seconds * fps)
    try:
        print("\x1b[?25l", end="")  # hide cursor
        for t in range(ticks):
            parts = []
            for n in names:
                if n == "ring-fade":
                    parts.append(f"{n:>{width}} {dimmed_ring_frame(t % len(RING_PATH))}")
                else:
                    parts.append(f"{n:>{width}} {SPINNERS[n][t % len(SPINNERS[n])]} Working")
            print("\r" + "   ".join(parts) + "   ", end="", flush=True)
            time.sleep(delay)
        print()
    finally:
        print("\x1b[?25h", end="")  # show cursor


def main():
    args = sys.argv[1:]
    if "--list" in args:
        list_rust()
        return
    if "--grid" in args:
        i = args.index("--grid")
        show_grid(args[i + 1] if i + 1 < len(args) else "ring")
        return
    selectable = list(SPINNERS.keys()) + ["ring-fade"]
    names = [a for a in args if a in selectable] or selectable
    bad = [a for a in args if a not in selectable and not a.startswith("--")]
    if bad:
        print(f"unknown spinner(s): {bad}; choices: {list(SPINNERS)}")
        return
    print(f"animating: {', '.join(names)}  (Ctrl-C to stop)\n")
    try:
        animate(names)
    except KeyboardInterrupt:
        print("\x1b[?25h")


if __name__ == "__main__":
    main()
