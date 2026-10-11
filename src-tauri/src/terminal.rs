//! Terminal detection for actions with mode=terminal.
//!
//! Terminals disagree on how a command is passed (`-e`, `--`, `-x`, nothing),
//! so each known terminal has its own prefix. The window class/app-id is set
//! where the terminal supports it, so tiling compositors can float it.

use std::path::Path;

pub const TERM_CLASS: &str = "svcdeck-term";

/// Arguments placed between the program and `bash -lc <command>`.
fn prefix_for(program: &str) -> Option<Vec<&'static str>> {
    let name = Path::new(program).file_name()?.to_str()?;
    Some(match name {
        "kitty" => vec!["--class", TERM_CLASS],
        "foot" => vec!["--app-id", TERM_CLASS],
        "alacritty" => vec!["--class", TERM_CLASS, "-e"],
        "wezterm" => vec!["start", "--class", TERM_CLASS, "--"],
        // Ghostty requires a reverse-DNS style class
        "ghostty" => vec!["--class=io.github.bl4z3_0o.SvcDeck.term", "-e"],
        "konsole" => vec!["-e"],
        "gnome-terminal" | "kgx" | "ptyxis" => vec!["--"],
        "xfce4-terminal" => vec!["-x"],
        "tilix" | "terminator" | "lxterminal" | "mate-terminal" => vec!["-e"],
        "xterm" | "urxvt" => vec!["-class", TERM_CLASS, "-e"],
        "xdg-terminal-exec" => vec![],
        _ => return None,
    })
}

fn in_path(program: &str) -> bool {
    if program.contains('/') {
        return Path::new(program).is_file();
    }
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|d| d.join(program).is_file()))
        .unwrap_or(false)
}

/// Order of preference when nothing is configured
const CANDIDATES: [&str; 14] = [
    "kitty",
    "foot",
    "alacritty",
    "wezterm",
    "ghostty",
    "konsole",
    "ptyxis",
    "gnome-terminal",
    "kgx",
    "xfce4-terminal",
    "tilix",
    "xdg-terminal-exec",
    "terminator",
    "xterm",
];

/// Full invocation without the trailing `bash -lc <command>`.
pub fn detect() -> Option<Vec<String>> {
    // $TERMINAL is the user's explicit choice
    if let Ok(t) = std::env::var("TERMINAL") {
        let t = t.trim();
        if !t.is_empty() && in_path(t) {
            let prefix = prefix_for(t).unwrap_or_else(|| vec!["-e"]);
            return Some(std::iter::once(t.to_string()).chain(prefix.into_iter().map(String::from)).collect());
        }
    }
    CANDIDATES.iter().find(|c| in_path(c)).map(|c| {
        std::iter::once(c.to_string())
            .chain(prefix_for(c).unwrap_or_default().into_iter().map(String::from))
            .collect()
    })
}

/// The configured invocation, or the detected one if empty.
pub fn resolve(configured: &[String]) -> Option<Vec<String>> {
    if configured.is_empty() { detect() } else { Some(configured.to_vec()) }
}
