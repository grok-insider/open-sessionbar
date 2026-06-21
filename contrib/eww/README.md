# eww (ElKowar's Wacky Widgets)

`opensessions bar --format eww` (and `watch --format eww`) emit JSON, so a
`deflisten` can destructure fields directly:

```lisp
(deflisten sessionbar
  :initial "{\"text\":\"\",\"class\":\"empty\",\"total\":0,\"waiting\":0,\"busy\":0}"
  "opensessions watch --format eww")

(defwidget sessionbar []
  (button :onclick "opensessions tui &"
          :class {sessionbar.class}
    (label :text {sessionbar.text})))
```

Fields: `text`, `class` (permission/question/busy/idle/empty), `headline`,
`total`, `waiting`, `busy`.

Use `watch` (above) for push updates, or `bar --format eww` with `defpoll` for
polling.
