use chrono::Local;
use rusqlite::{params, Connection, Result};
use std::collections::HashMap;
use std::path::Path;

use crate::types::{DayActivity, MilestoneRecord, StreakData, TodayStats};

/// Initialize the database at the given directory, creating schema if needed.
pub fn init(app_dir: &Path) -> Result<Connection> {
    std::fs::create_dir_all(app_dir).ok();
    let db_path = app_dir.join("keyflex.db");
    let conn = Connection::open(db_path)?;

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS shortcuts (
            id        INTEGER PRIMARY KEY AUTOINCREMENT,
            shortcut  TEXT    NOT NULL,
            app_name  TEXT    NOT NULL,
            exe_path  TEXT    DEFAULT '',
            timestamp INTEGER NOT NULL,
            date      TEXT    NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_shortcuts_date     ON shortcuts(date);
        CREATE INDEX IF NOT EXISTS idx_shortcuts_app_date ON shortcuts(app_name, date);
        CREATE INDEX IF NOT EXISTS idx_shortcuts_combo    ON shortcuts(shortcut, app_name);

        CREATE TABLE IF NOT EXISTS streak (
            id              INTEGER PRIMARY KEY CHECK(id = 1),
            current_streak  INTEGER NOT NULL DEFAULT 0,
            longest_streak  INTEGER NOT NULL DEFAULT 0,
            last_active_date TEXT
        );
        INSERT OR IGNORE INTO streak(id) VALUES(1);

        CREATE TABLE IF NOT EXISTS milestones (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            milestone_key TEXT    UNIQUE NOT NULL,
            achieved_at   INTEGER,
            shortcut      TEXT
        );

        CREATE TABLE IF NOT EXISTS settings (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        INSERT OR IGNORE INTO settings VALUES('onboarding_done', 'false');
        INSERT OR IGNORE INTO settings VALUES('autostart',       'true');
        ",
    )?;

    Ok(conn)
}

/// Insert a new shortcut event.
pub fn insert_shortcut(
    conn: &Connection,
    shortcut: &str,
    app_name: &str,
    exe_path: &str,
) -> Result<()> {
    let now = Local::now();
    let timestamp = now.timestamp_millis();
    let date = now.format("%Y-%m-%d").to_string();

    conn.execute(
        "INSERT INTO shortcuts (shortcut, app_name, exe_path, timestamp, date) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![shortcut, app_name, exe_path, timestamp, date],
    )?;
    Ok(())
}

/// Get top shortcuts by count, optionally filtered by app, within last N days.
pub fn get_top_shortcuts(
    conn: &Connection,
    app: &str,
    days: u32,
    limit: u32,
) -> Result<Vec<(String, String, u64)>> {
    let date_cutoff = date_n_days_ago(days);

    let (query, params_vec): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = if app == "All Apps"
        || app.is_empty()
    {
        (
            "SELECT shortcut, app_name, COUNT(*) as cnt FROM shortcuts WHERE date >= ?1 GROUP BY shortcut, app_name ORDER BY cnt DESC LIMIT ?2".to_string(),
            vec![Box::new(date_cutoff), Box::new(limit)],
        )
    } else {
        (
            "SELECT shortcut, app_name, COUNT(*) as cnt FROM shortcuts WHERE app_name = ?1 AND date >= ?2 GROUP BY shortcut, app_name ORDER BY cnt DESC LIMIT ?3".to_string(),
            vec![Box::new(app.to_string()), Box::new(date_cutoff), Box::new(limit)],
        )
    };

    let mut stmt = conn.prepare(&query)?;
    let params_refs: Vec<&dyn rusqlite::types::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
    let rows = stmt.query_map(params_refs.as_slice(), |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, u64>(2)?,
        ))
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// Get heatmap key counts: key_name → total press count.
pub fn get_heatmap_keys(
    conn: &Connection,
    app: &str,
    days: u32,
) -> Result<HashMap<String, u64>> {
    let date_cutoff = date_n_days_ago(days);
    let mut map = HashMap::new();

    let (query, params_vec): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = if app == "All Apps"
        || app.is_empty()
    {
        (
            "SELECT shortcut, COUNT(*) as cnt FROM shortcuts WHERE date >= ?1 GROUP BY shortcut"
                .to_string(),
            vec![Box::new(date_cutoff)],
        )
    } else {
        (
            "SELECT shortcut, COUNT(*) as cnt FROM shortcuts WHERE app_name = ?1 AND date >= ?2 GROUP BY shortcut"
                .to_string(),
            vec![Box::new(app.to_string()), Box::new(date_cutoff)],
        )
    };

    let mut stmt = conn.prepare(&query)?;
    let params_refs: Vec<&dyn rusqlite::types::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
    let rows = stmt.query_map(params_refs.as_slice(), |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, u64>(1)?))
    })?;

    for row in rows {
        let (shortcut, count) = row?;
        // Split "Ctrl+Shift+P" into individual keys and accumulate
        for key in shortcut.split('+') {
            let entry = map.entry(key.to_string()).or_insert(0);
            *entry += count;
        }
    }

    Ok(map)
}

