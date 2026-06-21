//! Spinner frame sets + animation settings for the "Working" state.
//!
//! Two animation styles, both selectable via CLI flags (which the waybar `exec`
//! string carries, so the choice lives in your bar config):
//!   --animate glyph   animate the bar TEXT (a spinner frame prefix); needs `watch`
//!   --animate pulse   emit a CSS class so the bar opacity-pulses (works under `bar`)
//!   --animate off     no animation (default)
//!
//! Spinner frame sets for `glyph`:
//!   braille   ⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏   (OpenCode-style)
//!   shimmer   ·⢀⢄⢆⢇⢧⢷⣷⢷⢧⢇⢆⢄·  (amber dot shimmer, à la the search animation)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimateMode {
    Off,
    Glyph,
    Pulse,
}

impl AnimateMode {
    pub fn parse(s: &str) -> Option<AnimateMode> {
        match s {
            "off" | "none" => Some(AnimateMode::Off),
            "glyph" => Some(AnimateMode::Glyph),
            "pulse" => Some(AnimateMode::Pulse),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpinnerStyle {
    Braille,
    Shimmer,
}

impl SpinnerStyle {
    pub fn parse(s: &str) -> Option<SpinnerStyle> {
        match s {
            "braille" => Some(SpinnerStyle::Braille),
            "shimmer" => Some(SpinnerStyle::Shimmer),
            _ => None,
        }
    }

    pub fn frames(self) -> &'static [&'static str] {
        match self {
            SpinnerStyle::Braille => &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            SpinnerStyle::Shimmer => &["·", "⢀", "⢄", "⢆", "⢇", "⢧", "⢷", "⣷", "⢷", "⢧", "⢇", "⢆", "⢄"],
        }
    }

    /// The frame for a given tick counter (wraps).
    pub fn frame(self, tick: usize) -> &'static str {
        let f = self.frames();
        f[tick % f.len()]
    }
}

/// Animation settings resolved from CLI flags, passed into the formatters.
#[derive(Debug, Clone, Copy)]
pub struct Anim {
    pub mode: AnimateMode,
    pub spinner: SpinnerStyle,
    /// Current frame counter (advanced by the `watch` ticker). Ignored unless
    /// mode == Glyph.
    pub tick: usize,
}

impl Default for Anim {
    fn default() -> Self {
        Anim {
            mode: AnimateMode::Off,
            spinner: SpinnerStyle::Braille,
            tick: 0,
        }
    }
}

impl Anim {
    /// The spinner glyph to prefix to the bar text while busy, if any.
    pub fn glyph(&self) -> Option<&'static str> {
        if self.mode == AnimateMode::Glyph {
            Some(self.spinner.frame(self.tick))
        } else {
            None
        }
    }

    /// Whether to emit the `pulse` CSS class.
    pub fn pulse(&self) -> bool {
        self.mode == AnimateMode::Pulse
    }
}
