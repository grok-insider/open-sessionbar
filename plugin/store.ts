// opencode-sessionbar - shared types and helpers for the session snapshot.
//
// The snapshot is what we serve over HTTP (GET /sessions) and push over SSE
// (GET /sessions/stream). It mirrors the "SessionBar" list: one entry per
// top-level session with a coarse status, a human detail line, and a
// relative-time label.

export const STATUSES = ["waiting", "busy", "done", "idle"] as const;
export type SessionBarStatus = (typeof STATUSES)[number];

/** What the bar headline is reacting to (drives waybar `class`). */
export const HEADLINE_KINDS = ["permission", "question", "busy", "idle"] as const;
export type HeadlineKind = (typeof HEADLINE_KINDS)[number];

export type SessionEntry = {
  id: string;
  title: string;
  status: SessionBarStatus;
  /** Human one-liner, e.g. "Waiting for your permission · WebSearch" or "Done". */
  detail: string;
  /** epoch ms of session.time.updated */
  updated: number;
  /** compact relative label, e.g. "5m", "waiting <1m" */
  ageLabel: string;
  /** Agent of the latest assistant message: "build" | "plan" | custom name. */
  mode?: string;
};

export type SessionSummary = {
  total: number;
  busy: number;
  waiting: number;
  idle: number;
  headline: string;
  headlineKind: HeadlineKind;
  /** Agent/mode of the session driving the headline (the top busy one). */
  mode?: string;
};

export type SessionSnapshot = {
  summary: SessionSummary;
  sessions: SessionEntry[];
  /** epoch ms the snapshot was built (lets consumers detect staleness) */
  at: number;
};

export const EMPTY_SNAPSHOT: SessionSnapshot = {
  summary: { total: 0, busy: 0, waiting: 0, idle: 0, headline: "", headlineKind: "idle" },
  sessions: [],
  at: 0,
};

/** Compact relative time: "now", "42s", "5m", "3h", "2d". */
export const timeAgo = (at: number, now: number = Date.now()): string => {
  const s = Math.max(0, Math.floor((now - at) / 1000));
  if (s < 10) return "now";
  if (s < 60) return `${s}s`;
  const m = Math.floor(s / 60);
  if (m < 60) return `${m}m`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h}h`;
  return `${Math.floor(h / 24)}d`;
};

export const truncate = (text: string, max: number): string => {
  if (text.length <= max) return text;
  return `${text.slice(0, Math.max(1, max - 1)).trimEnd()}…`;
};
