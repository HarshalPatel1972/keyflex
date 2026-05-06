use rusqlite::Connection;

use crate::db;
use crate::types::GapItem;

/// Known shortcuts database — minimum set for gap analysis.
/// Each entry: (shortcut, app, description, power_user_pct, priority)
static KNOWN_SHORTCUTS: &[(&str, &str, &str, u8, u8)] = &[
    // ── VS Code ──────────────────────────────────────────────────
    ("Ctrl+Shift+P", "VS Code", "Command Palette", 94, 1),
    ("Ctrl+P", "VS Code", "Quick Open file by name", 91, 1),
    ("Ctrl+`", "VS Code", "Toggle integrated terminal", 88, 1),
    ("Ctrl+B", "VS Code", "Toggle sidebar", 75, 1),
    ("Alt+\u{2191}", "VS Code", "Move line up", 70, 1),
    ("Alt+\u{2193}", "VS Code", "Move line down", 70, 1),
    ("Ctrl+D", "VS Code", "Select next occurrence", 82, 1),
    ("Ctrl+Shift+K", "VS Code", "Delete line", 68, 1),
    ("Ctrl+/", "VS Code", "Toggle line comment", 85, 1),
    ("Ctrl+Shift+L", "VS Code", "Select all occurrences", 60, 2),
    ("Ctrl+G", "VS Code", "Go to line", 55, 2),
    ("Ctrl+Shift+F", "VS Code", "Search across all files", 78, 1),
    ("Shift+F12", "VS Code", "Go to definition", 80, 1),
    ("Alt+F12", "VS Code", "Peek definition", 45, 2),
    ("Ctrl+Shift+O", "VS Code", "Go to symbol in file", 50, 2),
    ("Ctrl+Shift+V", "VS Code", "Markdown preview", 40, 3),
    ("Ctrl+\\", "VS Code", "Split editor", 62, 2),
    ("Ctrl+W", "VS Code", "Close tab", 88, 1),
    ("Ctrl+Tab", "VS Code", "Cycle open editors", 75, 1),
    ("Ctrl+Shift+E", "VS Code", "Focus file explorer", 55, 2),
    // ── Chrome ───────────────────────────────────────────────────
    ("Ctrl+L", "Chrome", "Focus address bar", 72, 1),
    ("Ctrl+T", "Chrome", "New tab", 95, 1),
    ("Ctrl+W", "Chrome", "Close tab", 92, 1),
    ("Ctrl+Shift+T", "Chrome", "Reopen closed tab", 80, 1),
    ("Ctrl+Tab", "Chrome", "Next tab", 85, 1),
    ("Ctrl+Shift+Tab", "Chrome", "Previous tab", 65, 1),
    ("Ctrl+Shift+J", "Chrome", "Open DevTools Console", 60, 2),
    ("Ctrl+Shift+I", "Chrome", "Open DevTools", 65, 2),
    ("Ctrl+Shift+N", "Chrome", "New incognito window", 70, 2),
    ("Ctrl+F", "Chrome", "Find on page", 90, 1),
    ("Ctrl+R", "Chrome", "Reload page", 93, 1),
    ("Ctrl+Shift+R", "Chrome", "Hard reload (no cache)", 55, 2),
    ("Ctrl+D", "Chrome", "Bookmark page", 60, 2),
    ("Alt+\u{2190}", "Chrome", "Go back", 75, 1),
    ("Alt+\u{2192}", "Chrome", "Go forward", 65, 1),
    // ── Windows System ───────────────────────────────────────────
    ("Win+D", "Windows", "Show/hide desktop", 60, 1),
    ("Win+E", "Windows", "Open File Explorer", 75, 1),
    ("Win+L", "Windows", "Lock screen", 80, 1),
    ("Win+V", "Windows", "Clipboard history", 38, 2),
    ("Win+Shift+S", "Windows", "Screenshot snip", 65, 1),
    ("Win+\u{2191}", "Windows", "Maximize window", 55, 2),
    ("Win+\u{2190}", "Windows", "Snap window left", 70, 1),
    ("Win+\u{2192}", "Windows", "Snap window right", 70, 1),
    ("Win+Tab", "Windows", "Task View", 50, 2),
    ("Ctrl+Shift+Esc", "Windows", "Open Task Manager", 68, 1),
    ("Alt+F4", "Windows", "Close window", 80, 1),
    ("Win+.", "Windows", "Emoji picker", 42, 3),
    // ── General ──────────────────────────────────────────────────
    ("Ctrl+Z", "General", "Undo", 98, 1),
    ("Ctrl+Y", "General", "Redo", 90, 1),
    ("Ctrl+Shift+Z", "General", "Redo (alt)", 70, 1),
    ("Ctrl+C", "General", "Copy", 99, 1),
    ("Ctrl+X", "General", "Cut", 95, 1),
    ("Ctrl+V", "General", "Paste", 99, 1),
    ("Ctrl+Shift+V", "General", "Paste without formatting", 48, 2),
    ("Ctrl+A", "General", "Select all", 97, 1),
    ("Ctrl+S", "General", "Save", 98, 1),
    ("Ctrl+F", "General", "Find", 93, 1),
    ("Ctrl+H", "General", "Find & Replace", 72, 1),
    ("Ctrl+N", "General", "New", 85, 1),
    ("Ctrl+O", "General", "Open", 82, 1),
    ("Ctrl+P", "General", "Print", 60, 2),
    ("Ctrl+Home", "General", "Jump to top of document", 55, 2),
    ("Ctrl+End", "General", "Jump to bottom", 55, 2),
];

