//! Minimal language selection for texts produced by the backend
//! (error messages, the generated example config, terminal prompts).
//! The UI has its own translations; both follow the same rule:
//! config "language" wins, otherwise the system locale, otherwise English.

use std::sync::atomic::{AtomicU8, Ordering};

const AUTO: u8 = 0;
const EN: u8 = 1;
const DE: u8 = 2;

static OVERRIDE: AtomicU8 = AtomicU8::new(AUTO);

/// Called whenever the config is loaded or saved.
pub fn set_override(lang: &str) {
    let v = match lang {
        "en" => EN,
        "de" => DE,
        _ => AUTO,
    };
    OVERRIDE.store(v, Ordering::Relaxed);
}

fn system_is_german() -> bool {
    // Same precedence as glibc: LC_ALL > LC_MESSAGES > LANG
    ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .filter_map(|k| std::env::var(k).ok())
        .find(|v| !v.is_empty())
        .map(|v| v.to_ascii_lowercase().starts_with("de"))
        .unwrap_or(false)
}

pub fn is_german() -> bool {
    match OVERRIDE.load(Ordering::Relaxed) {
        EN => false,
        DE => true,
        _ => system_is_german(),
    }
}

/// Pick the English or German variant.
pub fn tr<'a>(en: &'a str, de: &'a str) -> &'a str {
    if is_german() { de } else { en }
}

#[cfg(test)]
pub static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn override_beats_system_language() {
        let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        set_override("de");
        assert_eq!(tr("Start", "Starten"), "Starten");
        set_override("en");
        assert_eq!(tr("Start", "Starten"), "Start");
        set_override("auto");
    }
}
