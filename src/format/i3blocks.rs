//! i3blocks: three lines — full_text, short_text, color (hex or empty).
//! i3blocks reads stdout line-by-line; a non-zero color signals attention.

use crate::model::Snapshot;

pub fn render(snap: Option<&Snapshot>) -> String {
    let snap = match snap {
        Some(s) if !s.is_empty() => s,
        _ => return "\n\n".to_string(), // empty block
    };
    let full = super::bar_text(snap);
    let short = match snap.summary.headline_kind.as_str() {
        "permission" | "question" => format!("⏳{}", snap.summary.waiting.max(1)),
        "busy" => "●".to_string(),
        _ => format!("{}", snap.sessions.len()),
    };
    // i3bar accepts #RRGGBB; leave blank for idle so themes win.
    let color = match snap.summary.headline_kind.as_str() {
        "permission" | "question" => "#ffffff",
        _ => "",
    };
    format!("{full}\n{short}\n{color}")
}
