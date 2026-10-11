//! Tile configuration.
//!
//! Stored in ~/.config/svcdeck/apps.json and the single source of truth:
//! actions are always looked up by (app_id, action_id) in the file on disk,
//! never taken as a free-form command from the UI.

use crate::locale::tr;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub const APP_DIR_NAME: &str = "svcdeck";
/// Config directory of the predecessor "Local AI Hub" (migrated once)
const LEGACY_DIR_NAME: &str = "local-ai-hub";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ActionMode {
    /// Run in the background, show the output in the window
    #[default]
    Capture,
    /// Run in a separate terminal window (e.g. follow logs)
    Terminal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    pub id: String,
    pub label: String,
    /// Executed with `bash -lc` in the app's working directory
    pub command: String,
    #[serde(default)]
    pub mode: ActionMode,
    /// Ask before running (e.g. stop)
    #[serde(default)]
    pub confirm: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct App {
    pub id: String,
    pub name: String,
    /// Opened in the browser on left click. Empty = left click runs the
    /// first action instead.
    #[serde(default)]
    pub url: String,
    /// Absolute image path or an icon name from the icon theme.
    /// Empty = glyph, or the default icon.
    #[serde(default)]
    pub icon: String,
    /// Short text shown when no icon is set
    #[serde(default)]
    pub glyph: String,
    /// Working directory for all actions
    #[serde(default)]
    pub dir: String,
    /// host:port for the status dot (empty = derive from the URL)
    #[serde(default)]
    pub health: String,
    /// Status via command: exit code 0 = running. Takes precedence over
    /// `health`, e.g. `systemctl --user is-active --quiet foo.service`.
    #[serde(default)]
    pub health_command: String,
    #[serde(default)]
    pub actions: Vec<Action>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default = "default_title")]
    pub title: String,
    #[serde(default = "default_columns")]
    pub columns: u32,
    /// Terminal invocation for actions with mode=terminal; the command is
    /// appended as `bash -lc <command>`. Empty = detect automatically.
    #[serde(default)]
    pub terminal: Vec<String>,
    /// "auto" (HyDE colors or system light/dark), "light" or "dark"
    #[serde(default = "default_auto")]
    pub theme: String,
    /// "auto" (system language), "en" or "de"
    #[serde(default = "default_auto")]
    pub language: String,
    /// Ask GitHub for a newer release on start
    #[serde(default = "default_true")]
    pub update_check: bool,
    #[serde(default)]
    pub apps: Vec<App>,
}

fn default_title() -> String {
    "SvcDeck".into()
}

fn default_columns() -> u32 {
    2
}

fn default_auto() -> String {
    "auto".into()
}

fn default_true() -> bool {
    true
}

pub fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

fn xdg_config_home() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| home_dir().join(".config"))
}

pub fn config_dir() -> PathBuf {
    xdg_config_home().join(APP_DIR_NAME)
}

pub fn config_path() -> PathBuf {
    config_dir().join("apps.json")
}

fn act(id: &str, label: &str, command: &str, mode: ActionMode, confirm: bool) -> Action {
    Action {
        id: id.into(),
        label: label.into(),
        command: command.into(),
        mode,
        confirm,
    }
}

/// First-run config: one example tile that only uses commands available on
/// practically every Linux system, so new users see how an entry looks.
fn default_config() -> Config {
    use ActionMode::{Capture as C, Terminal as T};
    Config {
        title: default_title(),
        columns: default_columns(),
        terminal: Vec::new(),
        theme: default_auto(),
        language: default_auto(),
        update_check: true,
        apps: vec![App {
            id: "system".into(),
            name: tr("System (example)", "System (Beispiel)").into(),
            url: String::new(),
            icon: "utilities-system-monitor".into(),
            glyph: String::new(),
            dir: String::new(),
            health: String::new(),
            // Always "running": the machine itself is up
            health_command: "true".into(),
            actions: vec![
                act(
                    "status",
                    tr("Overview", "Übersicht"),
                    "uptime; echo; free -h; echo; df -h /",
                    C,
                    false,
                ),
                act("processes", tr("Processes", "Prozesse"), "top", T, false),
                act(
                    "logs",
                    tr("System logs (systemd)", "System-Logs (systemd)"),
                    "journalctl -f -n 50",
                    T,
                    false,
                ),
            ],
        }],
    }
}

/// One-time takeover of the "Local AI Hub" config (rename to SvcDeck).
/// The old directory stays untouched.
fn migrate_legacy() -> Result<Option<Config>, String> {
    let old_dir = xdg_config_home().join(LEGACY_DIR_NAME);
    let old_file = old_dir.join("apps.json");
    if !old_file.is_file() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&old_file).map_err(|e| format!("{}: {e}", old_file.display()))?;
    let mut cfg: Config =
        serde_json::from_str(&raw).map_err(|e| format!("{}: {e}", old_file.display()))?;

    let new_dir = config_dir();
    fs::create_dir_all(&new_dir).map_err(|e| e.to_string())?;

    // Copy icons and point the entries at the new location
    let old_icons = old_dir.join("icons");
    if old_icons.is_dir() {
        let new_icons = new_dir.join("icons");
        fs::create_dir_all(&new_icons).map_err(|e| e.to_string())?;
        for entry in fs::read_dir(&old_icons)
            .map_err(|e| e.to_string())?
            .flatten()
        {
            let target = new_icons.join(entry.file_name());
            if entry.path().is_file() && !target.exists() {
                fs::copy(entry.path(), &target).map_err(|e| e.to_string())?;
            }
        }
    }
    let old_prefix = old_dir.to_string_lossy().into_owned();
    let new_prefix = new_dir.to_string_lossy().into_owned();
    for app in &mut cfg.apps {
        if app.icon.starts_with(&old_prefix) {
            app.icon = app.icon.replacen(&old_prefix, &new_prefix, 1);
        }
    }
    // "Local AI" was the old built-in default, not a user choice
    if cfg.title == "Local AI" {
        cfg.title = default_title();
    }
    for part in &mut cfg.terminal {
        if part == "local-ai-hub-term" {
            *part = "svcdeck-term".into();
        }
    }
    save(&cfg)?;
    Ok(Some(cfg))
}

