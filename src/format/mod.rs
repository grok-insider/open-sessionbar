//! Status-bar formatters. Each turns a Snapshot into one line of text for a
//! specific bar. Desktop-environment-agnostic: pick with `bar --format <name>`.

use crate::model::Snapshot;

mod eww;
mod i3blocks;
mod json;
mod plain;
mod polybar;
mod waybar;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Waybar,
    I3blocks,
    Polybar,
    Eww,
    Plain,
    Json,
}

impl Format {
    pub fn parse(s: &str) -> Option<Format> {
        match s {
            "waybar" => Some(Format::Waybar),
            "i3blocks" => Some(Format::I3blocks),
            "polybar" => Some(Format::Polybar),
            "eww" => Some(Format::Eww),
            "plain" => Some(Format::Plain),
            "json" => Some(Format::Json),
            _ => None,
        }
    }

    pub fn all() -> &'static [&'static str] {
        &["waybar", "i3blocks", "polybar", "eww", "plain", "json"]
    }
}

/// Render a snapshot for the given bar format. `None` snapshot means the plugin
/// is unreachable; formatters render an empty/hidden module.
pub fn render(fmt: Format, snap: Option<&Snapshot>) -> String {
    match fmt {
        Format::Waybar => waybar::render(snap),
        Format::I3blocks => i3blocks::render(snap),
        Format::Polybar => polybar::render(snap),
        Format::Eww => eww::render(snap),
        Format::Plain => plain::render(snap),
        Format::Json => json::render(snap),
    }
}

/// Text shown on the bar: the headline, or a compact count, or empty.
pub(crate) fn bar_text(snap: &Snapshot) -> String {
    if snap.is_empty() {
        return String::new();
    }
    if !snap.summary.headline.is_empty() {
        snap.summary.headline.clone()
    } else {
        let n = snap.sessions.len();
        if n == 1 {
            "1 session".to_string()
        } else {
            format!("{n} sessions")
        }
    }
}

/// Multi-line tooltip body listing every session (plain text; bar-specific
/// markup is applied by the caller).
pub(crate) fn tooltip_lines(snap: &Snapshot) -> Vec<String> {
    snap.sessions
        .iter()
        .map(|s| format!("{} {}  {}  ({})", s.glyph(), s.title, s.detail, s.age_label))
        .collect()
}
