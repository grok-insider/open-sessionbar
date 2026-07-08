/** @jsxImportSource @opentui/solid */
// opencode-sessionbar - serves live session state to waybar.
//
// Exposes the "SessionBar" list (titles, status, time-ago, waiting-for-
// permission) over localhost so a waybar custom module can render it:
//   GET /sessions         snapshot (waybar polls, interval:2)
//   GET /sessions/stream  SSE push (the click-to-open TUI popup consumes this)
//   GET /health           {ok,name} for primary/follower election
//
// Multi-instance aware: only one opencode instance binds the port (primary);
// others detect the peer via /health and stay quiet. If the primary exits, a
// follower takes over within ~30s. Port convention: 4098 (see AGENTS.md).
//
// Command: /sessionbar (shows the list in a dialog; reports listener role)
// Options (tui.json tuple): { enabled?: true, port?: 4098 }

import type { TuiPlugin, TuiPluginModule } from "@opencode-ai/plugin/tui";
import { buildSnapshot } from "./snapshot.ts";
import { startSessionServer, type SessionServer } from "./server.ts";
import { EMPTY_SNAPSHOT, type SessionSnapshot } from "./store.ts";

const id = "sessionbar";

const DEFAULT_PORT = 4098;
const BIND_RETRY_MS = 30_000;
const PROBE_TIMEOUT_MS = 1_000;
// Debounce snapshot rebuilds: events can arrive in bursts.
const REBUILD_DEBOUNCE_MS = 150;

type Role = "starting" | "primary" | "follower" | "error";

