# opencode-sessionbar (plugin)

The OpenCode **TUI plugin** half of [open-sessionbar](../README.md). It runs
inside the OpenCode TUI process and serves live session state over a localhost
HTTP + SSE server so the `opensessions` binary (or any consumer) can render a
status-bar module and a live popup.

You normally **don't install this by hand** — the `opensessions` binary embeds
these files and installs them for you:

```sh
opensessions plugin install
```

That writes this directory to `~/.config/opencode/plugins/opencode-sessionbar/`
and registers `"./plugins/opencode-sessionbar"` in your `tui.json`.

## Server

Binds `127.0.0.1:4098` (override via the `tui.json` options tuple
`["./plugins/opencode-sessionbar", { "port": 4098 }]`):

| Route | Returns |
|-------|---------|
| `GET /health` | `{ ok, name: "opencode-sessionbar" }` (election probe) |
| `GET /sessions` | the snapshot JSON |
| `GET /sessions/stream` | SSE: an `event: snapshot` frame on every change |

See [`../docs/PROTOCOL.md`](../docs/PROTOCOL.md) for the wire format.

## Files

- `store.ts` — snapshot types + helpers
- `snapshot.ts` — builds the snapshot from `api.client.session.*` + `api.state.session.*`
- `server.ts` — `Bun.serve` (with `node:http` fallback): HTTP + SSE
- `tui.tsx` — plugin entry: election, event-driven rebuild, `/sessionbar` command

## Multi-instance

Standard primary/follower election: only one OpenCode instance binds the port;
others detect the peer via `/health` and stay silent. If the primary exits, a
follower takes over within ~30s.
