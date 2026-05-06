use rusqlite::Connection;

use crate::db;
use crate::types::{EfficiencyScore, MilestoneRecord};

/// Check and award milestones based on current state.
/// Returns a list of newly achieved milestones.
pub fn check_milestones(
    conn: &Connection,
    shortcut: &str,
    app_name: &str,
) -> Vec<MilestoneRecord> {
    let mut achieved = Vec::new();

    // Total shortcut count
    let total = db::get_total_count(conn).unwrap_or(0);
    let unique = db::get_total_unique_count(conn).unwrap_or(0);
    let streak = db::get_streak(conn).unwrap_or(crate::types::StreakData {
        current: 0,
        longest: 0,
        last_date: String::new(),
    });

    // first_shortcut
    if total == 1 {
        if let Ok(true) = db::save_milestone(conn, "first_shortcut", shortcut) {
            achieved.push(make_milestone("first_shortcut", shortcut));
        }
    }

    // ten_unique
    if unique >= 10 {
        if let Ok(true) = db::save_milestone(conn, "ten_unique", shortcut) {
            achieved.push(make_milestone("ten_unique", shortcut));
        }
    }

    // fifty_unique
    if unique >= 50 {
        if let Ok(true) = db::save_milestone(conn, "fifty_unique", shortcut) {
            achieved.push(make_milestone("fifty_unique", shortcut));
        }
    }

    // hundred_unique
    if unique >= 100 {
        if let Ok(true) = db::save_milestone(conn, "hundred_unique", shortcut) {
            achieved.push(make_milestone("hundred_unique", shortcut));
        }
    }

    // streak_7
    if streak.current >= 7 {
        if let Ok(true) = db::save_milestone(conn, "streak_7", shortcut) {
            achieved.push(make_milestone("streak_7", shortcut));
        }
    }

    // streak_30
    if streak.current >= 30 {
        if let Ok(true) = db::save_milestone(conn, "streak_30", shortcut) {
            achieved.push(make_milestone("streak_30", shortcut));
        }
    }

    // vscode_power: 15+ unique shortcuts in VS Code
    if app_name == "VS Code" {
        if let Ok(count) = db::get_app_unique_count(conn, "VS Code") {
            if count >= 15 {
                if let Ok(true) = db::save_milestone(conn, "vscode_power", shortcut) {
                    achieved.push(make_milestone("vscode_power", shortcut));
                }
            }
        }
    }

    // Check efficiency score milestone
    if let Ok(score) = compute_efficiency_score(conn) {
        if score.score >= 90.0 {
            if let Ok(true) = db::save_milestone(conn, "score_90", shortcut) {
                achieved.push(make_milestone("score_90", shortcut));
            }
        }
    }

    achieved
}

/// Compute the efficiency score (0–100).
pub fn compute_efficiency_score(conn: &Connection) -> Result<EfficiencyScore, String> {
    let today_stats = db::get_today_stats(conn, "All Apps").map_err(|e| e.to_string())?;
    let streak = db::get_streak(conn).map_err(|e| e.to_string())?;

    // Check if user used any gap shortcut today (simplified: any new unlock counts)
    let gap_bonus: f32 = if today_stats.new_unlocks > 0 {
        10.0
    } else {
        0.0
    };

    let variety_pts = (today_stats.unique as f32 / 10.0) * 40.0;
    let volume_pts = (today_stats.total as f32 / 50.0) * 30.0;
    let streak_pts = (streak.current as f32 / 7.0) * 20.0;

    let score = (variety_pts + volume_pts + streak_pts + gap_bonus).clamp(0.0, 100.0);

    Ok(EfficiencyScore {
        score,
        shortcuts_today: today_stats.total,
        unique_today: today_stats.unique,
        mouse_escapes: 0,
    })
}

fn make_milestone(key: &str, shortcut: &str) -> MilestoneRecord {
    let (label, description) = match key {
        "first_shortcut" => ("First Key!", "You fired your first shortcut"),
        "ten_unique" => ("10 Combos Unlocked", "You know 10 unique shortcuts"),
        "fifty_unique" => ("Shortcut Veteran", "50 unique combos in your arsenal"),
        "hundred_unique" => ("Keyboard Master", "100 shortcuts unlocked"),
        "streak_7" => ("Week Warrior", "7-day streak"),
        "streak_30" => ("Monthly Legend", "30-day streak"),
        "first_gap_unlock" => ("Gap Closer", "You used a recommended shortcut"),
        "score_90" => ("Elite Reflexes", "Efficiency score above 90"),
        "vscode_power" => ("VS Code Wizard", "15+ VS Code shortcuts"),
        "no_mouse_day" => ("Mouseless", "Full day, keyboard only"),
        _ => ("Unknown", "Unknown milestone"),
    };

    MilestoneRecord {
        key: key.to_string(),
        shortcut: shortcut.to_string(),
        achieved_at: Some(chrono::Local::now().timestamp_millis()),
        label: label.to_string(),
        description: description.to_string(),
    }
}
