//! Raw snapshot JSON passthrough (re-serialized canonical form). When the
//! plugin is unreachable, emits an explicit empty snapshot.

use crate::model::Snapshot;

pub fn render(snap: Option<&Snapshot>) -> String {
    match snap {
        Some(s) => serde_json::json!({
            "summary": {
                "total": s.summary.total,
                "busy": s.summary.busy,
                "waiting": s.summary.waiting,
                "idle": s.summary.idle,
                "headline": s.summary.headline,
                "headlineKind": s.summary.headline_kind,
            },
            "sessions": s.sessions.iter().map(|e| serde_json::json!({
                "id": e.id, "title": e.title, "status": e.status,
                "detail": e.detail, "updated": e.updated, "ageLabel": e.age_label,
            })).collect::<Vec<_>>(),
            "at": s.at,
        })
        .to_string(),
        None => serde_json::json!({
            "summary": { "total": 0, "busy": 0, "waiting": 0, "idle": 0, "headline": "", "headlineKind": "idle" },
            "sessions": [],
            "at": 0,
        })
        .to_string(),
    }
}
