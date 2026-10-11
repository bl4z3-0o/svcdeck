// Mirror of the Rust structs in src-tauri/src/config.rs

export type ActionMode = "capture" | "terminal";

export interface Action {
  id: string;
  label: string;
  command: string;
  mode: ActionMode;
  confirm: boolean;
}

export interface App {
  id: string;
  name: string;
  url: string;
  icon: string;
  glyph: string;
  dir: string;
  health: string;
  healthCommand: string;
  actions: Action[];
}

export type ThemePref = "auto" | "light" | "dark";
export type LangPref = "auto" | "en" | "de";

export interface Config {
  title: string;
  columns: number;
  terminal: string[];
  theme: ThemePref;
  language: LangPref;
  apps: App[];
}

export interface ActionResult {
  ok: boolean;
  code: number | null;
  output: string;
  detached: boolean;
}

export interface Health {
  id: string;
  /** null = no status check configured */
  running: boolean | null;
}

export interface Theme {
  available: boolean;
  mode: string;
  colors: Record<string, string>;
}
