//! Self-contained OpenCode plugin installer.
//!
//! The plugin (a TUI plugin) is embedded in this binary at build time. We write
//! it to the opencode plugins dir and register it in `tui.json`. No npm.
//!
//!   opensessions plugin install [--global|--project DIR]
//!   opensessions plugin update
//!   opensessions plugin uninstall
//!   opensessions plugin status

use std::fs;
use std::path::{Path, PathBuf};

// Embedded plugin sources — single source of truth is the repo's plugin/ dir.
const PKG_JSON: &str = include_str!("../plugin/package.json");
const STORE_TS: &str = include_str!("../plugin/store.ts");
const SNAPSHOT_TS: &str = include_str!("../plugin/snapshot.ts");
const SERVER_TS: &str = include_str!("../plugin/server.ts");
const TUI_TSX: &str = include_str!("../plugin/tui.tsx");

const PLUGIN_DIRNAME: &str = "opencode-sessionbar";
/// The spec we register in tui.json — a relative local-dir reference, which is
/// how opencode resolves directory plugins (via the package "exports"./tui").
const PLUGIN_SPEC: &str = "./plugins/opencode-sessionbar";

/// open-sessionbar version stamp (matches the crate), written alongside the
/// plugin so `status` can detect drift after a binary upgrade.
const VERSION: &str = env!("CARGO_PKG_VERSION");
const STAMP_FILE: &str = ".open-sessionbar-version";

struct Target {
    /// e.g. ~/.config/opencode  (global)  or  <project>/.opencode  (project)
    config_dir: PathBuf,
}

impl Target {
    fn plugins_dir(&self) -> PathBuf {
        self.config_dir.join("plugins").join(PLUGIN_DIRNAME)
    }
    fn tui_json(&self) -> PathBuf {
        self.config_dir.join("tui.json")
    }
}

fn global_target() -> Result<Target, String> {
    let base = dirs::config_dir().ok_or("cannot locate user config dir")?;
    Ok(Target {
        config_dir: base.join("opencode"),
    })
}

fn project_target(dir: &Path) -> Target {
    Target {
        config_dir: dir.join(".opencode"),
    }
}

const FILES: &[(&str, &str)] = &[
    ("package.json", PKG_JSON),
    ("store.ts", STORE_TS),
    ("snapshot.ts", SNAPSHOT_TS),
    ("server.ts", SERVER_TS),
    ("tui.tsx", TUI_TSX),
];

fn write_plugin_files(t: &Target) -> Result<(), String> {
    let dir = t.plugins_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
    for (name, body) in FILES {
        let path = dir.join(name);
        fs::write(&path, body).map_err(|e| format!("write {}: {e}", path.display()))?;
    }
    fs::write(dir.join(STAMP_FILE), VERSION).map_err(|e| format!("write version stamp: {e}"))?;
    Ok(())
}

/// Add PLUGIN_SPEC to tui.json's `.plugin` array if absent. Strict-JSON only;
/// aborts if the file appears to contain comments (JSONC) so we never corrupt a
/// hand-edited config. Backs up to tui.json.bak.
fn register_in_tui_json(t: &Target) -> Result<bool, String> {
    let path = t.tui_json();
    let raw = if path.exists() {
        fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?
    } else {
        String::from("{}\n")
    };

    if looks_like_jsonc(&raw) {
        return Err(format!(
            "{} appears to contain comments (JSONC); refusing to edit automatically.\n\
             Add \"{PLUGIN_SPEC}\" to the \"plugin\" array manually.",
            path.display()
        ));
    }

    let mut root: serde_json::Value = if raw.trim().is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_str(&raw).map_err(|e| format!("parse {}: {e}", path.display()))?
    };

    if !root.is_object() {
        return Err(format!("{} is not a JSON object", path.display()));
    }

    let obj = root.as_object_mut().unwrap();
    let arr = obj
        .entry("plugin")
        .or_insert_with(|| serde_json::Value::Array(vec![]));
    let arr = arr
        .as_array_mut()
        .ok_or_else(|| format!("{}: \"plugin\" is not an array", path.display()))?;

    if plugin_present(arr) {
        return Ok(false); // already registered
    }

    if path.exists() {
        let bak = path.with_extension("json.bak");
        fs::copy(&path, &bak).map_err(|e| format!("backup {}: {e}", bak.display()))?;
    }

    arr.push(serde_json::Value::String(PLUGIN_SPEC.to_string()));
    let pretty = serde_json::to_string_pretty(&root).map_err(|e| e.to_string())?;
    fs::write(&path, format!("{pretty}\n"))
        .map_err(|e| format!("write {}: {e}", path.display()))?;
    Ok(true)
}

/// True if the spec is already present, in either string or [spec, opts] form.
fn plugin_present(arr: &[serde_json::Value]) -> bool {
    arr.iter().any(|v| match v {
        serde_json::Value::String(s) => s == PLUGIN_SPEC || s == PLUGIN_DIRNAME,
        serde_json::Value::Array(t) => t
            .first()
            .and_then(|x| x.as_str())
            .map(|s| s == PLUGIN_SPEC || s == PLUGIN_DIRNAME)
            .unwrap_or(false),
        _ => false,
    })
}