/// Get list of all app names that have recorded shortcuts.
pub fn get_apps_list(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt =
        conn.prepare("SELECT DISTINCT app_name FROM shortcuts ORDER BY app_name ASC")?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;

    let mut apps = vec!["All Apps".to_string()];
    for row in rows {
        apps.push(row?);
    }
    Ok(apps)
}

/// Get activity counts per day for the last N days.
pub fn get_day_activity(conn: &Connection, days: u32) -> Result<Vec<DayActivity>> {
    let date_cutoff = date_n_days_ago(days);

    let mut stmt = conn.prepare(
        "SELECT date, COUNT(*) as cnt FROM shortcuts WHERE date >= ?1 GROUP BY date ORDER BY date ASC",
    )?;
    let rows = stmt.query_map(params![date_cutoff], |row| {
        let count: u64 = row.get(1)?;
        let level = match count {
            0 => 0u8,
            1..=5 => 1,
            6..=20 => 2,
            21..=50 => 3,
            _ => 4,
        };
        Ok(DayActivity {
            date: row.get(0)?,
            count,
            level,
        })
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// Get today's stats, optionally filtered by app.
pub fn get_today_stats(conn: &Connection, app: &str) -> Result<TodayStats> {
    let today = today_date_string();

    let (total, unique) = if app == "All Apps" || app.is_empty() {
        let total: u64 = conn.query_row(
            "SELECT COUNT(*) FROM shortcuts WHERE date = ?1",
            params![today],
            |row| row.get(0),
        )?;
        let unique: u64 = conn.query_row(
            "SELECT COUNT(DISTINCT shortcut) FROM shortcuts WHERE date = ?1",
            params![today],
            |row| row.get(0),
        )?;
        (total, unique)
    } else {
        let total: u64 = conn.query_row(
            "SELECT COUNT(*) FROM shortcuts WHERE date = ?1 AND app_name = ?2",
            params![today, app],
            |row| row.get(0),
        )?;
        let unique: u64 = conn.query_row(
            "SELECT COUNT(DISTINCT shortcut) FROM shortcuts WHERE date = ?1 AND app_name = ?2",
            params![today, app],
            |row| row.get(0),
        )?;
        (total, unique)
    };

    // New unlocks today: shortcuts pressed today that were never pressed before today
    let new_unlocks: u64 = if app == "All Apps" || app.is_empty() {
        conn.query_row(
            "SELECT COUNT(DISTINCT s1.shortcut) FROM shortcuts s1
             WHERE s1.date = ?1
             AND NOT EXISTS (SELECT 1 FROM shortcuts s2 WHERE s2.shortcut = s1.shortcut AND s2.date < ?1)",
            params![today],
            |row| row.get(0),
        )?
    } else {
        conn.query_row(
            "SELECT COUNT(DISTINCT s1.shortcut) FROM shortcuts s1
             WHERE s1.date = ?1 AND s1.app_name = ?2
             AND NOT EXISTS (SELECT 1 FROM shortcuts s2 WHERE s2.shortcut = s1.shortcut AND s2.app_name = ?2 AND s2.date < ?1)",
            params![today, app],
            |row| row.get(0),
        )?
    };

    Ok(TodayStats {
        total,
        unique,
        new_unlocks,
        mouse_escapes: 0, // Future: WH_MOUSE_LL
    })
}

/// Get all unique shortcut strings ever recorded, optionally filtered by app.
pub fn get_unique_shortcuts(conn: &Connection, app: &str) -> Result<Vec<String>> {
    let mut results = Vec::new();

    if app == "All Apps" || app.is_empty() {
        let mut stmt = conn.prepare("SELECT DISTINCT shortcut FROM shortcuts")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        for row in rows {
            results.push(row?);
        }
    } else {
        let mut stmt =
            conn.prepare("SELECT DISTINCT shortcut FROM shortcuts WHERE app_name = ?1")?;
        let rows = stmt.query_map(params![app], |row| row.get::<_, String>(0))?;
        for row in rows {
            results.push(row?);
        }
    }

    Ok(results)
}

/// Get current streak data.
pub fn get_streak(conn: &Connection) -> Result<StreakData> {
    let mut stmt =
        conn.prepare("SELECT current_streak, longest_streak, last_active_date FROM streak WHERE id = 1")?;
    let result = stmt.query_row([], |row| {
        Ok(StreakData {
            current: row.get::<_, u32>(0)?,
            longest: row.get::<_, u32>(1)?,
            last_date: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
        })
    })?;
    Ok(result)
}

/// Update streak based on today's date. Returns updated streak data.
pub fn update_streak(conn: &Connection, today: &str) -> Result<StreakData> {
    let current = get_streak(conn)?;

    if current.last_date == today {
        // Already active today, no change
        return Ok(current);
    }

    let yesterday = yesterday_date_string();
    let (new_current, new_longest) = if current.last_date == yesterday {
        // Continuing streak
        let new_c = current.current + 1;
        let new_l = std::cmp::max(current.longest, new_c);
        (new_c, new_l)
    } else {
        // Streak broken (or first ever)
        (1, std::cmp::max(current.longest, 1))
    };

    conn.execute(
        "UPDATE streak SET current_streak = ?1, longest_streak = ?2, last_active_date = ?3 WHERE id = 1",
        params![new_current, new_longest, today],
    )?;

    Ok(StreakData {
        current: new_current,
        longest: new_longest,
        last_date: today.to_string(),
    })
}

/// Get a setting value by key.
pub fn get_setting(conn: &Connection, key: &str) -> Result<String> {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
}

/// Set a setting value.
pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        params![key, value],
    )?;
    Ok(())
}

/// Save a milestone. Returns true if newly achieved (first time).
pub fn save_milestone(conn: &Connection, key: &str, shortcut: &str) -> Result<bool> {
    let exists: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM milestones WHERE milestone_key = ?1",
            params![key],
            |row| row.get::<_, u64>(0),
        )
        .map(|c| c > 0)?;

    if exists {
        return Ok(false);
    }

    let now = chrono::Local::now().timestamp_millis();
    conn.execute(
        "INSERT INTO milestones (milestone_key, achieved_at, shortcut) VALUES (?1, ?2, ?3)",
        params![key, now, shortcut],
    )?;
    Ok(true)
}

