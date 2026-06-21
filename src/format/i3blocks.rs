//! i3blocks: three lines — full_text, short_text, color (hex or empty).
//! i3blocks reads stdout line-by-line; a non-zero color signals attention.

use crate::model::Snapshot;
use crate::spinner::Anim;

pub fn render(snap: Option<&Snapshot>, anim: Anim) -> String {
    let snap = match snap {
        Some(s) if !s.is_empty() => s,
        _ => return "\n\n".to_string(), // empty block
    };
    let full = super::bar_text(snap, anim);
    let short = match snap.summary.headline_kind.as_str() {
        "permission" | "question" => format!("⏳{}", snap.summary.waiting.max(1)),
        "busy" => "●".to_string(),
        _ => format!("{}", snap.sessions.len()),
    };
    // i3bar accepts #RRGGBB. Color busy by OpenCode agent mode; waiting white.
    let color = match snap.summary.headline_kind.as_str() {
        "busy" => mode_hex(snap.summary.mode.as_deref()).unwrap_or(""),
        "permission" | "question" => "#ffffff",
        _ => "",
    };
    format!("{full}\n{short}\n{color}")
}

/// OpenCode agent colors (dark theme): build #034cff, plan #a753ae.
fn mode_hex(mode: Option<&str>) -> Option<&'static str> {
    match mode {
        Some("build") => Some("#034cff"),
        Some("plan") => Some("#a753ae"),
        _ => None,
    }
}
