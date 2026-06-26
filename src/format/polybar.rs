//! Polybar: a single line with optional format tags. Color busy text by the
//! OpenCode agent mode (build/plan) via %{F...} foreground tags.

use crate::model::Snapshot;
use crate::spinner::Anim;

pub fn render(snap: Option<&Snapshot>, anim: Anim) -> String {
    let snap = match snap {
        Some(s) if !s.is_empty() => s,
        _ => return String::new(),
    };
    let text = super::bar_text(snap, anim);
    match snap.summary.headline_kind.as_str() {
        // %{F#fff}...%{F-} = set then reset foreground (polybar markup).
        "busy" => match super::mode_hex(snap.summary.mode.as_deref()) {
            Some(hex) => format!("%{{F{hex}}}{text}%{{F-}}"),
            None => text,
        },
        "permission" | "question" => format!("%{{F#ffffff}}{text}%{{F-}}"),
        _ => text,
    }
}
