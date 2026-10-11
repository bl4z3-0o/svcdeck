//! HyDE/wallbash colors (optional).
//!
//! On every theme or wallpaper change HyDE writes the palette to
//! ~/.cache/hyde/wall.dcol (a symlink to the current .dcol file). A
//! background thread polls the file and emits "theme-changed" to the UI.
//! Without HyDE `available` is false and the UI uses its own palette.

use serde::Serialize;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Theme {
    /// false when no HyDE palette was found
    pub available: bool,
    /// "dark" or "light"
    pub mode: String,
    /// e.g. "pry1" -> "#C8A5C8", "1xa4" -> "#C27AC2"
    pub colors: BTreeMap<String, String>,
}

fn dcol_path() -> PathBuf {
    let cache = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| crate::config::home_dir().join(".cache"));
    cache.join("hyde").join("wall.dcol")
}

fn parse(raw: &str) -> Theme {
    let mut colors = BTreeMap::new();
    let mut mode = String::from("dark");
    for line in raw.lines() {
        let Some((key, val)) = line.split_once('=') else {
            continue;
        };
        let Some(key) = key.trim().strip_prefix("dcol_") else {
            continue;
        };
        let val = val.trim().trim_matches('"');
        if key == "mode" {
            mode = val.to_string();
        } else if !key.ends_with("_rgba")
            && val.len() == 6
            && val.chars().all(|c| c.is_ascii_hexdigit())
        {
            colors.insert(key.to_string(), format!("#{val}"));
        }
    }
    Theme {
        available: !colors.is_empty(),
        mode,
        colors,
    }
}

pub fn current() -> Theme {
    match std::fs::read_to_string(dcol_path()) {
        Ok(raw) => parse(&raw),
        Err(_) => Theme {
            available: false,
            mode: "dark".into(),
            colors: BTreeMap::new(),
        },
    }
}

pub fn watch(app: AppHandle) {
    std::thread::spawn(move || {
        let mut last = current();
        loop {
            std::thread::sleep(Duration::from_millis(1500));
            let now = current();
            if now != last {
                let _ = app.emit("theme-changed", &now);
                last = now;
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex_colors_and_mode() {
        let raw = "dcol_mode=\"light\"\ndcol_pry1=\"C8A5C8\"\ndcol_pry1_rgba=\"rgba(200,165,200,\\1)\"\ndcol_1xa4=\"C27AC2\"\n";
        let t = parse(raw);
        assert!(t.available);
        assert_eq!(t.mode, "light");
        assert_eq!(t.colors.get("pry1").map(String::as_str), Some("#C8A5C8"));
        assert_eq!(t.colors.get("1xa4").map(String::as_str), Some("#C27AC2"));
        // rgba variants are not colors for the UI
        assert!(!t.colors.contains_key("pry1_rgba"));
    }

    #[test]
    fn ignores_garbage_and_reports_unavailable() {
        let t = parse("hello\ndcol_pry1=\"nothex\"\nfoo=bar\n");
        assert!(!t.available);
        assert_eq!(t.mode, "dark");
    }
}
