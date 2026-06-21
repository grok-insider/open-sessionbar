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
    Dots,
    Ring,
    RingComet,
}

impl SpinnerStyle {
    pub fn parse(s: &str) -> Option<SpinnerStyle> {
        match s {
            "braille" => Some(SpinnerStyle::Braille),
            "shimmer" => Some(SpinnerStyle::Shimmer),
            "dots" => Some(SpinnerStyle::Dots),
            "ring" => Some(SpinnerStyle::Ring),
            "ring-comet" | "ringcomet" => Some(SpinnerStyle::RingComet),
            _ => None,
        }
    }

    pub fn frames(self) -> &'static [&'static str] {
        match self {
            // Smooth single-dot orbit around the cell — each frame moves the lit
            // dot one position clockwise, so it reads as one dot circling.
            SpinnerStyle::Braille => &["⠈", "⠐", "⠠", "⢀", "⡀", "⠄", "⠂", "⠁"],
            // Full-cell rotation: the filled mass sweeps around the perimeter
            // (the classic smooth "loading" spinner).
            SpinnerStyle::Shimmer => &["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"],
            // Two opposite orbiting dots (denser circular motion).
            SpinnerStyle::Dots => &["⠋", "⠙", "⠚", "⠞", "⠖", "⠦", "⠴", "⠲", "⠳", "⠓"],
            // Hollow "0": a single dot orbits the perimeter of a 3-wide × 4-tall
            // dot grid (rendered as 3 braille cells), with the 2 center columns
            // always empty. See ring_frames() for generation.
            SpinnerStyle::Ring => ring_frames(),
            // Same hollow-0 ring, but a 3-dot comet (head + 2 trailing) orbits,
            // so the motion is more visible at small bar sizes.
            SpinnerStyle::RingComet => ring_comet_frames(),
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

/// Frames for the "ring" / hollow-0 spinner.
///
/// The shape is a 4-wide × 4-tall dot grid forming a hollow rectangle (the
/// center 2×2 is empty), rendered as two adjacent braille cells. Using the full
/// 4 columns keeps the top and bottom edges continuous across the cell seam (a
/// 3-wide version left a visible gap at the top-middle). A single dot travels
/// clockwise around the perimeter. Path:
///   (0,0)(1,0)(2,0)(3,0)(3,1)(3,2)(3,3)(2,3)(1,3)(0,3)(0,2)(0,1)
/// Precomputed (see scripts/spinners.py). Full outline: ⣏⣹
fn ring_frames() -> &'static [&'static str] {
    &[
        "⠁⠀", // (0,0) top-left
        "⠈⠀", // (1,0)
        "⠀⠁", // (2,0)
        "⠀⠈", // (3,0) top-right
        "⠀⠐", // (3,1)
        "⠀⠠", // (3,2)
        "⠀⢀", // (3,3) bottom-right
        "⠀⡀", // (2,3)
        "⢀⠀", // (1,3)
        "⡀⠀", // (0,3) bottom-left
        "⠄⠀", // (0,2)
        "⠂⠀", // (0,1)
    ]
}

/// Hollow-0 ring with a 4-dot comet (head + 3 trailing dots) orbiting the
/// perimeter — the most visible variant at small bar sizes. Same path as
/// ring_frames(). Precomputed (see scripts/spinners.py).
///
/// Note: a single waybar/GTK text module has ONE color, so the trailing dots
/// cannot be greyed out independently — braille glyphs are monochrome. The
/// "fade" is approximated by the comet shape (solid head, thinning tail). True
/// per-dot dimming is only possible in truecolor terminals (the `tui` popup).
fn ring_comet_frames() -> &'static [&'static str] {
    &[
        "⡇⠀", "⠏⠀", "⠋⠁", "⠉⠉", "⠈⠙", "⠀⠹", "⠀⢸", "⠀⣰", "⢀⣠", "⣀⣀", "⣄⡀", "⣆⠀",
    ]
}

/// The full static hollow-0 ring outline (all perimeter dots lit). Useful as a
/// non-animated busy marker or a backdrop. `⣏⣹`
#[allow(dead_code)]
pub const RING_OUTLINE: &str = "⣏⣹";