/// Get the "Shortcut of the Day" — deterministic per calendar day.
pub fn get_shortcut_of_day(conn: &Connection) -> GapItem {
    let user_shortcuts = db::get_unique_shortcuts(conn, "All Apps").unwrap_or_default();
    let day_of_year = chrono::Local::now().format("%j").to_string();
    let seed: usize = day_of_year.parse().unwrap_or(1);

    // Filter to unlocked gaps first, then fall back to any
    let gaps: Vec<_> = KNOWN_SHORTCUTS
        .iter()
        .filter(|(sc, _, _, _, _)| !user_shortcuts.contains(&sc.to_string()))
        .collect();

    if gaps.is_empty() {
        // All known shortcuts are unlocked — return any
        let entry = &KNOWN_SHORTCUTS[seed % KNOWN_SHORTCUTS.len()];
        return GapItem {
            shortcut: entry.0.to_string(),
            app: entry.1.to_string(),
            description: entry.2.to_string(),
            power_user_pct: entry.3,
            unlocked: true,
            priority: entry.4,
        };
    }

    let entry = gaps[seed % gaps.len()];
    GapItem {
        shortcut: entry.0.to_string(),
        app: entry.1.to_string(),
        description: entry.2.to_string(),
        power_user_pct: entry.3,
        unlocked: false,
        priority: entry.4,
    }
}

/// Get gap analysis: known shortcuts the user hasn't pressed yet.
/// Sorted by priority ASC, then power_user_pct DESC.
pub fn get_gap_items(conn: &Connection, app_filter: &str) -> Vec<GapItem> {
    let user_shortcuts = db::get_unique_shortcuts(conn, "All Apps").unwrap_or_default();

    let mut items: Vec<GapItem> = KNOWN_SHORTCUTS
        .iter()
        .filter(|(_, app, _, _, _)| {
            app_filter == "All Apps" || app_filter.is_empty() || *app == app_filter
        })
        .map(|(sc, app, desc, pct, pri)| {
            let unlocked = user_shortcuts.contains(&sc.to_string());
            GapItem {
                shortcut: sc.to_string(),
                app: app.to_string(),
                description: desc.to_string(),
                power_user_pct: *pct,
                unlocked,
                priority: *pri,
            }
        })
        .collect();

    items.sort_by(|a, b| {
        a.priority
            .cmp(&b.priority)
            .then(b.power_user_pct.cmp(&a.power_user_pct))
    });

    items
}
