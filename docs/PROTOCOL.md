# open-sessionbar wire protocol

The OpenCode plugin (`opencode-sessionbar`) runs inside the OpenCode TUI process
and serves session state over a localhost HTTP server. Any consumer — the
`opensessions` binary, a shell script, a custom widget — can read it.

- Host: `127.0.0.1` only (never exposed off-machine).
- Port: `4098` by default. Override via the plugin's `tui.json` options tuple
  `["./plugins/opencode-sessionbar", { "port": 4098 }]`, and tell consumers with
  `--port` or `OPENCODE_SESSIONBAR_PORT`.

## Routes

### `GET /health`
```json
{ "ok": true, "name": "opencode-sessionbar" }
```
Used for liveness and primary/follower election (multiple OpenCode instances).

### `GET /sessions`
Returns the current snapshot (poll this for a status bar):
```jsonc
{
  "summary": {
    "total": 3,
    "busy": 1,
    "waiting": 1,
    "idle": 1,
    "headline": "Waiting for your permission",
    "headlineKind": "permission"          // permission | question | busy | idle
  },
  "sessions": [
    {
      "id": "ses_...",
      "title": "search online about foo",
      "status": "waiting",                 // waiting | busy | done | idle
      "detail": "Waiting for your permission · WebSearch",
      "updated": 1782025564412,            // epoch ms
      "ageLabel": "waiting <1m",
      "mode": "build"                       // agent of latest assistant msg
    }
  ],
  "at": 1782025574506                       // epoch ms snapshot was built
}
```

`summary.mode` is the agent of the session driving the headline (the top busy
one). Each `session.mode` is the agent (`"build"`, `"plan"`, or a custom agent
name) of that session's latest assistant message, read from
`api.state.session.messages(id)`. Consumers map it to OpenCode's agent colors
(dark theme: build `#034cff`, plan `#a753ae`). `mode` may be absent if no
assistant message has run yet.

Headline precedence: `permission > question > busy > idle/done`. Within each
waiting kind, the most recently updated session drives the headline. Sub-sessions
(forked children) are filtered out; stale idle/done sessions (older than 6h) are
dropped; the list is capped at 12 with **active sessions preferred**, then sorted
by status priority and `updated` desc. Summary counts (`busy`/`waiting`/`idle`/
`total`) match the final listed `sessions[]`. `summary.mode` is the agent of the
session driving the headline (when known).

### `GET /sessions/stream`
Server-Sent Events. One frame on connect (priming) and one on every change:
```
event: snapshot
data: {"summary":{...},"sessions":[...],"at":...}

```
Plus periodic `: ping` keep-alive comments (~25s). Consume this for push-based
live updates (the `opensessions tui` popup does).

## Snapshot fields

| Field | Meaning |
|-------|---------|
| `summary.headlineKind` | drives a bar's CSS class / color |
| `summary.headline` | the one-line bar text |
| `session.status` | coarse state for the glyph (★ ● ✓ ○) |
| `session.detail` | human one-liner (tool name on permission, "Done", "Working…") |
| `session.ageLabel` | relative time, e.g. `5m`, `waiting <1m` |