/// Loads the config; creates the example (or migrates) on first start.
pub fn load() -> Result<Config, String> {
    let path = config_path();
    let cfg = if path.exists() {
        let raw = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        serde_json::from_str(&raw).map_err(|e| {
            format!(
                "{} {}: {e}",
                path.display(),
                tr("is invalid", "ist ungültig")
            )
        })?
    } else if let Some(cfg) = migrate_legacy()? {
        cfg
    } else {
        let cfg = default_config();
        save(&cfg)?;
        cfg
    };
    crate::locale::set_override(&cfg.language);
    Ok(cfg)
}

fn validate(cfg: &Config) -> Result<(), String> {
    let mut ids = std::collections::HashSet::new();
    for app in &cfg.apps {
        if app.id.trim().is_empty() {
            return Err(format!("{}: {}", app.name, tr("missing ID", "ID fehlt")));
        }
        if !ids.insert(app.id.as_str()) {
            return Err(format!(
                "{} '{}'",
                tr("Duplicate app ID", "Doppelte App-ID"),
                app.id
            ));
        }
        let mut aids = std::collections::HashSet::new();
        for a in &app.actions {
            if a.id.trim().is_empty() || !aids.insert(a.id.as_str()) {
                return Err(format!(
                    "{}: {} '{}'",
                    app.name,
                    tr(
                        "action ID missing or duplicate",
                        "Aktions-ID fehlt oder ist doppelt"
                    ),
                    a.id
                ));
            }
        }
    }
    Ok(())
}

/// Atomic write (temp file + rename), mode 0600.
pub fn save(cfg: &Config) -> Result<(), String> {
    validate(cfg)?;
    let dir = config_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    write_private(
        &config_path(),
        serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?,
    )?;
    crate::locale::set_override(&cfg.language);
    Ok(())
}

fn write_private(path: &Path, json: String) -> Result<(), String> {
    use std::os::unix::fs::OpenOptionsExt;
    let tmp = path.with_extension("json.tmp");
    {
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&tmp)
            .map_err(|e| format!("{}: {e}", tmp.display()))?;
        f.write_all(json.as_bytes()).map_err(|e| e.to_string())?;
        f.write_all(b"\n").map_err(|e| e.to_string())?;
    }
    fs::rename(&tmp, path).map_err(|e| format!("{}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_config_is_valid() {
        let cfg = default_config();
        validate(&cfg).expect("example config validates");
        assert_eq!(cfg.apps.len(), 1);
        assert!(
            cfg.apps[0].url.is_empty(),
            "example has no URL, left click runs the first action"
        );
    }

    #[test]
    fn rejects_duplicate_ids() {
        let mut cfg = default_config();
        cfg.apps.push(cfg.apps[0].clone());
        assert!(validate(&cfg).is_err());

        let mut cfg = default_config();
        let first = cfg.apps[0].actions[0].clone();
        cfg.apps[0].actions.push(first);
        assert!(validate(&cfg).is_err());
    }

    #[test]
    fn old_files_without_new_fields_still_load() {
        // Shape of a "Local AI Hub" config: no theme/language/healthCommand
        let raw = r#"{"title":"Local AI","columns":2,"terminal":["kitty"],
            "apps":[{"id":"a","name":"A","url":"http://127.0.0.1:1","actions":[]}]}"#;
        let cfg: Config = serde_json::from_str(raw).expect("parses");
        assert_eq!(cfg.theme, "auto");
        assert_eq!(cfg.language, "auto");
        assert!(cfg.update_check, "update check defaults to on");
        assert!(cfg.apps[0].health_command.is_empty());
    }

    #[test]
    fn migrates_legacy_config_once() {
        let _guard = crate::locale::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let root = std::env::temp_dir().join(format!("svcdeck-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let legacy = root.join(LEGACY_DIR_NAME);
        fs::create_dir_all(legacy.join("icons")).unwrap();
        fs::write(legacy.join("icons/a.svg"), "<svg/>").unwrap();
        let icon = legacy.join("icons/a.svg").to_string_lossy().into_owned();
        let raw = format!(
            r#"{{"title":"Local AI","columns":2,"terminal":["kitty","--class","local-ai-hub-term"],
               "apps":[{{"id":"a","name":"A","url":"","icon":"{icon}","actions":[]}}]}}"#
        );
        fs::write(legacy.join("apps.json"), raw).unwrap();

        // SAFETY: only this test touches XDG_CONFIG_HOME, serialized by TEST_LOCK
        unsafe { std::env::set_var("XDG_CONFIG_HOME", &root) };
        let cfg = load().expect("migrates");
        unsafe { std::env::remove_var("XDG_CONFIG_HOME") };

        assert_eq!(cfg.title, "SvcDeck", "old default title is replaced");
        assert_eq!(cfg.terminal[2], "svcdeck-term");
        let new_icon = root.join(APP_DIR_NAME).join("icons/a.svg");
        assert_eq!(cfg.apps[0].icon, new_icon.to_string_lossy());
        assert!(new_icon.is_file(), "icon copied");
        assert!(
            root.join(APP_DIR_NAME).join("apps.json").is_file(),
            "new config written"
        );
        assert!(
            legacy.join("apps.json").is_file(),
            "old config left in place"
        );
        let _ = fs::remove_dir_all(&root);
    }
}
