import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface Tip {
  id: string;
  apps: string[];
  keys: string;
  line: string;
  shown: number;
  last_shown: number;
  muted: boolean;
  learned: boolean;
}

export type Theme = "system" | "light" | "dark";

export type Mood = "knowing" | "wink" | "cheeky" | "pleading" | "proud" | "sleepy";

/** How the character is feeling overall. */
export interface Status {
  mood: Mood;
  /** The shortcut it is still hoping you will try, e.g. "Ctrl + J". */
  waiting_on: string | null;
}

export interface Settings {
  theme: Theme;
  paused: boolean;
  tips_per_day: number;
  disabled_apps: string[];
  intro_seen: boolean;
}

export const api = {
  getTips: () => invoke<Tip[]>("get_tips"),
  setTipMuted: (id: string, muted: boolean) => invoke<void>("set_tip_muted", { id, muted }),
  getSettings: () => invoke<Settings>("get_settings"),
  setSettings: (settings: Settings) => invoke<void>("set_settings", { settings }),
  getApps: () => invoke<string[]>("get_apps"),
  getAutostart: () => invoke<boolean>("get_autostart"),
  setAutostart: (enabled: boolean) => invoke<void>("set_autostart", { enabled }),
  previewTip: () => invoke<void>("preview_tip"),
  getStatus: () => invoke<Status>("get_status"),
};

/** Runs `callback` whenever tips or settings change in the background. */
export const onChanged = (callback: () => void) => listen("changed", callback);

const APP_NAMES: Record<string, string> = {
  "*": "Everywhere",
  "chrome.exe": "Chrome",
  "msedge.exe": "Edge",
  "brave.exe": "Brave",
  "explorer.exe": "File Explorer",
  "taskmgr.exe": "Task Manager",
  "systemsettings.exe": "Settings",
  "snippingtool.exe": "Snipping Tool",
  "winword.exe": "Word",
  "excel.exe": "Excel",
  "powerpnt.exe": "PowerPoint",
  "onenote.exe": "OneNote",
  "notepad.exe": "Notepad",
  "code.exe": "VS Code",
  "cursor.exe": "Cursor",
};

export const appName = (exe: string) => APP_NAMES[exe] ?? exe.replace(/\.exe$/, "");
