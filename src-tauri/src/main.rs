// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // WebKitGTK + NVIDIA on Wayland: without this the window often stays
    // blank or flickers. Set before any thread starts, unless already given.
    //
    // The window is deliberately NOT transparent: WebKitGTK repaints
    // transparent windows only partially under Hyprland (blank areas on
    // hover). On Hyprland/HyDE the glass look comes from the compositor
    // (decoration:active_opacity + blur) instead.
    #[cfg(target_os = "linux")]
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        // SAFETY: first statement in main, no other threads exist yet.
        unsafe { std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1") };
    }
    app_lib::run();
}
