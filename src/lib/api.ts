// Thin, typed layer over the Tauri commands (src-tauri/src/lib.rs).
import { invoke } from "@tauri-apps/api/core";
import type { ActionResult, AppInfo, Config, Health, Theme } from "./types";

export const api = {
  loadConfig: () => invoke<Config>("load_config"),
  saveConfig: (config: Config) => invoke<void>("save_config", { config }),
  configPath: () => invoke<string>("config_path"),
  detectedTerminal: () => invoke<string | null>("detected_terminal"),
  appInfo: () => invoke<AppInfo>("app_info"),
  runAction: (appId: string, actionId: string) =>
    invoke<ActionResult>("run_action", { appId, actionId }),
  getTheme: () => invoke<Theme>("get_theme"),
  loadIcon: (spec: string) => invoke<string>("load_icon", { spec }),
  checkHealth: () => invoke<Health[]>("check_health"),
  fitWindow: (width: number, height: number) => invoke<void>("fit_window", { width, height }),
};
