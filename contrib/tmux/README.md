# tmux

Drop the session line into your tmux status bar via a `#(...)` shell command in
`~/.tmux.conf`:

```tmux
set -g status-interval 2
set -g status-right-length 60
set -g status-right '#(opensessions bar --format tmux)'
```

The output uses tmux style tags (`#[fg=#034cff]…#[default]`) to color the
"Working" label by the OpenCode agent mode — **build = `#034cff`** (blue),
**plan = `#a753ae`** (purple) — and bold white for waiting/permission states.
Any literal `#` in the text is doubled (`##`) so tmux renders it instead of
treating it as a format expansion.

tmux re-runs the command every `status-interval` seconds (default 15; set it
lower for snappier updates). The segment collapses to nothing when no sessions
are active or the plugin is unreachable.

Open the live popup with a key binding (tmux 3.2+):

```tmux
bind-key S display-popup -E -w 80% -h 60% 'opensessions tui'
```

Note: tmux polls (it doesn't stream stdout line-by-line), so the
`--animate glyph` spinner frame won't advance between refreshes — the
color-by-mode label is the recommended tmux presentation. For a continuously
animated spinner, use a streaming bar such as waybar with `watch`.
