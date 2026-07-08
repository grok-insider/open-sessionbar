// opencode-sessionbar - builds a SessionSnapshot from live opencode state.
//
// Sources (all read-only):
//   api.client.session.list()    -> all sessions (title, time, parentID)
//   api.client.session.status()  -> { [sessionID]: SessionStatus } map
//   api.state.session.permission/.question/.todo(id) -> per-session detail
//
// Headline precedence matches the screenshot semantics:
//   permission > question > busy > idle/done

import type { TuiPluginApi } from "@opencode-ai/plugin/tui";
import {
  EMPTY_SNAPSHOT,
  timeAgo,
  truncate,
  type HeadlineKind,
  type SessionBarStatus,
  type SessionEntry,
  type SessionSnapshot,
} from "./store.ts";

// Minimal structural views of the SDK shapes we consume (avoids depending on
// the full generated types at call sites; the real types live in @opencode-ai).
type SessionLite = {
  id: string;
  title?: string;
  parentID?: string;
  time?: { updated?: number; created?: number };
};
type StatusLite = { type: "idle" | "busy" | "retry" } | undefined;

const TITLE_MAX = 80;
const DETAIL_MAX = 80;
// The bar mirrors *active* work, not the whole history. Drop sessions whose
// last update is older than this (unless they're currently waiting/busy), and
// cap how many we list so the tooltip stays readable.
const STALE_MS = 6 * 60 * 60 * 1000; // 6h
const MAX_LISTED = 12;

/** Pull a tool/permission label out of a permission request for the detail line. */
const permissionLabel = (req: { permission?: string; patterns?: string[] }): string => {
  const pat = req.patterns?.find((p) => typeof p === "string" && p.length > 0);
  return pat ?? req.permission ?? "permission";
};

/**
 * Build the snapshot. Defensive throughout: any SDK hiccup degrades to the
 * empty snapshot rather than throwing into the HTTP handler / event loop.
 */