fn unregister_from_tui_json(t: &Target) -> Result<bool, String> {
    let path = t.tui_json();
    if !path.exists() {
        return Ok(false);
    }
    let raw = fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    if looks_like_jsonc(&raw) {
        return Err(format!(
            "{} contains comments (JSONC); remove \"{PLUGIN_SPEC}\" manually.",
            path.display()
        ));
    }
    let mut root: serde_json::Value =
        serde_json::from_str(&raw).map_err(|e| format!("parse {}: {e}", path.display()))?;
    let changed = if let Some(arr) = root.get_mut("plugin").and_then(|p| p.as_array_mut()) {
        let before = arr.len();
        arr.retain(|v| match v {
            serde_json::Value::String(s) => s != PLUGIN_SPEC && s != PLUGIN_DIRNAME,
            serde_json::Value::Array(t) => t
                .first()
                .and_then(|x| x.as_str())
                .map(|s| s != PLUGIN_SPEC && s != PLUGIN_DIRNAME)
                .unwrap_or(true),
            _ => true,
        });
        arr.len() != before
    } else {
        false
    };
    if changed {
        let bak = path.with_extension("json.bak");
        let _ = fs::copy(&path, &bak);
        let pretty = serde_json::to_string_pretty(&root).map_err(|e| e.to_string())?;
        fs::write(&path, format!("{pretty}\n"))
            .map_err(|e| format!("write {}: {e}", path.display()))?;
    }
    Ok(changed)
}

/// Cheap heuristic: a `//` or `/*` outside of a string strongly suggests JSONC.
/// We only need to avoid clobbering a commented config, so false positives just
/// mean "edit manually" — acceptable.
fn looks_like_jsonc(s: &str) -> bool {
    let mut in_str = false;
    let mut escaped = false;
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if in_str {
            if escaped {
                escaped = false;
            } else if c == b'\\' {
                escaped = true;
            } else if c == b'"' {
                in_str = false;
            }
        } else if c == b'"' {
            in_str = true;
        } else if c == b'/' && i + 1 < bytes.len() && (bytes[i + 1] == b'/' || bytes[i + 1] == b'*')
        {
            return true;
        }
        i += 1;
    }
    false
}

fn resolve_target(args: &[String]) -> Result<Target, String> {
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--global" | "-g" => return global_target(),
            "--project" => {
                let dir = args.get(i + 1).ok_or("--project requires a directory")?;
                return Ok(project_target(Path::new(dir)));
            }
            _ => {}
        }
        i += 1;
    }
    global_target()
}

pub fn cmd(args: &[String]) -> Result<(), String> {
    let sub = args.first().map(String::as_str).unwrap_or("status");
    let rest = if args.is_empty() { &[][..] } else { &args[1..] };
    match sub {
        "install" => install(rest),
        "update" => update(rest),
        "uninstall" | "remove" => uninstall(rest),
        "status" => status(rest),
        other => Err(format!(
            "unknown: plugin {other}\nUSAGE: opensessions plugin <install|update|uninstall|status> [--global|--project DIR]"
        )),
    }
}

fn install(args: &[String]) -> Result<(), String> {
    let t = resolve_target(args)?;
    write_plugin_files(&t)?;
    let added = register_in_tui_json(&t)?;
    println!("plugin files installed to {}", t.plugins_dir().display());
    if added {
        println!("registered \"{PLUGIN_SPEC}\" in {}", t.tui_json().display());
    } else {
        println!("already registered in {}", t.tui_json().display());
    }
    println!("restart opencode to load it.");
    Ok(())
}

fn update(args: &[String]) -> Result<(), String> {
    let t = resolve_target(args)?;
    if !t.plugins_dir().exists() {
        return Err(format!(
            "not installed at {} — run `opensessions plugin install` first",
            t.plugins_dir().display()
        ));
    }
    write_plugin_files(&t)?;
    // Ensure registration too, in case tui.json was reset.
    let added = register_in_tui_json(&t)?;
    println!(
        "plugin updated to v{VERSION} at {}",
        t.plugins_dir().display()
    );
    if added {
        println!("re-registered in {}", t.tui_json().display());
    }
    println!("restart opencode to load the new version.");
    Ok(())
}

fn uninstall(args: &[String]) -> Result<(), String> {
    let t = resolve_target(args)?;
    let dir = t.plugins_dir();
    if dir.exists() {
        fs::remove_dir_all(&dir).map_err(|e| format!("remove {}: {e}", dir.display()))?;
        println!("removed {}", dir.display());
    } else {
        println!("no plugin files at {}", dir.display());
    }
    let removed = unregister_from_tui_json(&t)?;
    if removed {
        println!("unregistered from {}", t.tui_json().display());
    }
    println!("restart opencode to apply.");
    Ok(())
}

fn status(args: &[String]) -> Result<(), String> {
    let t = resolve_target(args)?;
    let dir = t.plugins_dir();
    let installed = dir.join("tui.tsx").exists();
    let stamp = fs::read_to_string(dir.join(STAMP_FILE)).ok();
    let registered = match fs::read_to_string(t.tui_json()) {
        Ok(raw) => serde_json::from_str::<serde_json::Value>(&raw)
            .ok()
            .and_then(|v| v.get("plugin").and_then(|p| p.as_array().cloned()))
            .map(|arr| plugin_present(&arr))
            .unwrap_or(false),
        Err(_) => false,
    };

    println!("open-sessionbar v{VERSION}");
    println!("config dir : {}", t.config_dir.display());
    println!(
        "installed  : {}{}",
        if installed { "yes" } else { "no" },
        match &stamp {
            Some(v) if installed => {
                let v = v.trim();
                if v == VERSION {
                    format!(" (v{v}, up to date)")
                } else {
                    format!(" (v{v}, binary is v{VERSION} — run `plugin update`)")
                }
            }
            _ => String::new(),
        }
    );
    println!("registered : {}", if registered { "yes" } else { "no" });

    // Is a server actually answering on the configured port right now?
    let port = std::env::var("OPENCODE_SESSIONBAR_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .filter(|p| *p > 0)
        .unwrap_or(4098);
    let live = crate::client::Client::new(port).healthy();
    println!(
        "server     : {} (127.0.0.1:{port})",
        if live { "live" } else { "not running" }
    );
    Ok(())
}
