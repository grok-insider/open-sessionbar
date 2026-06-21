//! Wire types for the snapshot served by the opencode-sessionbar plugin.
//!
//! These mirror `plugin/store.ts`. Kept permissive (serde defaults) so a
//! newer plugin adding fields never breaks an older binary.

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Snapshot {
    #[serde(default)]
    pub summary: Summary,
    #[serde(default)]
    pub sessions: Vec<SessionEntry>,
    #[serde(default)]
    pub at: i64,
}

impl Default for Snapshot {
    fn default() -> Self {
        Snapshot {
            summary: Summary::default(),
            sessions: Vec::new(),
            at: 0,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    #[serde(default)]
    pub total: u32,
    #[serde(default)]
    pub busy: u32,
    #[serde(default)]
    pub waiting: u32,
    #[serde(default)]
    pub idle: u32,
    #[serde(default)]
    pub headline: String,
    #[serde(default = "default_kind")]
    pub headline_kind: String,
    /// Agent of the headline session: "build" | "plan" | custom. None if unknown.
    #[serde(default)]
    pub mode: Option<String>,
}

fn default_kind() -> String {
    "idle".to_string()
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionEntry {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub detail: String,
    #[serde(default)]
    pub updated: i64,
    #[serde(default)]
    pub age_label: String,
    /// Agent of the latest assistant message: "build" | "plan" | custom.
    #[serde(default)]
    pub mode: Option<String>,
}

impl SessionEntry {
    /// A short status glyph shared by the text formatters and the TUI.
    pub fn glyph(&self) -> &'static str {
        match self.status.as_str() {
            "waiting" => "★",
            "busy" => "●",
            "done" => "✓",
            _ => "○",
        }
    }
}

impl Snapshot {
    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }
}
