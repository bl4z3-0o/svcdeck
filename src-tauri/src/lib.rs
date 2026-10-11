mod actions;
mod config;
mod icons;
mod locale;
mod terminal;
mod theme;

use serde::Serialize;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;
use tauri::Manager;

/// Wayland app_id / X11 class of the main window (= binary name)
const WINDOW_CLASS: &str = "svcdeck";

#[tauri::command]
fn load_config() -> Result<config::Config, String> {
    config::load()
}

#[tauri::command]
fn save_config(config: config::Config) -> Result<(), String> {
    config::save(&config)
}

#[tauri::command]
fn config_path() -> String {
    config::config_path().to_string_lossy().into_owned()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppInfo {
    version: String,
    /// "package" (installed by a package manager under /usr), "user"
    /// (install.sh, ~/.local) or "other" (e.g. a development build)
    install: &'static str,
}

/// Version and how SvcDeck was installed, for the update hint.
#[tauri::command]
fn app_info(app: tauri::AppHandle) -> AppInfo {
    let exe = std::env::current_exe().unwrap_or_default();
    let home = config::home_dir();
    let install = if exe.starts_with("/usr/") {
        "package"
    } else if exe.starts_with(home.join(".local")) {
        "user"
    } else {
        "other"
    };
    AppInfo {
        version: app.package_info().version.to_string(),
        install,
    }
}

/// Terminal that would be used when none is configured (shown as hint).
#[tauri::command]
fn detected_terminal() -> Option<String> {
    terminal::detect().map(|t| t.join(" "))
}

#[tauri::command]
async fn run_action(app_id: String, action_id: String) -> Result<actions::ActionResult, String> {
    tauri::async_runtime::spawn_blocking(move || actions::run(&app_id, &action_id))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
fn get_theme() -> theme::Theme {
    theme::current()
}

#[tauri::command]
async fn load_icon(spec: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || icons::load(&spec))
        .await
        .map_err(|e| e.to_string())?
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Health {
    id: String,
    /// None = no status check configured for this app
    running: Option<bool>,
}

/// host:port from the explicit setting or the URL (http/https only)
fn health_addr(app: &config::App) -> Option<String> {
    let h = app.health.trim();
    if !h.is_empty() {
        return Some(h.to_string());
    }
    let url = app.url.trim();
    let (scheme, rest) = url.split_once("://")?;
    let default_port = match scheme {
        "http" => 80,
        "https" => 443,
        _ => return None,
    };
    let authority = rest.split(['/', '?', '#']).next()?.rsplit('@').next()?;
    if authority.is_empty() {
        return None;
    }
    // [IPv6]:port, host:port or host
    let has_port = authority
        .rsplit_once(':')
        .is_some_and(|(_, p)| p.chars().all(|c| c.is_ascii_digit()))
        && !authority.ends_with(']');
    Some(if has_port {
        authority.to_string()
    } else {
        format!("{authority}:{default_port}")
    })
}

/// Status of every app: command (exit 0) > port > nothing.
#[tauri::command]
async fn check_health() -> Result<Vec<Health>, String> {
    let cfg = config::load()?;
    let handles: Vec<_> = cfg
        .apps
        .into_iter()
        .map(|app| {
            tauri::async_runtime::spawn_blocking(move || {
                let running = if !app.health_command.trim().is_empty() {
                    Some(actions::check_command(
                        &app,
                        &app.health_command,
                        Duration::from_secs(3),
                    ))
                } else {
                    health_addr(&app).map(|addr| {
                        addr.to_socket_addrs().is_ok_and(|mut it| {
                            it.any(|sa| {
                                TcpStream::connect_timeout(&sa, Duration::from_millis(400)).is_ok()
                            })
                        })
                    })
                };
                Health {
                    id: app.id,
                    running,
                }
            })
        })
        .collect();
    let mut out = Vec::with_capacity(handles.len());
    for h in handles {
        if let Ok(v) = h.await {
            out.push(v);
        }
    }
    Ok(out)
}

/// Resizes the window (logical pixels).
///
/// Hyprland does not reliably honor client resize requests for floating
/// windows, so under Hyprland the size is additionally set via hyprctl:
/// Lua syntax first (0.55+), then the classic dispatcher.
#[tauri::command]
async fn fit_window(window: tauri::WebviewWindow, width: f64, height: f64) -> Result<(), String> {
    let _ = window.set_size(tauri::LogicalSize::new(width, height));
    if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_none() {
        return Ok(());
    }
    let (w, h) = (width.round() as i64, height.round() as i64);
    tauri::async_runtime::spawn_blocking(move || {
        let selector = format!("class:^({WINDOW_CLASS})$");
        let lua = format!(
            "hl.dsp.window.resize({{window = \"{selector}\", x = {w}, y = {h}, exact = true}})"
        );
        let ok = std::process::Command::new("hyprctl")
            .args(["dispatch", &lua])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "ok")
            .unwrap_or(false);
        if !ok {
            let _ = std::process::Command::new("hyprctl")
                .args([
                    "dispatch",
                    "resizewindowpixel",
                    &format!("exact {w} {h},{selector}"),
                ])
                .output();
        }
    })
    .await
    .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Second launch (e.g. from the app launcher again): show the window
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.show();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            load_config,
            save_config,
            config_path,
            detected_terminal,
            app_info,
            run_action,
            get_theme,
            load_icon,
            check_health,
            fit_window
        ])
        .setup(|app| {
            // Loads (or migrates/creates) the config once, which also sets
            // the backend language before anything else runs
            let _ = config::load();
            let _ = actions::ensure_askpass();
            theme::watch(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("SvcDeck failed to start");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app(url: &str, health: &str) -> config::App {
        config::App {
            id: "a".into(),
            name: "A".into(),
            url: url.into(),
            icon: String::new(),
            glyph: String::new(),
            dir: String::new(),
            health: health.into(),
            health_command: String::new(),
            actions: Vec::new(),
        }
    }

    #[test]
    fn health_address_from_url() {
        assert_eq!(
            health_addr(&app("http://127.0.0.1:3000/x", "")).as_deref(),
            Some("127.0.0.1:3000")
        );
        assert_eq!(
            health_addr(&app("http://localhost", "")).as_deref(),
            Some("localhost:80")
        );
        assert_eq!(
            health_addr(&app("https://example.org/a?b#c", "")).as_deref(),
            Some("example.org:443")
        );
        assert_eq!(
            health_addr(&app("http://user:pw@host:8080", "")).as_deref(),
            Some("host:8080")
        );
        assert_eq!(
            health_addr(&app("http://[::1]:9000/", "")).as_deref(),
            Some("[::1]:9000")
        );
        assert_eq!(
            health_addr(&app("http://[::1]/", "")).as_deref(),
            Some("[::1]:80")
        );
    }

    #[test]
    fn explicit_health_wins_and_odd_urls_give_none() {
        assert_eq!(
            health_addr(&app("http://a:1", "b:2")).as_deref(),
            Some("b:2")
        );
        assert_eq!(health_addr(&app("", "")), None);
        assert_eq!(health_addr(&app("ftp://host", "")), None);
        assert_eq!(health_addr(&app("not a url", "")), None);
    }
}
