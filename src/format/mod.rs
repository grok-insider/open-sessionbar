//! Status-bar formatters. Each turns a Snapshot into one line of text for a
//! specific bar. Desktop-environment-agnostic: pick with `bar --format <name>`.

use crate::model::Snapshot;
use crate::spinner::Anim;

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
/// is unreachable; formatters render an empty/hidden module. `anim` carries the
/// animation settings (spinner frame / pulse class), applied only while busy.
pub fn render(fmt: Format, snap: Option<&Snapshot>, anim: Anim) -> String {
    match fmt {
        Format::Waybar => waybar::render(snap, anim),
        Format::I3blocks => i3blocks::render(snap, anim),
        Format::Polybar => polybar::render(snap, anim),
        Format::Eww => eww::render(snap, anim),
        Format::Plain => plain::render(snap, anim),
        Format::Json => json::render(snap),
    }
}

/// True when the snapshot's headline is the "working" state.
pub(crate) fn is_busy(snap: &Snapshot) -> bool {
    snap.summary.headline_kind == "busy"
}

/// None-safe `is_busy` for the watch loop.
pub fn is_busy_opt(snap: &Snapshot) -> bool {
    is_busy(snap)
}

/// Text shown on the bar: the headline (with optional spinner glyph prefix when
/// busy), or a compact count, or empty.
pub(crate) fn bar_text(snap: &Snapshot, anim: Anim) -> String {
    if snap.is_empty() {
        return String::new();
    }
    let base = if !snap.summary.headline.is_empty() {
        snap.summary.headline.clone()
    } else {
        let n = snap.sessions.len();
        if n == 1 {
            "1 session".to_string()
        } else {
            format!("{n} sessions")
        }
    };
    // Prefix the animated spinner glyph only while working.
    if is_busy(snap) {
        if let Some(g) = anim.glyph() {
            return format!("{g} {base}");
        }
    }
    base
}

/// Space-separated CSS classes for the module: the headline kind, plus the
/// agent mode (build/plan/…) when busy, plus `pulse` when pulse-animating.
/// Lets CSS color the label by OpenCode mode and animate opacity.
pub(crate) fn bar_classes(snap: &Snapshot, anim: Anim) -> String {
    let mut classes: Vec<String> = vec![snap.summary.headline_kind.clone()];
    if is_busy(snap) {
        if let Some(mode) = snap.summary.mode.as_deref() {
            if !mode.is_empty() {
                classes.push(sanitize_class(mode));
            }
        }
        if anim.pulse() {
            classes.push("pulse".to_string());
        }
    }
    classes.join(" ")
}

/// OpenCode agent colors (dark theme, from packages/ui/src/styles/theme.css):
/// build `#034cff`, plan `#a753ae`. Single source of truth shared by the
/// hex-emitting formatters (i3blocks, polybar, eww, tmux). Returns `None` for an
/// unknown/custom agent so consumers fall back to their default foreground.
pub(crate) fn mode_hex(mode: Option<&str>) -> Option<&'static str> {
    match mode {
        Some("build") => Some("#034cff"),
        Some("plan") => Some("#a753ae"),
        _ => None,
    }
}

/// CSS class names allow [A-Za-z0-9_-]; map anything else to '-'.
fn sanitize_class(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

/// Multi-line tooltip body listing every session (plain text; bar-specific
/// markup is applied by the caller).
pub(crate) fn tooltip_lines(snap: &Snapshot) -> Vec<String> {
    snap.sessions
        .iter()
        .map(|s| {
            let mode = s
                .mode
                .as_deref()
                .filter(|m| !m.is_empty())
                .map(|m| format!(" [{m}]"))
                .unwrap_or_default();
            format!(
                "{} {}{}  {}  ({})",
                s.glyph(),
                s.title,
                mode,
                s.detail,
                s.age_label
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{SessionEntry, Snapshot, Summary};
    use crate::spinner::{Anim, AnimateMode, SpinnerStyle};

    fn busy_snap(mode: Option<&str>) -> Snapshot {
        Snapshot {
            summary: Summary {
                total: 1,
                busy: 1,
                waiting: 0,
                idle: 0,
                headline: "Working".into(),
                headline_kind: "busy".into(),
                mode: mode.map(|m| m.into()),
            },
            sessions: vec![SessionEntry {
                id: "s1".into(),
                title: "do a thing".into(),
                status: "busy".into(),
                detail: "Working…".into(),
                updated: 0,
                age_label: "now".into(),
                mode: mode.map(|m| m.into()),
            }],
            at: 0,
        }
    }

    #[test]
    fn classes_include_mode_when_busy() {
        let s = busy_snap(Some("build"));
        assert_eq!(bar_classes(&s, Anim::default()), "busy build");
        let s = busy_snap(Some("plan"));
        assert_eq!(bar_classes(&s, Anim::default()), "busy plan");
    }

    #[test]
    fn classes_pulse_added_when_animating() {
        let s = busy_snap(Some("build"));
        let anim = Anim {
            mode: AnimateMode::Pulse,
            ..Anim::default()
        };
        assert_eq!(bar_classes(&s, anim), "busy build pulse");
    }

    #[test]
    fn glyph_prefixes_text_when_busy() {
        let s = busy_snap(None);
        let anim = Anim {
            mode: AnimateMode::Glyph,
            spinner: SpinnerStyle::Braille,
            tick: 0,
        };
        let t = bar_text(&s, anim);
        let frame0 = SpinnerStyle::Braille.frame(0);
        assert!(t.starts_with(&format!("{frame0} ")), "got: {t}");
        assert!(t.ends_with("Working"));
    }

    #[test]
    fn ring_frames_have_no_gap_at_top() {
        // The top edge (tick 3 = both inner-top columns) must render the
        // continuous-top comet, not the gapped 3-wide version.
        let comet_top = SpinnerStyle::RingComet.frame(3);
        assert_eq!(comet_top, "⠉⠉", "top edge should be continuous");
        // ring + ring-comet are 2 braille cells wide
        assert_eq!(SpinnerStyle::Ring.frame(0).chars().count(), 2);
        assert_eq!(SpinnerStyle::RingComet.frame(0).chars().count(), 2);
    }

    #[test]
    fn glyph_frame_advances_with_tick() {
        let s = busy_snap(None);
        let a0 = Anim {
            mode: AnimateMode::Glyph,
            spinner: SpinnerStyle::Braille,
            tick: 0,
        };
        let a1 = Anim {
            mode: AnimateMode::Glyph,
            spinner: SpinnerStyle::Braille,
            tick: 1,
        };
        assert_ne!(bar_text(&s, a0), bar_text(&s, a1));
    }

    #[test]
    fn no_glyph_when_not_busy() {
        let mut s = busy_snap(None);
        s.summary.headline_kind = "idle".into();
        s.summary.headline = "1 session".into();
        let anim = Anim {
            mode: AnimateMode::Glyph,
            spinner: SpinnerStyle::Shimmer,
            tick: 3,
        };
        assert_eq!(bar_text(&s, anim), "1 session");
    }

    #[test]
    fn waybar_emits_mode_class_and_color_path() {
        let s = busy_snap(Some("plan"));
        let out = waybar::render(Some(&s), Anim::default());
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["class"], "busy plan");
        assert_eq!(v["text"], "Working");
    }
}