export const buildSnapshot = async (api: TuiPluginApi): Promise<SessionSnapshot> => {
  let sessions: SessionLite[] = [];
  let statusMap: Record<string, StatusLite> = {};

  try {
    const listed = await api.client.session.list();
    const data = (listed as { data?: unknown }).data;
    if (Array.isArray(data)) sessions = data as SessionLite[];
  } catch {
    return { ...EMPTY_SNAPSHOT, at: Date.now() };
  }

  try {
    const res = await api.client.session.status();
    const data = (res as { data?: unknown }).data;
    if (data && typeof data === "object") statusMap = data as Record<string, StatusLite>;
  } catch {
    /* status map best-effort; treat as all-idle */
  }

  const now = Date.now();
  const entries: SessionEntry[] = [];
  // Kind of waiting (permission vs question) for headline selection after sort.
  const waitKind = new Map<string, "permission" | "question">();

  for (const s of sessions) {
    if (s.parentID) continue; // skip sub-sessions (forked children)
    const id = s.id;
    if (!id) continue;

    const updated = s.time?.updated ?? s.time?.created ?? 0;
    const rawTitle = (s.title ?? "").trim() || "Untitled session";
    const title = truncate(rawTitle, TITLE_MAX);

    // Permission / question take precedence over the coarse busy/idle status.
    let perms: ReadonlyArray<{ permission?: string; patterns?: string[] }> = [];
    let questions: ReadonlyArray<unknown> = [];
    try {
      perms = api.state.session.permission(id) as never;
    } catch {
      /* ignore */
    }
    try {
      questions = api.state.session.question(id) as never;
    } catch {
      /* ignore */
    }

    const st = statusMap[id];
    let status: SessionBarStatus;
    let detail: string;
    let ageLabel: string;

    if (perms.length > 0) {
      status = "waiting";
      detail = truncate(`Waiting for your permission · ${permissionLabel(perms[0]!)}`, DETAIL_MAX);
      ageLabel = `waiting ${timeAgo(updated, now)}`;
      waitKind.set(id, "permission");
    } else if (questions.length > 0) {
      status = "waiting";
      detail = "Waiting for your answer";
      ageLabel = `waiting ${timeAgo(updated, now)}`;
      waitKind.set(id, "question");
    } else if (st?.type === "busy" || st?.type === "retry") {
      status = "busy";
      detail = st.type === "retry" ? "Retrying…" : "Working…";
      ageLabel = timeAgo(updated, now);
    } else {
      // Idle: distinguish "done" (had todos, all complete) from plain idle.
      let allDone = false;
      try {
        const todos = api.state.session.todo(id);
        allDone = todos.length > 0 && todos.every((t) => t.status === "completed");
      } catch {
        /* ignore */
      }
      status = allDone ? "done" : "idle";
      detail = allDone ? "Done" : "Idle";
      ageLabel = timeAgo(updated, now);
    }

    const active = status === "waiting" || status === "busy";
    // Stale idle/done sessions don't belong on the bar.
    if (!active && now - updated > STALE_MS) {
      continue;
    }

    // Agent/mode of the latest assistant message — only for active sessions
    // (drives OpenCode-matching color in consumers). Skip idle/done to avoid
    // walking message history on every rebuild for sessions that won't color.
    let mode: string | undefined;
    if (active) {
      try {
        const msgs = api.state.session.messages(id) as ReadonlyArray<{
          role?: string;
          agent?: string;
          mode?: string;
        }>;
        for (let i = msgs.length - 1; i >= 0; i--) {
          const m = msgs[i]!;
          if (m.role === "assistant") {
            mode = m.agent ?? m.mode ?? undefined;
            break;
          }
        }
      } catch {
        /* ignore */
      }
    }

    entries.push({ id, title, status, detail, updated, ageLabel, mode });
  }

  // Prefer active sessions when capping; within each group sort by updated desc.
  const rank = (s: SessionEntry): number =>
    s.status === "waiting" ? 0 : s.status === "busy" ? 1 : 2;
  entries.sort((a, b) => {
    const ra = rank(a);
    const rb = rank(b);
    if (ra !== rb) return ra - rb;
    return b.updated - a.updated;
  });
  if (entries.length > MAX_LISTED) entries.length = MAX_LISTED;

  // Summary counts from the final listed set so total/busy/waiting/idle match
  // sessions[] (no pre-cap drift).
  let busy = 0;
  let waiting = 0;
  let idle = 0;
  for (const e of entries) {
    if (e.status === "busy") busy++;
    else if (e.status === "waiting") waiting++;
    else idle++;
  }

  // Headline drivers: most recently updated of each kind in the listed set.
  let topPermission: SessionEntry | undefined;
  let topQuestion: SessionEntry | undefined;
  let topBusy: SessionEntry | undefined;
  for (const e of entries) {
    if (e.status === "waiting" && waitKind.get(e.id) === "permission") {
      if (!topPermission || e.updated > topPermission.updated) topPermission = e;
    } else if (e.status === "waiting" && waitKind.get(e.id) === "question") {
      if (!topQuestion || e.updated > topQuestion.updated) topQuestion = e;
    } else if (e.status === "busy") {
      if (!topBusy || e.updated > topBusy.updated) topBusy = e;
    }
  }

  let headline = "";
  let headlineKind: HeadlineKind = "idle";
  let headlineSession: SessionEntry | undefined;
  if (topPermission) {
    headline = "Waiting for your permission";
    headlineKind = "permission";
    headlineSession = topPermission;
  } else if (topQuestion) {
    headline = "Waiting for your answer";
    headlineKind = "question";
    headlineSession = topQuestion;
  } else if (topBusy) {
    headline = busy > 1 ? `Working · ${busy} sessions` : "Working";
    headlineKind = "busy";
    headlineSession = topBusy;
  } else if (entries.length > 0) {
    headline = entries.length === 1 ? "1 session" : `${entries.length} sessions`;
    headlineKind = "idle";
  }

  return {
    summary: {
      total: entries.length,
      busy,
      waiting,
      idle,
      headline,
      headlineKind,
      // Mode of the session driving the headline (if any).
      mode: headlineSession?.mode,
    },
    sessions: entries,
    at: now,
  };
};
