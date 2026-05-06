use std::collections::HashMap;
use parking_lot::Mutex;
use std::sync::Arc;
use rusqlite::Connection;

use crate::analytics;
use crate::db;
use crate::gaps;
use crate::types::*;

/// Database state managed by Tauri.
pub struct DbState(pub Arc<Mutex<Connection>>);

// Implement Clone for DbState so we can share it across threads
impl Clone for DbState {
    fn clone(&self) -> Self {
        DbState(Arc::clone(&self.0))
    }
}

#[tauri::command]
pub async fn get_top_shortcuts(
    state: tauri::State<'_, DbState>,
    app: String,
    days: u32,
    limit: u32,
) -> Result<Vec<ShortcutEntry>, String> {
    let conn = state.0.lock();
    let rows = db::get_top_shortcuts(&conn, &app, days, limit).map_err(|e| e.to_string())?;
    Ok(rows
        .into_iter()
        .map(|(shortcut, app_name, count)| ShortcutEntry {
            shortcut,
            app_name,
            count,
        })
        .collect())
}

#[tauri::command]
pub async fn get_heatmap_data(
    state: tauri::State<'_, DbState>,
    app: String,
    days: u32,
) -> Result<HashMap<String, u64>, String> {
    let conn = state.0.lock();
    db::get_heatmap_keys(&conn, &app, days).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_apps_list(
    state: tauri::State<'_, DbState>,
) -> Result<Vec<String>, String> {
    let conn = state.0.lock();
    db::get_apps_list(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_today_stats(
    state: tauri::State<'_, DbState>,
    app: String,
) -> Result<TodayStats, String> {
    let conn = state.0.lock();
    db::get_today_stats(&conn, &app).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_efficiency_score(
    state: tauri::State<'_, DbState>,
) -> Result<EfficiencyScore, String> {
    let conn = state.0.lock();
    analytics::compute_efficiency_score(&conn)
}

#[tauri::command]
pub async fn get_streak(
    state: tauri::State<'_, DbState>,
) -> Result<StreakData, String> {
    let conn = state.0.lock();
    db::get_streak(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_day_activity(
    state: tauri::State<'_, DbState>,
    days: u32,
) -> Result<Vec<DayActivity>, String> {
    let conn = state.0.lock();
    db::get_day_activity(&conn, days).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_unique_shortcuts(
    state: tauri::State<'_, DbState>,
    app: String,
) -> Result<Vec<String>, String> {
    let conn = state.0.lock();
    db::get_unique_shortcuts(&conn, &app).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_milestones(
    state: tauri::State<'_, DbState>,
) -> Result<Vec<MilestoneRecord>, String> {
    let conn = state.0.lock();
    db::get_milestones(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_setting(
    state: tauri::State<'_, DbState>,
    key: String,
) -> Result<String, String> {
    let conn = state.0.lock();
    db::get_setting(&conn, &key).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_setting(
    state: tauri::State<'_, DbState>,
    key: String,
    value: String,
) -> Result<(), String> {
    let conn = state.0.lock();
    db::set_setting(&conn, &key, &value).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_shortcut_of_day(
    state: tauri::State<'_, DbState>,
) -> Result<GapItem, String> {
    let conn = state.0.lock();
    Ok(gaps::get_shortcut_of_day(&conn))
}

#[tauri::command]
pub async fn get_per_app_breakdown(
    state: tauri::State<'_, DbState>,
    app: String,
    days: u32,
) -> Result<Vec<ShortcutEntry>, String> {
    let conn = state.0.lock();
    let rows = db::get_top_shortcuts(&conn, &app, days, 100).map_err(|e| e.to_string())?;
    Ok(rows
        .into_iter()
        .map(|(shortcut, app_name, count)| ShortcutEntry {
            shortcut,
            app_name,
            count,
        })
        .collect())
}

#[tauri::command]
pub async fn get_gap_items(
    state: tauri::State<'_, DbState>,
    app: String,
) -> Result<Vec<GapItem>, String> {
    let conn = state.0.lock();
    Ok(gaps::get_gap_items(&conn, &app))
}