/// Get all milestones.
pub fn get_milestones(conn: &Connection) -> Result<Vec<MilestoneRecord>> {
    let mut stmt = conn.prepare("SELECT milestone_key, shortcut, achieved_at FROM milestones")?;
    let rows = stmt.query_map([], |row| {
        let key: String = row.get(0)?;
        let shortcut: String = row.get::<_, Option<String>>(1)?.unwrap_or_default();
        let achieved_at: Option<i64> = row.get(2)?;
        let (label, description) = milestone_meta(&key);
        Ok(MilestoneRecord {
            key: key.clone(),
            shortcut,
            achieved_at,
            label: label.to_string(),
            description: description.to_string(),
        })
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

/// Get total unique shortcut count (all time, all apps).
pub fn get_total_unique_count(conn: &Connection) -> Result<u64> {
    conn.query_row(
        "SELECT COUNT(DISTINCT shortcut) FROM shortcuts",
        [],
        |row| row.get(0),
    )
}

/// Get total shortcut count (all time).
pub fn get_total_count(conn: &Connection) -> Result<u64> {
    conn.query_row("SELECT COUNT(*) FROM shortcuts", [], |row| row.get(0))
}

/// Get unique shortcut count for a specific app (all time).
pub fn get_app_unique_count(conn: &Connection, app: &str) -> Result<u64> {
    conn.query_row(
        "SELECT COUNT(DISTINCT shortcut) FROM shortcuts WHERE app_name = ?1",
        params![app],
        |row| row.get(0),
    )
}

// ── Helpers ─────────────────────────────────────────────────────────────────

pub fn today_date_string() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

fn yesterday_date_string() -> String {
    (Local::now() - chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string()
}

fn date_n_days_ago(days: u32) -> String {
    if days == 0 {
        return "1970-01-01".to_string(); // All time
    }
    (Local::now() - chrono::Duration::days(days as i64))
        .format("%Y-%m-%d")
        .to_string()
}

fn milestone_meta(key: &str) -> (&str, &str) {
    match key {
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
    }
}
