//! Polybar: a single line with optional format tags. We keep it plain text by
//! default but wrap waiting states in %{F...} foreground tags for emphasis.

use crate::model::Snapshot;

pub fn render(snap: Option<&Snapshot>) -> String {
    let snap = match snap {
        Some(s) if !s.is_empty() => s,
        _ => return String::new(),
    };
    let text = super::bar_text(snap);
    match snap.summary.headline_kind.as_str() {
        // %{F#fff}...%{F-} = set then reset foreground (polybar markup).
        "permission" | "question" => format!("%{{F#ffffff}}{text}%{{F-}}"),
        _ => text,
    }
}
