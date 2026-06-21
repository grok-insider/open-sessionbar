//! Waybar custom-module JSON: {"text","tooltip","class"}.
//! Tooltip uses Pango markup (waybar renders it).

use crate::model::Snapshot;

pub fn render(snap: Option<&Snapshot>) -> String {
    let snap = match snap {
        Some(s) if !s.is_empty() => s,
        _ => return json_line("", "", "empty"),
    };

    let text = super::bar_text(snap);
    let class = snap.summary.headline_kind.clone();

    let waiting = snap.summary.waiting;
    let mut tip = format!("<b>SessionBar</b>  {} sessions", snap.summary.total);
    if waiting > 0 {
        tip.push_str(&format!("  ·  {waiting} waiting"));
    }
    for s in &snap.sessions {
        tip.push('\n');
        tip.push_str(&format!(
            "{} {}  <i>{}</i>  ({})",
            s.glyph(),
            pango_escape(&s.title),
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
