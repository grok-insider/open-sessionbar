//! eww: emit JSON so an eww `(deflisten ...)` can destructure fields directly.
//! Includes `mode` and a resolved `color` (OpenCode agent hex) for convenience.

use crate::model::Snapshot;
use crate::spinner::Anim;

pub fn render(snap: Option<&Snapshot>, anim: Anim) -> String {
    let snap = match snap {
        Some(s) if !s.is_empty() => s,
        _ => {
            return serde_json::json!({
                "text": "",
                "class": "empty",
                "total": 0,
                "waiting": 0,
                "busy": 0,
                "mode": serde_json::Value::Null,
                "color": serde_json::Value::Null,
            })
            .to_string()
        }
    };
    let mode = snap.summary.mode.as_deref();
    serde_json::json!({
        "text": super::bar_text(snap, anim),
        "class": super::bar_classes(snap, anim),
        "headline": snap.summary.headline,
        "total": snap.summary.total,
        "waiting": snap.summary.waiting,
        "busy": snap.summary.busy,
        "mode": mode,
        "color": if super::is_busy(snap) { mode_hex(mode) } else { None },
    })
    .to_string()
}

/// OpenCode agent colors (dark theme): build #034cff, plan #a753ae.
fn mode_hex(mode: Option<&str>) -> Option<&'static str> {
    match mode {
        Some("build") => Some("#034cff"),
        Some("plan") => Some("#a753ae"),
        _ => None,
    }
}
