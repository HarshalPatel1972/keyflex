import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// IPC return types
export interface ShortcutEntry {
  shortcut: string;
  app_name: string;
  count: number;
}

export interface TodayStats {
  total: number;
  unique: number;
  new_unlocks: number;
  mouse_escapes: number;
}

export interface StreakData {
  current: number;
  longest: number;
  last_date: string;
}

export interface DayActivity {
  date: string;
  count: number;
  level: number;
}

export interface GapItem {
  shortcut: string;
  app: string;
  description: string;
  power_user_pct: number;
  unlocked: boolean;
  priority: number;
}

export interface EfficiencyScore {
  score: number;
  shortcuts_today: number;
  unique_today: number;
  mouse_escapes: number;
}

export interface MilestoneRecord {
  key: string;
  shortcut: string;
  achieved_at: number | null;
  label: string;
  description: string;
}

export const api = {
  getTopShortcuts: (app: string, days: number, limit: number) =>
    invoke<ShortcutEntry[]>("get_top_shortcuts", { app, days, limit }),
  getHeatmapData: (app: string, days: number) =>
    invoke<Record<string, number>>("get_heatmap_data", { app, days }),
  getAppsList: () => invoke<string[]>("get_apps_list"),
  getTodayStats: (app: string) => invoke<TodayStats>("get_today_stats", { app }),
  getEfficiencyScore: () => invoke<EfficiencyScore>("get_efficiency_score"),
  getStreak: () => invoke<StreakData>("get_streak"),
  getDayActivity: (days: number) => invoke<DayActivity[]>("get_day_activity", { days }),
  getUniqueShortcuts: (app: string) => invoke<string[]>("get_unique_shortcuts", { app }),
  getMilestones: () => invoke<MilestoneRecord[]>("get_milestones"),
  getSetting: (key: string) => invoke<string>("get_setting", { key }),
  setSetting: (key: string, value: string) => invoke<void>("set_setting", { key, value }),
  getShortcutOfDay: () => invoke<GapItem>("get_shortcut_of_day"),
  getPerAppBreakdown: (app: string, days: number) =>
    invoke<ShortcutEntry[]>("get_per_app_breakdown", { app, days }),
  getGapItems: (app: string) => invoke<GapItem[]>("get_gap_items", { app }),
};

export const onShortcutRecorded = (cb: (data: { shortcut: string; app: string }) => void) =>
  listen("shortcut-recorded", (e) => cb(e.payload as any));

export const onMilestoneAchieved = (cb: (data: MilestoneRecord) => void) =>
  listen("milestone-achieved", (e) => cb(e.payload as any));

export const onStreakUpdated = (cb: (data: StreakData) => void) =>
  listen("streak-updated", (e) => cb(e.payload as any));
