// opencode-sessionbar - localhost read-only session-state server.
//
// Runs inside the opencode TUI process (Bun). Binds to 127.0.0.1 only.
//
//   GET /health           liveness + identity probe (primary/follower election)
//   GET /sessions         current snapshot (waybar polls this)
//   GET /sessions/stream  Server-Sent Events: a "snapshot" frame on every change
//
// Port convention: 4098 (see ~/.config/opencode/AGENTS.md port table).
// Falls back to node:http when the Bun global is unavailable.

import type { SessionSnapshot } from "./store.ts";

export type SnapshotProvider = () => SessionSnapshot;

export type SessionServer = {
  port: number;
  /** Push the latest snapshot to every connected SSE subscriber. */
  broadcast: (snapshot: SessionSnapshot) => void;
  /** Number of live SSE subscribers (for diagnostics). */
  subscribers: () => number;
  stop: () => void;
};

const HOSTNAME = "127.0.0.1";
const NAME = "opencode-sessionbar";
const SSE_PING_MS = 25_000;

const json = (status: number, body: unknown): Response =>
  new Response(JSON.stringify(body), {
    status,
    headers: { "content-type": "application/json" },
  });

const sseFrame = (snapshot: SessionSnapshot): string =>
  `event: snapshot\ndata: ${JSON.stringify(snapshot)}\n\n`;

// ---- subscriber registry (shared by both runtimes) ----------------------

type Subscriber = {
  send: (chunk: string) => void;
  close: () => void;
};

const makeRegistry = () => {
  const subs = new Set<Subscriber>();
  return {
    add: (s: Subscriber) => subs.add(s),
    remove: (s: Subscriber) => subs.delete(s),
    size: () => subs.size,
    broadcast: (snapshot: SessionSnapshot) => {
      const frame = sseFrame(snapshot);
      for (const s of subs) {
        try {
          s.send(frame);
        } catch {
          subs.delete(s);
        }
      }
    },
    closeAll: () => {
      for (const s of subs) {
        try {
          s.close();
        } catch {
          /* ignore */
        }
      }
      subs.clear();
    },
  };
};

type Registry = ReturnType<typeof makeRegistry>;

// ---- Bun runtime --------------------------------------------------------

type BunLike = {
  serve: (options: {
    hostname: string;
    port: number;
    idleTimeout?: number;
    fetch: (req: Request) => Response | Promise<Response>;
  }) => { port: number; stop: (closeActive?: boolean) => void };
};

const startWithBun = (
  bun: BunLike,
  port: number,
  getSnapshot: SnapshotProvider,
  registry: Registry,
): SessionServer => {
  const pings = new Set<ReturnType<typeof setInterval>>();

  const server = bun.serve({
    hostname: HOSTNAME,
    port,
    // /sessions/stream is a long-lived SSE connection that only pings every
    // SSE_PING_MS (25s). Bun's default idleTimeout (10s) would kill a quiet
    // stream before the first ping, so disable it; the stream manages its own
    // lifecycle via the ping interval + cancel/abort cleanup.
    idleTimeout: 0,
    fetch: (req) => {
      const url = new URL(req.url);
      if (req.method !== "GET") return json(405, { ok: false, error: "GET only" });

      if (url.pathname === "/health") return json(200, { ok: true, name: NAME });
      if (url.pathname === "/sessions") return json(200, getSnapshot());

      if (url.pathname === "/sessions/stream") {
        const encoder = new TextEncoder();
        let ping: ReturnType<typeof setInterval> | undefined;
        let sub: Subscriber | undefined;
        const stream = new ReadableStream<Uint8Array>({
          start(controller) {
            sub = {
              send: (chunk) => controller.enqueue(encoder.encode(chunk)),
              close: () => {
                try {
                  controller.close();
                } catch {
                  /* already closed */
                }
              },
            };
            registry.add(sub);
            // Prime with the current snapshot immediately.
            sub.send(sseFrame(getSnapshot()));
            ping = setInterval(() => {
              try {
                controller.enqueue(encoder.encode(": ping\n\n"));
              } catch {
                /* ignore */
              }
            }, SSE_PING_MS);
            pings.add(ping);
          },
          cancel() {
            if (sub) registry.remove(sub);
            if (ping) {
              clearInterval(ping);
              pings.delete(ping);
            }
          },
        });
        return new Response(stream, {
          status: 200,
          headers: {
            "content-type": "text/event-stream",
            "cache-control": "no-cache",
            connection: "keep-alive",
          },
        });
      }

      return json(404, { ok: false, error: "not found" });
    },
  });

  return {
    port: server.port,
    broadcast: registry.broadcast,
    subscribers: registry.size,
    stop: () => {
      for (const p of pings) clearInterval(p);
      registry.closeAll();
      server.stop(true);
    },
  };
};

// ---- node:http fallback -------------------------------------------------

const startWithNode = async (
  port: number,
  getSnapshot: SnapshotProvider,
  registry: Registry,
): Promise<SessionServer> => {
  const http = await import("node:http");
  const pings = new Set<ReturnType<typeof setInterval>>();

  const server = http.createServer((req, res) => {
    const path = (req.url ?? "/").split("?")[0];
    if (req.method !== "GET") {
      res.writeHead(405, { "content-type": "application/json" });
      res.end(JSON.stringify({ ok: false, error: "GET only" }));
      return;
    }
    if (path === "/health") {
      res.writeHead(200, { "content-type": "application/json" });
      res.end(JSON.stringify({ ok: true, name: NAME }));
      return;
    }
    if (path === "/sessions") {
      res.writeHead(200, { "content-type": "application/json" });
      res.end(JSON.stringify(getSnapshot()));
      return;
    }
    if (path === "/sessions/stream") {
      res.writeHead(200, {
        "content-type": "text/event-stream",
        "cache-control": "no-cache",
        connection: "keep-alive",
      });
      const sub: Subscriber = {
        send: (chunk) => res.write(chunk),
        close: () => res.end(),
      };
      registry.add(sub);
      sub.send(sseFrame(getSnapshot()));
      const ping = setInterval(() => {
        try {
          res.write(": ping\n\n");
        } catch {
          /* ignore */
        }
      }, SSE_PING_MS);
      pings.add(ping);
      const cleanup = () => {
        registry.remove(sub);
        clearInterval(ping);
        pings.delete(ping);
      };
      req.on("close", cleanup);
      res.on("close", cleanup);
      return;
    }
    res.writeHead(404, { "content-type": "application/json" });
    res.end(JSON.stringify({ ok: false, error: "not found" }));
  });

  await new Promise<void>((resolve, reject) => {
    server.once("error", reject);
    server.listen(port, HOSTNAME, resolve);
  });

  return {
    port,
    broadcast: registry.broadcast,
    subscribers: registry.size,
    stop: () => {
      for (const p of pings) clearInterval(p);
      registry.closeAll();
      server.close();
    },
  };
};

/**
 * Start the read-only session server. Throws if the port is taken; the caller
 * probes /health to decide whether to become a follower.
 */
export const startSessionServer = async (
  port: number,
  getSnapshot: SnapshotProvider,
): Promise<SessionServer> => {
  const registry = makeRegistry();
  const bun = (globalThis as Record<string, unknown>).Bun as BunLike | undefined;
  if (bun && typeof bun.serve === "function") return startWithBun(bun, port, getSnapshot, registry);
  return startWithNode(port, getSnapshot, registry);
};
