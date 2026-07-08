//! HTTP/SSE client for the opencode-sessionbar plugin on 127.0.0.1:<port>.

use std::io::{BufRead, BufReader};
use std::time::Duration;

use crate::model::Snapshot;

pub struct Client {
    base: String,
    /// Shared blocking client for short requests (snapshot / health).
    http: reqwest::blocking::Client,
}

impl Client {
    pub fn new(port: u16) -> Self {
        let http = reqwest::blocking::Client::builder()
            .timeout(Duration::from_millis(1000))
            .build()
            // Builder only fails on bad TLS config; fall back to default.
            .unwrap_or_else(|_| reqwest::blocking::Client::new());
        Client {
            base: format!("http://127.0.0.1:{port}"),
            http,
        }
    }

    /// Fetch one snapshot. Returns None if the plugin isn't reachable.
    pub fn snapshot(&self) -> Option<Snapshot> {
        let resp = self
            .http
            .get(format!("{}/sessions", self.base))
            .send()
            .ok()?;
        if !resp.status().is_success() {
            return None;
        }
        resp.json::<Snapshot>().ok()
    }

    /// Liveness probe: true if /health responds with our plugin name.
    pub fn healthy(&self) -> bool {
        let resp = match self.http.get(format!("{}/health", self.base)).send() {
            Ok(r) => r,
            Err(_) => return false,
        };
        if !resp.status().is_success() {
            return false;
        }
        match resp.json::<serde_json::Value>() {
            Ok(v) => v.get("name").and_then(|n| n.as_str()) == Some("opencode-sessionbar"),
            Err(_) => false,
        }
    }

    /// Open the SSE stream and invoke `on_snapshot` for each pushed frame.
    /// Blocks until the stream ends or the callback returns `false`.
    /// Returns Err if the stream could not be opened.
    pub fn stream<F>(&self, mut on_snapshot: F) -> Result<(), String>
    where
        F: FnMut(Snapshot) -> bool,
    {
        // Dedicated client: long-lived stream (server keep-alive pings) with a
        // bounded connect timeout so a blackholed port fails into reconnect.
        let agent = reqwest::blocking::Client::builder()
            .connect_timeout(Duration::from_secs(2))
            .build()
            .map_err(|e| e.to_string())?;
        let resp = agent
            .get(format!("{}/sessions/stream", self.base))
            .send()
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("stream returned HTTP {}", resp.status()));
        }
        let reader = BufReader::new(resp);
        // SSE: lines like `event: snapshot` then `data: <json>` then blank line.
        // We only care about `data:` payloads; each is a complete JSON snapshot.
        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => break, // connection dropped
            };
            if let Some(payload) = line
                .strip_prefix("data: ")
                .or_else(|| line.strip_prefix("data:"))
            {
                let payload = payload.trim_start();
                if payload.is_empty() {
                    continue;
                }
                if let Ok(snap) = serde_json::from_str::<Snapshot>(payload) {
                    if !on_snapshot(snap) {
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}
