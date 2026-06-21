# Polybar

Add a custom/script module to your polybar `config.ini`:

```ini
[module/sessionbar]
type = custom/script
exec = opensessions bar --format polybar
interval = 2
click-left = opensessions tui &
```

The output uses polybar markup (`%{F#ffffff}…%{F-}`) to brighten waiting states.
Add `sessionbar` to your `modules-left/center/right`.

For push-based updates instead of polling, use a tailed script:

```ini
[module/sessionbar]
type = custom/script
exec = opensessions watch --format polybar
tail = true
```
