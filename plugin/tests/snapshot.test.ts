// Snapshot tests: precedence, stale filter, sub-session filter, counts, caps.
import { test, expect } from "bun:test";
import { buildSnapshot } from "../snapshot.ts";

const makeApi = (
  sessions: unknown[],
  statusMap: Record<string, { type: string }> = {},
  perms: Record<string, unknown[]> = {},
  messages: Record<string, unknown[]> = {},
  questions: Record<string, unknown[]> = {},
) =>
  ({
    client: {
      session: {
        list: async () => ({ data: sessions }),
        status: async () => ({ data: statusMap }),
      },
    },
    state: {
      session: {
        permission: (id: string) => perms[id] ?? [],
        question: (id: string) => questions[id] ?? [],
        todo: () => [],
        messages: (id: string) => messages[id] ?? [],
      },
    },
  }) as never;

test("precedence, stale + sub-session filtering, counts", async () => {
  const now = Date.now();
  const sessions = [
    { id: "fresh-idle", title: "Recent idle", time: { updated: now - 60_000 } },
    { id: "stale-idle", title: "Old idle", time: { updated: now - 9 * 3600 * 1000 } },
    { id: "busy1", title: "Working now", time: { updated: now - 1000 } },
    { id: "perm1", title: "Needs permission", time: { updated: now - 2000 } },
    { id: "child", title: "sub", parentID: "busy1", time: { updated: now } },
  ];
  const snap = await buildSnapshot(
    makeApi(sessions, { busy1: { type: "busy" } }, { perm1: [{ permission: "webfetch", patterns: ["WebSearch"] }] }),
  );

  const ids = snap.sessions.map((s) => s.id);
  expect(ids).toContain("busy1");
  expect(ids).toContain("perm1");
  expect(ids).toContain("fresh-idle");
  expect(ids).not.toContain("stale-idle"); // stale dropped
  expect(ids).not.toContain("child"); // sub-session dropped

  expect(snap.summary.headline).toBe("Waiting for your permission");
  expect(snap.summary.headlineKind).toBe("permission");
  expect(snap.summary.total).toBe(3);
  expect(snap.summary.busy).toBe(1);
  expect(snap.summary.waiting).toBe(1);
  expect(snap.summary.idle).toBe(1);

  expect(snap.sessions.find((s) => s.id === "perm1")?.detail).toBe("Waiting for your permission · WebSearch");
});

test("empty list -> empty snapshot", async () => {
  const snap = await buildSnapshot(makeApi([], {}));
  expect(snap.sessions.length).toBe(0);
  expect(snap.summary.headline).toBe("");
});

test("busy headline when no waiting", async () => {
  const now = Date.now();
  const snap = await buildSnapshot(
    makeApi([{ id: "b", title: "x", time: { updated: now } }], { b: { type: "busy" } }),
  );
  expect(snap.summary.headlineKind).toBe("busy");
  expect(snap.summary.headline).toBe("Working");
});

test("agent mode from latest assistant message", async () => {
  const now = Date.now();
  const messages = {
    b: [
      { role: "user" },
      { role: "assistant", agent: "build", mode: "build" },
      { role: "user" },
      { role: "assistant", agent: "plan", mode: "plan" }, // latest wins
    ],
  };
  const snap = await buildSnapshot(
    makeApi([{ id: "b", title: "x", time: { updated: now } }], { b: { type: "busy" } }, {}, messages),
  );
  expect(snap.sessions[0]!.mode).toBe("plan");
  expect(snap.summary.mode).toBe("plan"); // headline (top busy) mode
});

test("top permission is most recently updated, not list order", async () => {
  const now = Date.now();
  const sessions = [
    { id: "old-perm", title: "Old wait", time: { updated: now - 10_000 } },
    { id: "new-perm", title: "New wait", time: { updated: now - 100 } },
  ];
  const perms = {
    "old-perm": [{ permission: "bash" }],
    "new-perm": [{ permission: "webfetch", patterns: ["WebSearch"] }],
  };
  const snap = await buildSnapshot(makeApi(sessions, {}, perms));
  expect(snap.summary.headlineKind).toBe("permission");
  // Mode / headline should reflect the newer waiter (no mode without messages).
  expect(snap.sessions[0]!.id).toBe("new-perm"); // active sorted by updated desc
});

test("question takes precedence over busy", async () => {
  const now = Date.now();
  const snap = await buildSnapshot(
    makeApi(
      [
        { id: "b", title: "busy", time: { updated: now } },
        { id: "q", title: "ask", time: { updated: now - 1 } },
      ],
      { b: { type: "busy" } },
      {},
      {},
      { q: [{ id: "q1" }] },
    ),
  );
  expect(snap.summary.headlineKind).toBe("question");
  expect(snap.summary.headline).toBe("Waiting for your answer");
  expect(snap.summary.waiting).toBe(1);
  expect(snap.summary.busy).toBe(1);
});

test("multi-busy headline counts listed busy sessions", async () => {
  const now = Date.now();
  const snap = await buildSnapshot(
    makeApi(
      [
        { id: "a", title: "a", time: { updated: now } },
        { id: "b", title: "b", time: { updated: now - 1 } },
      ],
      { a: { type: "busy" }, b: { type: "busy" } },
    ),
  );
  expect(snap.summary.headlineKind).toBe("busy");
  expect(snap.summary.headline).toBe("Working · 2 sessions");
  expect(snap.summary.busy).toBe(2);
});

test("cap prefers active sessions; summary matches listed set", async () => {
  const now = Date.now();
  const sessions = [];
  for (let i = 0; i < 15; i++) {
    sessions.push({
      id: `idle${i}`,
      title: `Idle ${i}`,
      time: { updated: now - i * 1000 },
    });
  }
  sessions.push({ id: "busy1", title: "Busy", time: { updated: now + 1 } });
  const snap = await buildSnapshot(
    makeApi(sessions, { busy1: { type: "busy" } }),
  );
  expect(snap.sessions.length).toBe(12);
  expect(snap.sessions.some((s) => s.id === "busy1")).toBe(true);
  expect(snap.summary.total).toBe(12);
  expect(snap.summary.busy).toBe(1);
  expect(snap.summary.idle).toBe(11);
  expect(snap.summary.busy + snap.summary.waiting + snap.summary.idle).toBe(
    snap.summary.total,
  );
});
