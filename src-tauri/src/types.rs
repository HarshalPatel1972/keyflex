use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ShortcutEntry {
    pub shortcut: String,
    pub app_name: String,
    pub count: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TodayStats {
    pub total: u64,
    pub unique: u64,
    pub new_unlocks: u64,
    pub mouse_escapes: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct StreakData {
    pub current: u32,
    pub longest: u32,
    pub last_date: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct DayActivity {
    pub date: String,
    pub count: u64,
    pub level: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GapItem {
    pub shortcut: String,
    pub app: String,
    pub description: String,
    pub power_user_pct: u8,
    pub unlocked: bool,
    pub priority: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct EfficiencyScore {
    pub score: f32,
    pub shortcuts_today: u64,
    pub unique_today: u64,
    pub mouse_escapes: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MilestoneRecord {
    pub key: String,
    pub shortcut: String,
    pub achieved_at: Option<i64>,
    pub label: String,
    pub description: String,
}
