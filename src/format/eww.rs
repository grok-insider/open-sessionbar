//! eww: emit JSON so an eww `(deflisten ...)` can destructure fields directly.
//! Same shape as the raw snapshot summary plus a flat `text`, for convenience.

use crate::model::Snapshot;

pub fn render(snap: Option<&Snapshot>) -> String {
    let snap = match snap {
        Some(s) if !s.is_empty() => s,
        _ => {
            return serde_json::json!({
                "text": "",
                "class": "empty",
                "total": 0,
                "waiting": 0,
                "busy": 0,
            })
            .to_string()
        }
    };
    serde_json::json!({
        "text": super::bar_text(snap),
        "class": snap.summary.headline_kind,
        "headline": snap.summary.headline,
        "total": snap.summary.total,
        "waiting": snap.summary.waiting,
        "busy": snap.summary.busy,
    })
    .to_string()
}
