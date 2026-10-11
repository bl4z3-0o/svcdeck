//! Tile icons.
//!
//! Accepts an absolute image path, `~/...`, or an icon name from the active
//! icon theme (following its `Inherits=` chain, then hicolor). The image is
//! sent to the UI as a data: URL, so the webview needs no file access.

use crate::locale::tr;
use base64::Engine;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const MAX_BYTES: u64 = 4 * 1024 * 1024;

fn mime(path: &Path) -> Option<&'static str> {
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "png" => Some("image/png"),
        "svg" => Some("image/svg+xml"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "webp" => Some("image/webp"),
        "gif" => Some("image/gif"),
        "ico" => Some("image/x-icon"),
        "avif" => Some("image/avif"),
        _ => None,
    }
}

fn icon_roots() -> Vec<PathBuf> {
    let home = crate::config::home_dir();
    let mut roots = vec![home.join(".local/share/icons"), home.join(".icons")];
    let data_dirs = std::env::var("XDG_DATA_DIRS").unwrap_or_else(|_| "/usr/local/share:/usr/share".into());
    for d in data_dirs.split(':').filter(|d| !d.is_empty()) {
        let p = PathBuf::from(d).join("icons");
        if !roots.contains(&p) {
            roots.push(p);
        }
    }
    roots
}

/// Name of the active icon theme (GNOME/GTK settings, then KDE).
fn current_theme() -> Option<String> {
    let clean = |s: &str| {
        let t = s.trim().trim_matches(['\'', '"']).trim().to_string();
        (!t.is_empty()).then_some(t)
    };
    if let Ok(out) = std::process::Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "icon-theme"])
        .output()
        && out.status.success()
            && let Some(t) = clean(&String::from_utf8_lossy(&out.stdout)) {
                return Some(t);
            }
    let cfg = crate::config::home_dir().join(".config");
    for (file, key) in [("gtk-3.0/settings.ini", "gtk-icon-theme-name"), ("kdeglobals", "Theme")] {
        if let Ok(raw) = std::fs::read_to_string(cfg.join(file)) {
            for line in raw.lines() {
                if let Some((k, v)) = line.split_once('=')
                    && k.trim() == key
                        && let Some(t) = clean(v) {
                            return Some(t);
                        }
            }
        }
    }
    None
}

fn theme_dirs(theme: &str) -> Vec<PathBuf> {
    icon_roots().into_iter().map(|r| r.join(theme)).filter(|p| p.is_dir()).collect()
}

fn inherits(theme: &str) -> Vec<String> {
    for dir in theme_dirs(theme) {
        if let Ok(raw) = std::fs::read_to_string(dir.join("index.theme")) {
            for line in raw.lines() {
                if let Some(v) = line.strip_prefix("Inherits=") {
                    return v.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                }
            }
        }
    }
    Vec::new()
}

/// Active theme, its parents, then common fallbacks.
fn theme_chain() -> &'static [String] {
    static CHAIN: OnceLock<Vec<String>> = OnceLock::new();
    CHAIN.get_or_init(|| {
        let mut chain: Vec<String> = Vec::new();
        let mut queue: Vec<String> = current_theme().into_iter().collect();
        while let Some(t) = queue.pop() {
            if chain.contains(&t) || chain.len() > 12 {
                continue;
            }
            let parents = inherits(&t);
            chain.push(t);
            queue.extend(parents.into_iter().rev());
        }
        for t in ["hicolor", "Adwaita", "AdwaitaLegacy", "breeze", "Papirus"] {
            if !chain.iter().any(|c| c == t) {
                chain.push(t.into());
            }
        }
        chain
    })
}

/// Score a candidate: scalable SVG beats everything, otherwise larger is better.
fn score(path: &Path) -> u32 {
    let s = path.to_string_lossy();
    // Symbolic icons are monochrome outlines, only a last resort
    if s.contains("symbolic") {
        return 0;
    }
    if s.contains("scalable") {
        return 10_000;
    }
    s.split('/')
        .filter_map(|part| part.split(['x', '@']).next()?.parse::<u32>().ok())
        .max()
        .unwrap_or(1)
}

fn lookup_by_name(name: &str) -> Option<PathBuf> {
    // Plain names only, no path components
    if name.is_empty() || name.contains('/') || name.contains("..") {
        return None;
    }
    let files = [format!("{name}.svg"), format!("{name}.png")];
    for theme in theme_chain() {
        let mut best: Option<(u32, PathBuf)> = None;
        for dir in theme_dirs(theme) {
            // Covers both layouts: <size>/<context>/ (Papirus, Adwaita) and
            // <context>/<size>/ (Breeze)
            let Ok(level1) = std::fs::read_dir(&dir) else { continue };
            for a in level1.flatten().filter(|e| e.path().is_dir()) {
                let Ok(level2) = std::fs::read_dir(a.path()) else { continue };
                for b in level2.flatten().filter(|e| e.path().is_dir()) {
                    for f in &files {
                        let p = b.path().join(f);
                        if p.is_file() {
                            let s = score(&p);
                            if best.as_ref().is_none_or(|(bs, _)| s > *bs) {
                                best = Some((s, p));
                            }
                        }
                    }
                }
            }
        }
        if let Some((_, p)) = best {
            return Some(p);
        }
    }
    files
        .iter()
        .map(|f| PathBuf::from("/usr/share/pixmaps").join(f))
        .find(|p| p.is_file())
}

pub fn load(spec: &str) -> Result<String, String> {
    let spec = spec.trim();
    let path = if spec.starts_with('/') {
        PathBuf::from(spec)
    } else if let Some(rest) = spec.strip_prefix("~/") {
        crate::config::home_dir().join(rest)
    } else {
        lookup_by_name(spec)
            .ok_or_else(|| format!("{} '{spec}'", tr("Icon not found:", "Icon nicht gefunden:")))?
    };
    let mime = mime(&path)
        .ok_or_else(|| format!("{} {}", tr("Unsupported image format:", "Bildformat nicht unterstützt:"), path.display()))?;
    let meta = std::fs::metadata(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    if meta.len() > MAX_BYTES {
        return Err(format!("{} > 4 MB", path.display()));
    }
    let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(format!(
        "data:{mime};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
}
