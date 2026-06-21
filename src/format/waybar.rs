//! Waybar custom-module JSON: {"text","tooltip","class"}.
//! Tooltip uses Pango markup (waybar renders it). `class` carries the headline
//! kind + agent mode (build/plan) + optional `pulse`, so style.css can color by
//! OpenCode mode and animate.

use crate::model::Snapshot;
use crate::spinner::Anim;

pub fn render(snap: Option<&Snapshot>, anim: Anim) -> String {
    let snap = match snap {
        Some(s) if !s.is_empty() => s,
        _ => return json_line("", "", "empty"),
    };

    let text = super::bar_text(snap, anim);
    let class = super::bar_classes(snap, anim);

    let waiting = snap.summary.waiting;
    let mut tip = format!("<b>SessionBar</b>  {} sessions", snap.summary.total);
    if waiting > 0 {
        tip.push_str(&format!("  ·  {waiting} waiting"));
    }
    for s in &snap.sessions {
        let mode = s
            .mode
            .as_deref()
            .filter(|m| !m.is_empty())
            .map(|m| format!(" [{}]", pango_escape(m)))
            .unwrap_or_default();
        tip.push('\n');
        tip.push_str(&format!(
            "{} {}{}  <i>{}</i>  ({})",
            s.glyph(),
            pango_escape(&s.title),
            mode,
            pango_escape(&s.detail),
            pango_escape(&s.age_label),
        ));
    }

    json_line(&text, &tip, &class)
}

fn json_line(text: &str, tooltip: &str, class: &str) -> String {
    // serde_json guarantees correct escaping for the JSON layer.
    serde_json::json!({ "text": text, "tooltip": tooltip, "class": class }).to_string()
}

/// Escape the five XML/Pango entities so titles with `&`/`<` don't break markup.
fn pango_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\'', "&apos;")
        .replace('"', "&quot;")
}
