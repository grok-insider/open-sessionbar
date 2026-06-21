# Waybar

Add a `custom/sessionbar` module to your waybar `config`:

```jsonc
"custom/sessionbar": {
  "exec": "opensessions bar --format waybar",
  "return-type": "json",
  "interval": 2,
  "max-length": 40,
  "tooltip": true,
  "on-click": "ghostty -e opensessions tui"
}
```

Add `"custom/sessionbar"` to `modules-left/center/right`, then style it in
`style.css` (state classes: `permission`, `question`, `busy`, `idle`, `empty`):

```css
#custom-sessionbar.permission,
#custom-sessionbar.question { color: #ffffff; font-weight: 700; }
#custom-sessionbar.busy     { color: #ffffff; }
#custom-sessionbar.idle     { color: #888888; }
#custom-sessionbar.empty    { padding: 0; }   /* collapse when no sessions */
```

The tooltip (hover) shows the full per-session list as Pango markup. Click opens
the live `opensessions tui` popup (swap `ghostty` for your terminal).