const tui: TuiPlugin = async (api, options) => {
  const opts = (options ?? {}) as Record<string, unknown>;
  if (opts.enabled === false) return;

  // Port precedence: tui.json options tuple → OPENCODE_SESSIONBAR_PORT → 4098.
  // Matches consumers (CLI --port > env > default) so HM port and bar stay aligned.
  const envPort = (() => {
    const raw = (typeof process !== "undefined" && process.env?.OPENCODE_SESSIONBAR_PORT) || "";
    const n = Number.parseInt(raw, 10);
    return Number.isInteger(n) && n > 0 ? n : undefined;
  })();
  const port =
    typeof opts.port === "number" && Number.isInteger(opts.port) && opts.port > 0
      ? opts.port
      : (envPort ?? DEFAULT_PORT);

  let snapshot: SessionSnapshot = EMPTY_SNAPSHOT;
  let server: SessionServer | undefined;
  let role: Role = "starting";
  let warnedBindFailure = false;
  let rebuildTimer: ReturnType<typeof setTimeout> | undefined;
  let rebuilding = false;
  let rebuildAgain = false;

  const getSnapshot = (): SessionSnapshot => snapshot;

  // Rebuild the cached snapshot and push it to any SSE subscribers. Coalesces
  // overlapping rebuilds so a burst of events yields at most one in-flight.
  const rebuild = async (): Promise<void> => {
    if (rebuilding) {
      rebuildAgain = true;
      return;
    }
    rebuilding = true;
    try {
      snapshot = await buildSnapshot(api);
      server?.broadcast(snapshot);
    } catch {
      /* never break the TUI */
    } finally {
      rebuilding = false;
      if (rebuildAgain) {
        rebuildAgain = false;
        void rebuild();
      }
    }
  };

  const scheduleRebuild = (): void => {
    if (rebuildTimer) clearTimeout(rebuildTimer);
    rebuildTimer = setTimeout(() => void rebuild(), REBUILD_DEBOUNCE_MS);
  };

  // ---- listener: primary/follower election ----------------------------

  const probePeer = async (): Promise<boolean> => {
    try {
      const controller = new AbortController();
      const timer = setTimeout(() => controller.abort(), PROBE_TIMEOUT_MS);
      const response = await fetch(`http://127.0.0.1:${port}/health`, {
        signal: controller.signal,
      });
      clearTimeout(timer);
      if (!response.ok) return false;
      const body = (await response.json().catch(() => undefined)) as { name?: string } | undefined;
      return body?.name === "opencode-sessionbar";
    } catch {
      return false;
    }
  };

  const tryBind = async (): Promise<void> => {
    if (server) return;
    try {
      server = await startSessionServer(port, getSnapshot);
      role = "primary";
      await rebuild();
    } catch (error) {
      if (await probePeer()) {
        role = "follower"; // another opencode owns the port; expected, silent.
        return;
      }
      role = "error";
      if (!warnedBindFailure) {
        warnedBindFailure = true;
        api.ui.toast({
          title: "opencode-sessionbar",
          message: `Session server failed on 127.0.0.1:${port} (${String(error)}). Port held by a non-opencode process; retrying every ${BIND_RETRY_MS / 1000}s.`,
          variant: "warning",
          duration: 8_000,
        });
      }
    }
  };

  await tryBind();

  // ---- event subscriptions: keep the cached snapshot fresh ------------

  const unsubs = [
    api.event.on("session.status", scheduleRebuild),
    api.event.on("session.idle", scheduleRebuild),
    api.event.on("session.created", scheduleRebuild),
    api.event.on("session.updated", scheduleRebuild),
    api.event.on("session.deleted", scheduleRebuild),
    api.event.on("permission.asked", scheduleRebuild),
    api.event.on("permission.replied", scheduleRebuild),
    api.event.on("question.asked", scheduleRebuild),
    api.event.on("question.replied", scheduleRebuild),
    api.event.on("todo.updated", scheduleRebuild),
  ].filter((fn): fn is () => void => typeof fn === "function");

  // Periodic refresh so relative time labels age out + takeover when the
  // primary exits. Cheap: only the primary recomputes.
  const timers: ReturnType<typeof setInterval>[] = [
    setInterval(() => {
      if (role === "primary") void rebuild();
    }, BIND_RETRY_MS),
    setInterval(() => {
      if (!server) void tryBind();
    }, BIND_RETRY_MS),
  ];

  // ---- /sessionbar command --------------------------------------------

  const roleLine = (): string => {
    switch (role) {
      case "primary":
        return `listener: primary on 127.0.0.1:${server?.port ?? port} (${server?.subscribers() ?? 0} stream subscribers)`;
      case "follower":
        return `listener: follower (another opencode instance owns port ${port})`;
      case "error":
        return `listener: unavailable (port ${port} held by another process)`;
      default:
        return "listener: starting";
    }
  };

  const unregisterCommands = api.command.register(() => [
    {
      title: "SessionBar: Show sessions",
      value: "sessionbar.list",
      category: "SessionBar",
      description: "Current sessions served to waybar; reports listener role",
      slash: { name: "sessionbar" },
      onSelect: () => {
        void rebuild();
        const lines = snapshot.sessions.length
          ? snapshot.sessions.map(
              (s) => `[${s.status}] ${s.title} — ${s.detail} (${s.ageLabel})`,
            )
          : ["No active sessions."];
        const { DialogAlert } = api.ui;
        api.ui.dialog.replace(() => (
          <DialogAlert
            title={`SessionBar (${snapshot.sessions.length})`}
            message={[
              roleLine(),
              snapshot.summary.headline ? `headline: ${snapshot.summary.headline}` : "",
              "",
              ...lines,
            ]
              .filter(Boolean)
              .join("\n")}
            onConfirm={() => api.ui.dialog.clear()}
          />
        ));
      },
    },
  ]);

  api.lifecycle.onDispose(() => {
    if (rebuildTimer) clearTimeout(rebuildTimer);
    for (const t of timers) clearInterval(t);
    for (const off of unsubs) off();
    server?.stop();
    unregisterCommands();
  });
};

const plugin: TuiPluginModule & { id: string } = { id, tui };
export default plugin;
