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
  let busy = 0;
  let waiting = 0;
  let idle = 0;

  // Highest-priority waiting session, for the headline.
  let topPermission: SessionEntry | undefined;
  let topQuestion: SessionEntry | undefined;
  let topBusy: SessionEntry | undefined;

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

    // Agent/mode of the latest assistant message (build/plan/custom) — drives
    // the OpenCode-matching color in consumers. Best-effort; older messages
    // may lack the field.
    let mode: string | undefined;
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

    const st = statusMap[id];
    let status: SessionBarStatus;
    let detail: string;
    let ageLabel: string;

    if (perms.length > 0) {
      status = "waiting";
      detail = truncate(`Waiting for your permission · ${permissionLabel(perms[0]!)}`, DETAIL_MAX);
      ageLabel = `waiting ${timeAgo(updated, now)}`;
      waiting++;
    } else if (questions.length > 0) {
      status = "waiting";
      detail = "Waiting for your answer";
      ageLabel = `waiting ${timeAgo(updated, now)}`;
      waiting++;
    } else if (st?.type === "busy" || st?.type === "retry") {
      status = "busy";
      detail = st.type === "retry" ? "Retrying…" : "Working…";
      ageLabel = timeAgo(updated, now);
      busy++;
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
      idle++;
    }

    const active = status === "waiting" || status === "busy";
    // Stale idle/done sessions don't belong on the bar. Only idle/done reach
    // here when stale (active sessions are always kept), and both increment the
    // idle counter above, so roll that back.
    if (!active && now - updated > STALE_MS) {
      idle--;
      continue;
    }

    const entry: SessionEntry = { id, title, status, detail, updated, ageLabel, mode };
    entries.push(entry);

    if (status === "waiting" && perms.length > 0 && !topPermission) topPermission = entry;
    else if (status === "waiting" && !topQuestion && !topPermission) topQuestion = entry;
    else if (status === "busy" && !topBusy) topBusy = entry;
  }

  entries.sort((a, b) => b.updated - a.updated);
  if (entries.length > MAX_LISTED) entries.length = MAX_LISTED;

  let headline = "";
  let headlineKind: HeadlineKind = "idle";
  if (topPermission) {
    headline = "Waiting for your permission";
    headlineKind = "permission";
  } else if (topQuestion) {
    headline = "Waiting for your answer";
    headlineKind = "question";
  } else if (topBusy) {
    headline = busy > 1 ? `Working · ${busy} sessions` : "Working";
    headlineKind = "busy";
  } else if (entries.length > 0) {
    headline = entries.length === 1 ? "1 session" : `${entries.length} sessions`;
    headlineKind = "idle";
  }

  return {
    summary: {
      total: entries.length,
      busy,
      waiting,
      idle: entries.length - busy - waiting,
      headline,
      headlineKind,
      mode: topBusy?.mode,
    },
    sessions: entries,
    at: now,
  };
};
