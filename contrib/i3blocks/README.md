# i3blocks

Add a block to your `i3blocks.conf`:

```ini
[sessionbar]
command=opensessions bar --format i3blocks
interval=2
markup=none
```

The command prints three lines: `full_text`, `short_text`, `color`. Waiting
states emit `#ffffff` so they stand out; idle leaves color blank so your theme
wins.

To open the popup, bind a click in i3 instead (i3blocks click handling varies):

```ini
# ~/.config/i3/config
bindsym $mod+s exec --no-startup-id i3-sensible-terminal -e opensessions tui
```
