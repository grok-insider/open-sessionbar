//! tmux status-line fragment: a single line with tmux style tags
//! (`#[fg=...]`/`#[default]`). Color busy text by the OpenCode agent mode
//! (build/plan); waiting/permission shows bold white. Drop it into a tmux
//! `status-left`/`status-right` via `#(opensessions bar --format tmux)`.
//!
//! tmux treats `#` as the start of a format/strftime expansion, so any literal
//! `#` from the session-derived text is doubled (`##`) — the style tags we add
//! are emitted verbatim.

use crate::model::Snapshot;
use crate::spinner::Anim;

pub fn render(snap: Option<&Snapshot>, anim: Anim) -> String {
    let snap = match snap {
        Some(s) if !s.is_empty() => s,
        _ => return String::new(),
    };
    let text = tmux_escape(&super::bar_text(snap, anim));
    match snap.summary.headline_kind.as_str() {
        "busy" => match super::mode_hex(snap.summary.mode.as_deref()) {
            Some(hex) => format!("#[fg={hex}]{text}#[default]"),
            None => text,
        },
        "permission" | "question" => format!("#[fg=#ffffff,bold]{text}#[default]"),
        _ => text,
    }
}

/// Double literal `#` so tmux renders it instead of treating it as the start of
/// a `#[...]`/`#(...)`/strftime expansion. Applied only to the dynamic text, not
/// to the style tags this formatter emits.
fn tmux_escape(s: &str) -> String {
    s.replace('#', "##")
}
