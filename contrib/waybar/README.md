# Waybar

## Animated spinner (recommended) — continuous `watch`

For an OpenCode-style animated "Working" spinner, run `watch` (continuous
stdout, no `interval`):

```jsonc
"custom/sessionbar": {
  "exec": "opensessions watch --format waybar --animate glyph --spinner braille",
  "return-type": "json",
  "max-length": 40,
  "tooltip": true,
  "on-click": "ghostty -e opensessions tui"
}
```

- `--animate glyph` prefixes a spinner frame to the text while a session is busy.
- `--spinner`:
  - `braille` — single-cell 3-dot comet (recommended; **gapless**).
  - `dots` — single-cell dot rotation.
  - `shimmer` — filled cell with a rotating gap.
  - `ring` / `ring-comet` — a dot/comet orbits a hollow "0". Wider and more
    "ring-like", but braille renders the two cells with a small seam, so there's
    a slight gap at the top/bottom edges. Use `braille` if that bothers you.
- `--tick 100` sets the frame interval in ms (default 100).

The module never blanks on a brief plugin reconnect — `watch` keeps the last
state and only clears after the plugin is unreachable for ~6s.

Note: a waybar text module has ONE color, so trailing comet dots cannot be
greyed independently (braille glyphs are monochrome). The comet shape conveys
the motion; per-dot dimming is only possible in the `opensessions tui` popup.

## Lightweight alternative — polling + CSS pulse

If you'd rather poll and animate via CSS opacity instead of a long-running
process:

```jsonc
"custom/sessionbar": {
  "exec": "opensessions bar --format waybar --animate pulse",
  "return-type": "json",
  "interval": 2,
  "max-length": 40,
  "tooltip": true,
  "on-click": "ghostty -e opensessions tui"
}
```

## Styling

Add `"custom/sessionbar"` to `modules-left/center/right`, then style it.
Classes: `permission`, `question`, `busy`, `idle`, `empty`, plus the agent mode
(`build`, `plan`, …) and `pulse` when busy. Mode colors match OpenCode's dark
theme (build `#034cff`, plan `#a753ae`):

```css
/* Note: GTK CSS keyframes do NOT accept comma-separated selectors like
   "0%, 100%". Use separate from/50%/to blocks. */
@keyframes sessionbar-pulse {
  from { opacity: 1; }
  50%  { opacity: 0.45; }
  to   { opacity: 1; }
}
#custom-sessionbar.busy.pulse { animation: sessionbar-pulse 1.2s ease-in-out infinite; }

/* Color the label (and the spinner glyph, which is part of the text) by mode */
#custom-sessionbar.busy.build { color: #034cff; }  /* OpenCode build = blue */
#custom-sessionbar.busy.plan  { color: #a753ae; }  /* OpenCode plan  = purple */

#custom-sessionbar.permission,
#custom-sessionbar.question { color: #ffffff; font-weight: 700; }
#custom-sessionbar.busy     { color: #ffffff; }    /* busy w/ unknown mode */
#custom-sessionbar.idle     { color: #888888; }
#custom-sessionbar.empty    { padding: 0; }        /* collapse when no sessions */
```

The tooltip (hover) shows the full per-session list (with `[mode]` tags) as
Pango markup. Click opens the live `opensessions tui` popup (swap `ghostty` for
your terminal).
