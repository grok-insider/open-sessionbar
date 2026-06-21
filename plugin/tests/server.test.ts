// Server tests: /health, /sessions, SSE prime + push, cleanup, 404/405.
import { test, expect } from "bun:test";
import { startSessionServer } from "../server.ts";
import { EMPTY_SNAPSHOT, type SessionSnapshot } from "../store.ts";

const sample = (headline: string): SessionSnapshot => ({
  summary: { total: 2, busy: 1, waiting: 1, idle: 0, headline, headlineKind: "permission" },
  sessions: [
    { id: "s1", title: "search online", status: "waiting", detail: "Waiting for your permission · WebSearch", updated: Date.now(), ageLabel: "waiting <1m" },
    { id: "s2", title: "build login", status: "busy", detail: "Working…", updated: Date.now() - 60000, ageLabel: "1m" },
  ],
  at: Date.now(),
});

const freePort = () => 4300 + Math.floor(Math.random() * 200);

test("health + sessions", async () => {
  const port = freePort();
  const srv = await startSessionServer(port, () => sample("Waiting for your permission"));
  const base = `http://127.0.0.1:${port}`;
  try {
    const health = await (await fetch(`${base}/health`)).json();
    expect(health).toEqual({ ok: true, name: "opencode-sessionbar" });
    const snap = await (await fetch(`${base}/sessions`)).json();
    expect(snap.summary.headline).toBe("Waiting for your permission");
    expect(snap.sessions.length).toBe(2);
  } finally {
    srv.stop();
  }
});

test("SSE primes then pushes on broadcast", async () => {
  const port = freePort();
  let snap = sample("first");
  const srv = await startSessionServer(port, () => snap);
  const base = `http://127.0.0.1:${port}`;
  try {
    const res = await fetch(`${base}/sessions/stream`);
    const reader = res.body!.getReader();
    const dec = new TextDecoder();
    const readFrame = async (): Promise<string> => {
      let buf = "";
      while (!buf.includes("\n\n")) {
        const { value, done } = await reader.read();
        if (done) break;
        buf += dec.decode(value, { stream: true });
      }
      return buf;
    };
    const f1 = await readFrame();
    expect(f1.startsWith("event: snapshot")).toBe(true);
    expect(f1).toContain('"headline":"first"');

    snap = sample("second");
    srv.broadcast(snap);
    const f2 = await readFrame();
    const m = f2.match(/data: (.*)/);
    expect(m).toBeTruthy();
    expect(JSON.parse(m![1]).summary.headline).toBe("second");
    await reader.cancel();
  } finally {
    srv.stop();
  }
});

test("SSE subscriber cleaned up on connection abort", async () => {
  const port = freePort();
  const srv = await startSessionServer(port, () => EMPTY_SNAPSHOT);
  const base = `http://127.0.0.1:${port}`;
  try {
    const ac = new AbortController();
    const res = await fetch(`${base}/sessions/stream`, { signal: ac.signal });
    const reader = res.body!.getReader();
    await reader.read(); // primed frame
    expect(srv.subscribers()).toBe(1);
    ac.abort();
    await new Promise((r) => setTimeout(r, 300));
    expect(srv.subscribers()).toBe(0);
  } finally {
    srv.stop();
  }
});

test("404 + 405", async () => {
  const port = freePort();
  const srv = await startSessionServer(port, () => EMPTY_SNAPSHOT);
  const base = `http://127.0.0.1:${port}`;
  try {
    expect((await fetch(`${base}/nope`)).status).toBe(404);
    expect((await fetch(`${base}/sessions`, { method: "POST" })).status).toBe(405);
  } finally {
    srv.stop();
  }
});
