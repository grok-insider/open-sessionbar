//! Plain text: the bar text on the first line, then one line per session.
//! For generic bars, scripts, or `watch` in a terminal.

use crate::model::Snapshot;
use crate::spinner::Anim;

pub fn render(snap: Option<&Snapshot>, anim: Anim) -> String {
    let snap = match snap {
        Some(s) if !s.is_empty() => s,
        _ => return String::new(),
    };
    let mut out = super::bar_text(snap, anim);
    for line in super::tooltip_lines(snap) {
        out.push('\n');
        out.push_str(&line);
    }
    out
}
