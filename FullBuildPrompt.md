# Keyflex — Full Agent Build Prompt
**Repo:** https://github.com/HarshalPatel1972/keyflex.git  
**Platform:** Windows only (MVP)  
**Stack:** Tauri v2 · Rust · React 18 · TypeScript · SQLite  
**Deployment:** Distributed as a native `.exe` installer via GitHub Releases  
**Constraints:** Free tier only · No external servers · No cloud sync · All data stays local  

---

## 0. Project Philosophy

Keyflex tracks only keyboard shortcut combinations (modifier + key, e.g. Ctrl+C). It **never** records plain keystrokes, typed text, or clipboard content. Privacy is architectural, not a promise — the hook layer physically discards events the moment it detects no modifier key is held. Every design and code decision must reinforce this.

The soul of the product: discovery, not guilt. Shortcuts the user hasn't pressed are **locked abilities to unlock**, not failures. The UX is closer to a game than a surveillance tool.

---

## 1. Brand & Design System

Apply these tokens everywhere — CSS variables, Tauri window config, icon colors, README.

### Color Tokens
```css
:root {
  --bg:           #0D0D0D;
  --surface:      #161616;
  --surface2:     #1E1E1E;
  --border:       #2A2A2A;
  --ember:        #F5A623;
  --ember-glow:   rgba(245, 166, 35, 0.15);
  --flame:        #FF6B35;
  --ash:          #E8E8E8;
  --smoke:        #666666;
  --unlocked:     #4ADE80;
  --unlocked-glow: rgba(74, 222, 128, 0.12);
}
```

### Typography
| Role | Font | Weight | Usage |
|---|---|---|---|
| Display / headings | Space Grotesk | 700 | App name, screen titles, big numbers |
| UI body / labels | Inter | 400 / 500 | All prose, labels, descriptions |
| Key chips / data | JetBrains Mono | 500 / 600 | Shortcut chips, hex values, counts |

Load all three from Google Fonts in `index.html`. Never fall back to system-ui or Arial.

### App Name & Tagline
- **Name:** `Keyflex`
- **Tagline:** `Flex your keys. Own your machine.`
- **Mono style (code/README):** `// flex your keys. own your machine.`
- **Window title:** `Keyflex`
- **Tray tooltip:** `Keyflex — running`

### Keycap Chip Component Rules
Every shortcut rendered in the UI must use keycap chips. A chip is a `<span>` styled as:
- Background: `var(--surface2)`
- Border: `1px solid var(--border)`, bottom `3px solid #1A1A1A`
- Border-radius: `6px`
- Font: `JetBrains Mono 500 12px`
- Box-shadow: `0 1px 3px rgba(0,0,0,0.5), inset 0 1px 0 rgba(255,255,255,0.04)`
- State `.active`: border `var(--ember)`, color `var(--ember)`, bg `var(--ember-glow)`
- State `.hot` (top 5 most used): border `var(--flame)`, color `var(--flame)`

A shortcut row renders as: `<KeyChip>Ctrl</KeyChip> + <KeyChip>C</KeyChip>` — never as a flat string.

---

## 2. Repository Structure

Initialize with `npm create tauri-app@latest` choosing React + TypeScript. Then reshape to:

```
keyflex/
├── .github/
│   └── workflows/
│       └── release.yml          # Auto-build on version tag push
├── src-tauri/
│   ├── icons/                   # App icons (generate from SVG below)
│   ├── src/
│   │   ├── main.rs              # Entry point, Tauri builder, tray setup
│   │   ├── hook.rs              # Win32 WH_KEYBOARD_LL hook
│   │   ├── db.rs                # SQLite init + all queries
│   │   ├── apps.rs              # Foreground window → app name resolver
│   │   ├── analytics.rs         # Efficiency score, streak, milestones
│   │   ├── gaps.rs              # Gap analysis engine + known-shortcuts data
│   │   └── commands.rs          # All #[tauri::command] functions
│   ├── Cargo.toml
│   ├── build.rs
│   └── tauri.conf.json
├── src/
│   ├── main.tsx
│   ├── App.tsx
│   ├── components/
│   │   ├── Layout.tsx           # Root shell: sidebar + main panel
│   │   ├── Sidebar.tsx          # Nav links + streak mini-display
│   │   ├── Onboarding.tsx       # First-run full-screen overlay
│   │   ├── tabs/
│   │   │   ├── Overview.tsx
│   │   │   ├── Heatmap.tsx
│   │   │   ├── PerApp.tsx
│   │   │   ├── GapAnalysis.tsx
│   │   │   └── History.tsx
│   │   └── ui/
│   │       ├── KeyChip.tsx
│   │       ├── ShortcutRow.tsx  # Parses "Ctrl+Shift+P" → <KeyChip> sequence
│   │       ├── StatCard.tsx
│   │       ├── ScoreRing.tsx    # SVG ring with gradient stroke
│   │       ├── StreakDisplay.tsx
│   │       ├── AppDropdown.tsx  # App filter selector
│   │       └── HeatLegend.tsx   # Heatmap color legend
│   ├── lib/
│   │   ├── ipc.ts               # Typed wrappers around Tauri invoke()
│   │   ├── gaps-data.ts         # Known shortcuts per app (static JSON)
│   │   └── utils.ts             # Date helpers, shortcut parsing
│   └── styles/
│       ├── globals.css          # CSS variables, reset, scrollbar, fonts
│       └── keyboard.css         # Keyboard SVG / heatmap key styles
├── public/
│   └── keyflex-icon.svg
├── index.html
├── vite.config.ts
├── tsconfig.json
├── package.json
├── README.md
└── CHANGELOG.md
```

---

## 3. Cargo.toml

```toml
[package]
name = "keyflex"
version = "0.1.0"
edition = "2021"

[lib]
name = "keyflex_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-notification = "2"
tauri-plugin-autostart = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.31", features = ["bundled"] }
chrono = { version = "0.4", features = ["serde"] }
once_cell = "1"
parking_lot = "0.12"

[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = [
  "Win32_Foundation",
  "Win32_UI_WindowsAndMessaging",
  "Win32_UI_Input_KeyboardAndMouse",
  "Win32_System_Threading",
  "Win32_System_ProcessStatus",
  "Win32_Storage_FileSystem",
  "Win32_System_LibraryLoader",
] }
```

---

## 4. tauri.conf.json

```json
{
  "productName": "Keyflex",
  "version": "0.1.0",
  "identifier": "com.harshalpatel.keyflex",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build"
  },
  "app": {
    "windows": [
      {
        "title": "Keyflex",
        "width": 1100,
        "height": 720,
        "minWidth": 900,
        "minHeight": 600,
        "resizable": true,
        "decorations": false,
        "transparent": false,
        "visible": false,
        "center": true
      }
    ],
    "trayIcon": {
      "iconPath": "icons/icon.png",
      "tooltip": "Keyflex — running"
    }
  },
  "bundle": {
    "active": true,
    "targets": ["nsis", "msi"],
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  },
  "plugins": {
    "notification": {},
    "autostart": {
      "mobile": false,
      "desktop": true
    }
  }
}
```

Key decisions: `decorations: false` (custom titlebar), `visible: false` on start (app starts hidden in tray, opens on click), autostart plugin so Keyflex launches with Windows.

---

## 5. Database Schema — `db.rs`

Use `rusqlite` with bundled SQLite. Store the DB at `app_data_dir()/keyflex.db`.

```sql
-- Run on first launch / migration

CREATE TABLE IF NOT EXISTS shortcuts (
  id        INTEGER PRIMARY KEY AUTOINCREMENT,
  shortcut  TEXT    NOT NULL,   -- normalized, e.g. "Ctrl+Shift+P"
  app_name  TEXT    NOT NULL,   -- friendly name, e.g. "VS Code"
  exe_path  TEXT    DEFAULT '',
  timestamp INTEGER NOT NULL,   -- unix milliseconds
  date      TEXT    NOT NULL    -- "YYYY-MM-DD", indexed for fast daily queries
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
```

Implement these functions in `db.rs`:
```rust
pub fn init(app_dir: &Path) -> Result<Connection>
pub fn insert_shortcut(conn: &Connection, shortcut: &str, app_name: &str, exe_path: &str) -> Result<()>
pub fn get_top_shortcuts(conn: &Connection, app: &str, days: u32, limit: u32) -> Result<Vec<(String, u64)>>
pub fn get_heatmap_keys(conn: &Connection, app: &str, days: u32) -> Result<HashMap<String, u64>>
pub fn get_apps_list(conn: &Connection) -> Result<Vec<String>>
pub fn get_day_activity(conn: &Connection, days: u32) -> Result<Vec<DayActivity>>
pub fn get_today_stats(conn: &Connection, app: &str) -> Result<TodayStats>
pub fn get_unique_shortcuts(conn: &Connection, app: &str) -> Result<Vec<String>>
pub fn get_streak(conn: &Connection) -> Result<StreakData>
pub fn update_streak(conn: &Connection, today: &str) -> Result<StreakData>
pub fn get_setting(conn: &Connection, key: &str) -> Result<String>
pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<()>
pub fn save_milestone(conn: &Connection, key: &str, shortcut: &str) -> Result<bool>  // returns true if newly achieved
pub fn get_milestones(conn: &Connection) -> Result<Vec<MilestoneRecord>>
```

Use a global `Arc<Mutex<Connection>>` (or `parking_lot::Mutex`) stored in Tauri's managed state.

---

## 6. Rust Structs (Serde-serializable, shared across modules)

Define in `commands.rs` or a `types.rs`:

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ShortcutEntry {
    pub shortcut: String,
    pub app_name: String,
    pub count:    u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TodayStats {
    pub total:         u64,
    pub unique:        u64,
    pub new_unlocks:   u64,
    pub mouse_escapes: u64,   // future: WH_MOUSE_LL frequency spikes
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StreakData {
    pub current:  u32,
    pub longest:  u32,
    pub last_date: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DayActivity {
    pub date:  String,
    pub count: u64,
    pub level: u8,  // 0–4 for contribution grid shading
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GapItem {
    pub shortcut:    String,
    pub app:         String,
    pub description: String,
    pub power_user_pct: u8,   // "used by 94% of power users"
    pub unlocked:    bool,    // true if user has ever pressed it
    pub priority:    u8,      // 1=high, 2=medium, 3=future
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EfficiencyScore {
    pub score:         f32,   // 0–100
    pub shortcuts_today: u64,
    pub unique_today:  u64,
    pub mouse_escapes: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MilestoneRecord {
    pub key:         String,
    pub shortcut:    String,
    pub achieved_at: Option<i64>,
    pub label:       String,
    pub description: String,
}
```

---

## 7. Hook Layer — `hook.rs`

This is the heart of the app. Implement a global `WH_KEYBOARD_LL` hook in a **dedicated OS thread** (the message pump must not block the Tauri main thread).

```rust
// hook.rs — full implementation spec

use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::Threading::*;

// ── Modifier VK codes to watch ──────────────────────────────────────────
const MODIFIERS: &[u16] = &[
    VK_CONTROL.0, VK_LCONTROL.0, VK_RCONTROL.0,
    VK_MENU.0,    VK_LMENU.0,    VK_RMENU.0,
    VK_SHIFT.0,   VK_LSHIFT.0,   VK_RSHIFT.0,
    VK_LWIN.0,    VK_RWIN.0,
];

// ── Hook callback ────────────────────────────────────────────────────────
// Fires on every key event system-wide.
// MUST return immediately — any heavy work goes on a channel.
unsafe extern "system" fn keyboard_proc(
    ncode: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if ncode >= 0 && (wparam.0 as u32 == WM_KEYDOWN || wparam.0 as u32 == WM_SYSKEYDOWN) {
        let kb = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
        let vk = kb.vkCode as u16;

        // Skip if this key IS a modifier (we only want the non-modifier half of a combo)
        if !MODIFIERS.contains(&vk) {
            // Check which modifiers are currently held
            let ctrl  = is_mod_down(VK_CONTROL);
            let alt   = is_mod_down(VK_MENU);
            let shift = is_mod_down(VK_SHIFT);
            let win   = is_mod_down(VK_LWIN) || is_mod_down(VK_RWIN);

            if ctrl || alt || win || (shift && is_special_key(vk)) {
                let shortcut = build_shortcut_string(ctrl, alt, shift, win, vk);
                let _ = HOOK_SENDER.get().map(|tx| tx.send(shortcut));
            }
        }
    }
    CallNextHookEx(None, ncode, wparam, lparam)
}

fn is_mod_down(vk: VIRTUAL_KEY) -> bool {
    unsafe { (GetAsyncKeyState(vk.0 as i32) & 0x8000u16 as i16) != 0 }
}

// is_special_key: Shift alone counts only for function keys F1–F24,
// arrow keys, PgUp/Dn, Home, End, Insert, Delete — not for letters/numbers
// (Shift+A is typing, not a shortcut)
fn is_special_key(vk: u16) -> bool {
    matches!(vk,
        0x21..=0x2F |   // PgUp, PgDn, End, Home, arrows, Insert, Delete
        0x70..=0x87     // F1–F24
    )
}

// build_shortcut_string: normalize to "Ctrl+Alt+Shift+Key" format
// Key name lookup: F-keys → "F1"…"F12", VK_RETURN → "Enter", etc.
// Full VK → name map required (see Section 11)
fn build_shortcut_string(ctrl: bool, alt: bool, shift: bool, win: bool, vk: u16) -> String {
    let mut parts = vec![];
    if ctrl  { parts.push("Ctrl"); }
    if alt   { parts.push("Alt"); }
    if shift { parts.push("Shift"); }
    if win   { parts.push("Win"); }
    parts.push(vk_to_name(vk));
    parts.join("+")
}
```

**Hook startup function** (`pub fn start_hook(sender: Sender<String>)`):
1. Store sender in a global `OnceCell<Sender<String>>`
2. Spawn a new OS thread with `std::thread::spawn`
3. Inside thread: `SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), None, 0)`
4. Run a Windows message pump: `GetMessageW` / `TranslateMessage` / `DispatchMessageW` loop
5. The thread must keep running for the hook to stay active

**Hook receiver** (in `main.rs` setup):
- Spawn a Tokio task that receives from the channel
- For each shortcut string: resolve foreground app (call `apps::get_foreground_app()`), then call `db::insert_shortcut()`, then `analytics::check_milestones()`, then emit `shortcut-recorded` Tauri event to frontend

---

## 8. App Name Resolver — `apps.rs`

```rust
// Get the exe name of the currently focused window
pub fn get_foreground_app() -> (String, String) {
    // Returns (friendly_name, exe_path)
    // 1. GetForegroundWindow()
    // 2. GetWindowThreadProcessId() → pid
    // 3. OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)
    // 4. QueryFullProcessImageNameW() → full exe path
    // 5. Extract filename, normalize via APP_NAME_MAP
    // Return ("Unknown", "") on any failure
}

// Static lookup map: exe filename → friendly display name
// Must include at minimum:
static APP_NAME_MAP: &[(&str, &str)] = &[
    ("Code.exe",              "VS Code"),
    ("Code - Insiders.exe",   "VS Code Insiders"),
    ("chrome.exe",            "Chrome"),
    ("firefox.exe",           "Firefox"),
    ("msedge.exe",            "Edge"),
    ("brave.exe",             "Brave"),
    ("opera.exe",             "Opera"),
    ("notepad.exe",           "Notepad"),
    ("notepad++.exe",         "Notepad++"),
    ("WindowsTerminal.exe",   "Windows Terminal"),
    ("wt.exe",                "Windows Terminal"),
    ("powershell.exe",        "PowerShell"),
    ("cmd.exe",               "Command Prompt"),
    ("explorer.exe",          "File Explorer"),
    ("slack.exe",             "Slack"),
    ("discord.exe",           "Discord"),
    ("Teams.exe",             "Microsoft Teams"),
    ("OUTLOOK.EXE",           "Outlook"),
    ("WINWORD.EXE",           "Word"),
    ("EXCEL.EXE",             "Excel"),
    ("POWERPNT.EXE",          "PowerPoint"),
    ("figma.exe",             "Figma"),
    ("rider64.exe",           "Rider"),
    ("idea64.exe",            "IntelliJ IDEA"),
    ("webstorm64.exe",        "WebStorm"),
    ("studio64.exe",          "Android Studio"),
    ("cursor.exe",            "Cursor"),
    ("windsurf.exe",          "Windsurf"),
    ("sublime_text.exe",      "Sublime Text"),
    ("atom.exe",              "Atom"),
    ("spotify.exe",           "Spotify"),
    ("vlc.exe",               "VLC"),
    ("obs64.exe",             "OBS Studio"),
    ("postman.exe",           "Postman"),
    ("insomnia.exe",          "Insomnia"),
    ("TablePlus.exe",         "TablePlus"),
    ("GitKraken.exe",         "GitKraken"),
    ("SourceTree.exe",        "SourceTree"),
];

// Anything not in the map → use the exe filename without extension, title-cased
```

---

## 9. VK Code → Key Name Map

Full map needed in `hook.rs` for `vk_to_name(vk: u16) -> &'static str`:

```
0x08 → "Backspace", 0x09 → "Tab", 0x0D → "Enter", 0x1B → "Esc",
0x20 → "Space", 0x21 → "PgUp", 0x22 → "PgDn", 0x23 → "End",
0x24 → "Home", 0x25 → "←", 0x26 → "↑", 0x27 → "→", 0x28 → "↓",
0x2D → "Insert", 0x2E → "Delete",
0x30–0x39 → "0"–"9",
0x41–0x5A → "A"–"Z",
0x60–0x69 → "Num0"–"Num9",
0x6A → "Num*", 0x6B → "Num+", 0x6D → "Num-", 0x6E → "Num.",
0x6F → "Num/",
0x70–0x7B → "F1"–"F12",
0x7C–0x87 → "F13"–"F24",
0xBA → ";", 0xBB → "=", 0xBC → ",", 0xBD → "-", 0xBE → ".",
0xBF → "/", 0xC0 → "`", 0xDB → "[", 0xDC → "\\", 0xDD → "]",
0xDE → "'",
_ → format "VK({:#X})"
```

---

## 10. Analytics Engine — `analytics.rs`

### Efficiency Score (0–100)
```
score = clamp(
  (unique_shortcuts_today / 10.0) * 40   // up to 40pts for variety
  + (total_shortcuts_today / 50.0) * 30  // up to 30pts for volume
  + (streak_days / 7.0) * 20            // up to 20pts for consistency
  + gap_bonus                            // +10pts if used any gap shortcut today
, 0, 100)
```

### Streak Logic
On each shortcut insert, call `update_streak(today_date)`:
- If `last_active_date` == today → no change
- If `last_active_date` == yesterday → `current_streak += 1`, update longest if needed
- If gap > 1 day → `current_streak = 1`
- Update `last_active_date = today`

### Milestones to Detect
Check after each shortcut insert. Emit Tauri notification if newly achieved.

| Key | Condition | Label | Description |
|---|---|---|---|
| `first_shortcut` | total shortcuts == 1 | First Key! | You fired your first shortcut |
| `ten_unique` | unique shortcuts >= 10 | 10 Combos Unlocked | You know 10 unique shortcuts |
| `fifty_unique` | unique shortcuts >= 50 | Shortcut Veteran | 50 unique combos in your arsenal |
| `hundred_unique` | unique shortcuts >= 100 | Keyboard Master | 100 shortcuts unlocked |
| `streak_7` | current streak >= 7 | Week Warrior | 7-day streak |
| `streak_30` | current streak >= 30 | Monthly Legend | 30-day streak |
| `first_gap_unlock` | first gap shortcut used | Gap Closer | You used a recommended shortcut |
| `score_90` | efficiency score >= 90 | Elite Reflexes | Efficiency score above 90 |
| `vscode_power` | VS Code unique shortcuts >= 15 | VS Code Wizard | 15+ VS Code shortcuts |
| `no_mouse_day` | mouse_escapes == 0 for a day (future) | Mouseless | Full day, keyboard only |

---

## 11. Gap Analysis Data — `gaps-data.ts` (Frontend) + `gaps.rs` (Backend)

### The Known-Shortcuts Database
Build as a static array in `src/lib/gaps-data.ts`. Minimum 60 entries covering VS Code, Chrome, Windows system, and general. Structure:

```typescript
export interface KnownShortcut {
  shortcut: string;          // Normalized: "Ctrl+Shift+P"
  app: string;               // "VS Code" | "Chrome" | "Windows" | "General"
  description: string;       // "Open Command Palette"
  powerUserPct: number;      // 0–100
  priority: 1 | 2 | 3;      // 1=high, 2=medium, 3=advanced
  category: string;          // "Navigation" | "Editing" | "Search" | etc.
  tip?: string;              // Optional one-liner on how to use it
}

export const KNOWN_SHORTCUTS: KnownShortcut[] = [
  // ── VS Code ─────────────────────────────────────────────────────────
  { shortcut: "Ctrl+Shift+P", app: "VS Code", description: "Command Palette", powerUserPct: 94, priority: 1, category: "Navigation", tip: "Your most powerful shortcut. Opens everything." },
  { shortcut: "Ctrl+P",       app: "VS Code", description: "Quick Open file by name", powerUserPct: 91, priority: 1, category: "Navigation" },
  { shortcut: "Ctrl+`",       app: "VS Code", description: "Toggle integrated terminal", powerUserPct: 88, priority: 1, category: "Navigation", tip: "Stop switching windows. Terminal lives here." },
  { shortcut: "Ctrl+B",       app: "VS Code", description: "Toggle sidebar", powerUserPct: 75, priority: 1, category: "Navigation" },
  { shortcut: "Alt+↑",        app: "VS Code", description: "Move line up", powerUserPct: 70, priority: 1, category: "Editing", tip: "Replaces cut → move → paste entirely." },
  { shortcut: "Alt+↓",        app: "VS Code", description: "Move line down", powerUserPct: 70, priority: 1, category: "Editing" },
  { shortcut: "Ctrl+D",       app: "VS Code", description: "Select next occurrence", powerUserPct: 82, priority: 1, category: "Editing", tip: "Multi-cursor magic. Press repeatedly." },
  { shortcut: "Ctrl+Shift+K", app: "VS Code", description: "Delete line", powerUserPct: 68, priority: 1, category: "Editing" },
  { shortcut: "Ctrl+/",       app: "VS Code", description: "Toggle line comment", powerUserPct: 85, priority: 1, category: "Editing" },
  { shortcut: "Ctrl+Shift+L", app: "VS Code", description: "Select all occurrences", powerUserPct: 60, priority: 2, category: "Editing" },
  { shortcut: "Ctrl+G",       app: "VS Code", description: "Go to line", powerUserPct: 55, priority: 2, category: "Navigation" },
  { shortcut: "Ctrl+Shift+F", app: "VS Code", description: "Search across all files", powerUserPct: 78, priority: 1, category: "Search" },
  { shortcut: "F12",          app: "VS Code", description: "Go to definition", powerUserPct: 80, priority: 1, category: "Navigation" },
  { shortcut: "Alt+F12",      app: "VS Code", description: "Peek definition", powerUserPct: 45, priority: 2, category: "Navigation" },
  { shortcut: "Ctrl+Shift+O", app: "VS Code", description: "Go to symbol in file", powerUserPct: 50, priority: 2, category: "Navigation" },
  { shortcut: "Ctrl+K+Ctrl+F",app: "VS Code", description: "Format selection", powerUserPct: 55, priority: 2, category: "Editing" },
  { shortcut: "Ctrl+Shift+V", app: "VS Code", description: "Markdown preview", powerUserPct: 40, priority: 3, category: "View" },
  { shortcut: "Ctrl+\\",      app: "VS Code", description: "Split editor", powerUserPct: 62, priority: 2, category: "Navigation" },
  { shortcut: "Ctrl+W",       app: "VS Code", description: "Close tab", powerUserPct: 88, priority: 1, category: "Navigation" },
  { shortcut: "Ctrl+Tab",     app: "VS Code", description: "Cycle open editors", powerUserPct: 75, priority: 1, category: "Navigation" },

  // ── Chrome ──────────────────────────────────────────────────────────
  { shortcut: "Ctrl+L",       app: "Chrome", description: "Focus address bar", powerUserPct: 72, priority: 1, category: "Navigation", tip: "Faster than clicking the URL bar." },
  { shortcut: "Ctrl+T",       app: "Chrome", description: "New tab", powerUserPct: 95, priority: 1, category: "Navigation" },
  { shortcut: "Ctrl+W",       app: "Chrome", description: "Close tab", powerUserPct: 92, priority: 1, category: "Navigation" },
  { shortcut: "Ctrl+Shift+T", app: "Chrome", description: "Reopen closed tab", powerUserPct: 80, priority: 1, category: "Navigation" },
  { shortcut: "Ctrl+Tab",     app: "Chrome", description: "Next tab", powerUserPct: 85, priority: 1, category: "Navigation" },
  { shortcut: "Ctrl+Shift+Tab", app: "Chrome", description: "Previous tab", powerUserPct: 65, priority: 1, category: "Navigation" },
  { shortcut: "Ctrl+Shift+J", app: "Chrome", description: "Open DevTools Console", powerUserPct: 60, priority: 2, category: "Dev" },
  { shortcut: "Ctrl+Shift+I", app: "Chrome", description: "Open DevTools", powerUserPct: 65, priority: 2, category: "Dev" },
  { shortcut: "Ctrl+Shift+N", app: "Chrome", description: "New incognito window", powerUserPct: 70, priority: 2, category: "Navigation" },
  { shortcut: "Ctrl+F",       app: "Chrome", description: "Find on page", powerUserPct: 90, priority: 1, category: "Search" },
  { shortcut: "Ctrl+R",       app: "Chrome", description: "Reload page", powerUserPct: 93, priority: 1, category: "Navigation" },
  { shortcut: "Ctrl+Shift+R", app: "Chrome", description: "Hard reload (no cache)", powerUserPct: 55, priority: 2, category: "Navigation" },
  { shortcut: "Ctrl+D",       app: "Chrome", description: "Bookmark page", powerUserPct: 60, priority: 2, category: "Navigation" },
  { shortcut: "Alt+←",        app: "Chrome", description: "Go back", powerUserPct: 75, priority: 1, category: "Navigation" },
  { shortcut: "Alt+→",        app: "Chrome", description: "Go forward", powerUserPct: 65, priority: 1, category: "Navigation" },

  // ── Windows System ───────────────────────────────────────────────────
  { shortcut: "Win+D",        app: "Windows", description: "Show/hide desktop", powerUserPct: 60, priority: 1, category: "Window", tip: "Instant desktop. Better than minimize-all." },
  { shortcut: "Win+E",        app: "Windows", description: "Open File Explorer", powerUserPct: 75, priority: 1, category: "Navigation" },
  { shortcut: "Win+L",        app: "Windows", description: "Lock screen", powerUserPct: 80, priority: 1, category: "System" },
  { shortcut: "Win+V",        app: "Windows", description: "Clipboard history", powerUserPct: 38, priority: 2, category: "Editing", tip: "Most people don't know this exists." },
  { shortcut: "Win+Shift+S",  app: "Windows", description: "Screenshot snip", powerUserPct: 65, priority: 1, category: "System" },
  { shortcut: "Win+↑",        app: "Windows", description: "Maximize window", powerUserPct: 55, priority: 2, category: "Window" },
  { shortcut: "Win+←",        app: "Windows", description: "Snap window left", powerUserPct: 70, priority: 1, category: "Window" },
  { shortcut: "Win+→",        app: "Windows", description: "Snap window right", powerUserPct: 70, priority: 1, category: "Window" },
  { shortcut: "Win+Tab",      app: "Windows", description: "Task View", powerUserPct: 50, priority: 2, category: "Navigation" },
  { shortcut: "Ctrl+Shift+Esc", app: "Windows", description: "Open Task Manager", powerUserPct: 68, priority: 1, category: "System" },
  { shortcut: "Alt+F4",       app: "Windows", description: "Close window", powerUserPct: 80, priority: 1, category: "Window" },
  { shortcut: "Win+.",        app: "Windows", description: "Emoji picker", powerUserPct: 42, priority: 3, category: "Editing" },

  // ── General ──────────────────────────────────────────────────────────
  { shortcut: "Ctrl+Z",       app: "General", description: "Undo", powerUserPct: 98, priority: 1, category: "Editing" },
  { shortcut: "Ctrl+Y",       app: "General", description: "Redo", powerUserPct: 90, priority: 1, category: "Editing" },
  { shortcut: "Ctrl+Shift+Z", app: "General", description: "Redo (alt)", powerUserPct: 70, priority: 1, category: "Editing" },
  { shortcut: "Ctrl+C",       app: "General", description: "Copy", powerUserPct: 99, priority: 1, category: "Editing" },
  { shortcut: "Ctrl+X",       app: "General", description: "Cut", powerUserPct: 95, priority: 1, category: "Editing" },
  { shortcut: "Ctrl+V",       app: "General", description: "Paste", powerUserPct: 99, priority: 1, category: "Editing" },
  { shortcut: "Ctrl+Shift+V", app: "General", description: "Paste without formatting", powerUserPct: 48, priority: 2, category: "Editing", tip: "The paste you actually want 90% of the time." },
  { shortcut: "Ctrl+A",       app: "General", description: "Select all", powerUserPct: 97, priority: 1, category: "Editing" },
  { shortcut: "Ctrl+S",       app: "General", description: "Save", powerUserPct: 98, priority: 1, category: "File" },
  { shortcut: "Ctrl+F",       app: "General", description: "Find", powerUserPct: 93, priority: 1, category: "Search" },
  { shortcut: "Ctrl+H",       app: "General", description: "Find & Replace", powerUserPct: 72, priority: 1, category: "Search" },
  { shortcut: "Ctrl+N",       app: "General", description: "New", powerUserPct: 85, priority: 1, category: "File" },
  { shortcut: "Ctrl+O",       app: "General", description: "Open", powerUserPct: 82, priority: 1, category: "File" },
  { shortcut: "Ctrl+P",       app: "General", description: "Print", powerUserPct: 60, priority: 2, category: "File" },
  { shortcut: "Ctrl+Home",    app: "General", description: "Jump to top of document", powerUserPct: 55, priority: 2, category: "Navigation" },
  { shortcut: "Ctrl+End",     app: "General", description: "Jump to bottom", powerUserPct: 55, priority: 2, category: "Navigation" },
];
```

### Gap Analysis Logic (Frontend + Backend)
The gap analysis merges two datasets:
1. `KNOWN_SHORTCUTS` — static, bundled
2. User's `unique_shortcuts` from DB — what they've actually pressed

A shortcut is a **gap** if it appears in KNOWN_SHORTCUTS for the user's tracked apps and has never appeared in the DB. Sort gaps by `priority` ASC, then `powerUserPct` DESC. Show top 20.

"Unlocked" in gap context = user has pressed this shortcut at least once. Show it as dimmed/achieved in the gap list.

---

## 12. Tauri Commands — `commands.rs`

All commands registered in `main.rs` via `.invoke_handler(tauri::generate_handler![...])`.

```rust
#[tauri::command]
async fn get_top_shortcuts(
    state: tauri::State<'_, DbState>,
    app: String,
    days: u32,
    limit: u32,
) -> Result<Vec<ShortcutEntry>, String>

#[tauri::command]
async fn get_heatmap_data(
    state: tauri::State<'_, DbState>,
    app: String,
    days: u32,
) -> Result<HashMap<String, u64>, String>
// Returns map of key_name → count, e.g. {"C": 143, "Shift": 89, "Ctrl": 210}

#[tauri::command]
async fn get_apps_list(
    state: tauri::State<'_, DbState>,
) -> Result<Vec<String>, String>
// Returns ["All Apps", "VS Code", "Chrome", ...] with "All Apps" always first

#[tauri::command]
async fn get_today_stats(
    state: tauri::State<'_, DbState>,
    app: String,
) -> Result<TodayStats, String>

#[tauri::command]
async fn get_efficiency_score(
    state: tauri::State<'_, DbState>,
) -> Result<EfficiencyScore, String>

#[tauri::command]
async fn get_streak(
    state: tauri::State<'_, DbState>,
) -> Result<StreakData, String>

#[tauri::command]
async fn get_day_activity(
    state: tauri::State<'_, DbState>,
    days: u32,
) -> Result<Vec<DayActivity>, String>
// Used for History tab contribution grid. days=365 for full year.

#[tauri::command]
async fn get_unique_shortcuts(
    state: tauri::State<'_, DbState>,
    app: String,
) -> Result<Vec<String>, String>
// Used by gap analysis to diff against KNOWN_SHORTCUTS

#[tauri::command]
async fn get_milestones(
    state: tauri::State<'_, DbState>,
) -> Result<Vec<MilestoneRecord>, String>

#[tauri::command]
async fn get_setting(
    state: tauri::State<'_, DbState>,
    key: String,
) -> Result<String, String>

#[tauri::command]
async fn set_setting(
    state: tauri::State<'_, DbState>,
    key: String,
    value: String,
) -> Result<(), String>

#[tauri::command]
async fn get_shortcut_of_day(
    state: tauri::State<'_, DbState>,
) -> Result<GapItem, String>
// Picks one gap item not yet unlocked, deterministic per calendar day
// Algorithm: seed = day_of_year, pick index = seed % gaps.len()

#[tauri::command]
async fn get_per_app_breakdown(
    state: tauri::State<'_, DbState>,
    app: String,
    days: u32,
) -> Result<Vec<ShortcutEntry>, String>
```

Emit these Tauri events from Rust to frontend:
- `shortcut-recorded` → payload `{ shortcut: string, app: string }` — fires on every hook event
- `milestone-achieved` → payload `MilestoneRecord` — fires when a new milestone is detected
- `streak-updated` → payload `StreakData`

---

## 13. Frontend — IPC Layer (`src/lib/ipc.ts`)

```typescript
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export const api = {
  getTopShortcuts:     (app: string, days: number, limit: number) =>
    invoke<ShortcutEntry[]>("get_top_shortcuts", { app, days, limit }),
  getHeatmapData:      (app: string, days: number) =>
    invoke<Record<string, number>>("get_heatmap_data", { app, days }),
  getAppsList:         () => invoke<string[]>("get_apps_list"),
  getTodayStats:       (app: string) => invoke<TodayStats>("get_today_stats", { app }),
  getEfficiencyScore:  () => invoke<EfficiencyScore>("get_efficiency_score"),
  getStreak:           () => invoke<StreakData>("get_streak"),
  getDayActivity:      (days: number) => invoke<DayActivity[]>("get_day_activity", { days }),
  getUniqueShortcuts:  (app: string) => invoke<string[]>("get_unique_shortcuts", { app }),
  getMilestones:       () => invoke<MilestoneRecord[]>("get_milestones"),
  getSetting:          (key: string) => invoke<string>("get_setting", { key }),
  setSetting:          (key: string, value: string) => invoke<void>("set_setting", { key, value }),
  getShortcutOfDay:    () => invoke<GapItem>("get_shortcut_of_day"),
  getPerAppBreakdown:  (app: string, days: number) =>
    invoke<ShortcutEntry[]>("get_per_app_breakdown", { app, days }),
};

export const onShortcutRecorded = (cb: (data: {shortcut: string, app: string}) => void) =>
  listen("shortcut-recorded", (e) => cb(e.payload as any));

export const onMilestoneAchieved = (cb: (data: MilestoneRecord) => void) =>
  listen("milestone-achieved", (e) => cb(e.payload as any));
```

---

## 14. Custom Titlebar

Since `decorations: false`, implement a custom titlebar in `Layout.tsx`:

```tsx
// Top bar: 40px height, background var(--surface), border-bottom var(--border)
// Left: Keyflex logo (amber keycap SVG icon, 20px) + "Keyflex" in Space Grotesk
// Center (or right of logo): current app filter dropdown
// Right: minimize button, maximize/restore button, close button
// The entire bar must have `data-tauri-drag-region` attribute so it can be dragged
// Window control buttons use Tauri's window API:
//   appWindow.minimize(), appWindow.toggleMaximize(), appWindow.close()
// Close button: close window but keep hook running (app stays in tray)
//   Use appWindow.hide() not appWindow.close() — don't kill the process
```

---

## 15. UI — Tab by Tab

### 15.1 Overview Tab

Layout: 2-column grid (left: score + streak, right: stats + top shortcuts)

**Left column:**
- `ScoreRing` component: SVG circle, 100px diameter, stroke gradient ember→flame, percentage fill based on score, centered number in Space Grotesk bold
- Below ring: `score / 100`, three rows of sub-stats (shortcuts today, unique today, mouse escapes)
- `StreakDisplay` component: flame emoji (scaled by streak length, min 1em max 3em), streak number in flame color Space Grotesk 48px, "days in a row", 7 dots (Mon–Sun) — done/today/pending states

**Right column:**
- `TodayStats` row: three `StatCard` components (total shortcuts, unique combos, new unlocks today)
- "Your Top Shortcuts Today" section: ordered list of top 5, each row = `ShortcutRow` + count badge + bar (width = count/max * 100%)
- "Shortcut of the Day" card: amber-bordered card, `GapItem` data, keycap chips, description, power user pct, "Try it!" CTA label

All data fetched on mount via `useEffect` + `api.*` calls. Subscribe to `onShortcutRecorded` to refresh stats live as shortcuts are pressed.

### 15.2 Heatmap Tab

Full keyboard SVG — hardcode the QWERTY layout as a React component. Each key is a `<rect>` or `<g>` with a key name. Heat level (0–5) is computed from `heatmap_data[key_name]`:
- 0: never used
- 1: 1–5 times
- 2: 6–20 times
- 3: 21–50 times
- 4: 51–150 times
- 5: 150+ times

CSS classes `h0`–`h5` map to the ember gradient colors (as defined in brand section).

Controls above keyboard:
- App filter dropdown (same as global, but local to this tab)
- Time range: Today / 7 days / 30 days / All time (radio or segmented control)

On hover of any key: tooltip showing top 3 shortcuts that included this key + total count.

Heat legend below keyboard (left = never, right = blazing, 6 swatches).

The keyboard layout must include all standard rows:
- Row 1: ` 1 2 3 4 5 6 7 8 9 0 - = Backspace`
- Row 2: Tab Q W E R T Y U I O P [ ] \
- Row 3: CapsLock A S D F G H J K L ; ' Enter
- Row 4: Shift(L) Z X C V B N M , . / Shift(R)
- Row 5: Ctrl(L) Win Alt Space Alt Win Menu Ctrl(R)

Use relative units so keyboard scales with panel width.

### 15.3 Per App Tab

Top: app selector (large tabs or dropdown showing all tracked apps + icons if possible)

For selected app:
- Bar chart (Recharts `BarChart`): top 10 shortcuts on Y axis, count on X. Use `--ember` fill color. Horizontal bars. Custom tooltip showing full shortcut + count.
- Line chart (Recharts `LineChart`): last 7 days activity for this app. X = date, Y = shortcut count. Stroke `--ember`, fill gradient `--ember-glow`.
- Full sorted list below charts: `ShortcutRow` + count badge for all shortcuts ever used in this app.

### 15.4 Gap Analysis Tab

Header: "Level Up Your Shortcuts" — Space Grotesk 24px

Filter row: All Apps | VS Code | Chrome | Windows | General (tab pills)

For each gap item (not yet unlocked, sorted by priority then powerUserPct desc):
```
┌──────────────────────────────────────────────────────┐
│  [Ctrl] + [Shift] + [P]    VS Code                   │
│  Open Command Palette                                 │
│  ████████████░░  94% of power users                  │
│  "Your most powerful shortcut. Opens everything."    │
│                                          [UNLOCK →]  │
└──────────────────────────────────────────────────────┘
```
- `[UNLOCK →]` button: amber border, on click — expand card to show full tip + dismiss (mark acknowledged in localStorage). Does NOT mark as achieved — only actually pressing it does.
- Achieved shortcuts (user has pressed them): show dimmed with `✓ Unlocked` badge instead of button
- Locked (priority 3, user hasn't unlocked priority 1+2 yet): show greyed with `🔒 Locked` badge

### 15.5 History Tab

**GitHub-style contribution grid:**
- 52 columns (weeks) × 7 rows (days), cells are 12×12px with 2px gap
- Color levels 0–4 mapped to: `#1A1A1A` → increasing ember intensity
- Click any cell → show popover: date, shortcut count, top 3 shortcuts that day
- Hover → tooltip with date + count

**Summary stats above grid:**
- Total shortcuts this year, most active day, current streak, longest streak

**Monthly labels** above columns (Jan, Feb, …)

---

## 16. Onboarding Component

Shown on first launch (`settings.onboarding_done == "false"`). Full-screen overlay (z-index 9999) on top of the app.

```
┌────────────────────────────────────────────────────────┐
│                                                        │
│           [Amber keycap icon, 80px]                   │
│                                                        │
│               Welcome to Keyflex                      │
│      Flex your keys. Own your machine.                │
│                                                        │
│  ✓  Tracks only keyboard shortcuts (Ctrl+C, not text) │
│  ✓  All data stays on your machine. No cloud.         │
│  ✓  Starts with Windows, lives in your system tray    │
│  ✓  Shows you shortcuts you're missing — and why      │
│                                                        │
│            [ Start Tracking → ]                       │
│                                                        │
│        (amber CTA button, 48px height, ember bg)      │
└────────────────────────────────────────────────────────┘
```

On "Start Tracking": `api.setSetting("onboarding_done", "true")`, hide overlay, show tray notification "Keyflex is watching — press some shortcuts!"

---

## 17. System Tray

In `main.rs`, set up tray with these menu items:

```
Keyflex                    (title, non-clickable, bold)
─────────────────────────
Open Dashboard
─────────────────────────
📅 Today: 0 shortcuts      (updated dynamically)
🔥 Streak: 0 days
⚡ Score: --
─────────────────────────
💡 Shortcut of the Day: ...  (truncated to 30 chars)
─────────────────────────
Start with Windows ✓       (toggle, checkmark if enabled)
─────────────────────────
Quit Keyflex
```

Left-click on tray icon → show/toggle main window.
Right-click → context menu (above).

Update tray tooltip every 5 minutes from DB stats.

---

## 18. Notifications

Use `tauri-plugin-notification`. Fire notifications for:

1. **App startup** (if `onboarding_done == true`):
   - Title: `Keyflex is watching 👀`
   - Body: `Today's tip: {shortcut_of_day.shortcut} — {shortcut_of_day.description}`
   - Fire with 3-second delay after startup

2. **Milestone achieved** (from Rust, via event):
   - Title: `🏆 {milestone.label}`
   - Body: `{milestone.description}`

3. **Streak milestone** (7, 14, 30, 60, 100 days):
   - Title: `🔥 {n}-Day Streak!`
   - Body: `You've been flexing your keys for {n} days straight.`

---

## 19. Autostart

Use `tauri-plugin-autostart`. On first launch, enable autostart by default. Toggle via tray menu "Start with Windows" item. Persist preference in settings table.

---

## 20. CSS — `globals.css`

```css
@import url('https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@300;400;500;600;700&family=Inter:wght@300;400;500;600&family=JetBrains+Mono:wght@400;500;600&display=swap');

*, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }

:root {
  --bg:            #0D0D0D;
  --surface:       #161616;
  --surface2:      #1E1E1E;
  --border:        #2A2A2A;
  --ember:         #F5A623;
  --ember-glow:    rgba(245, 166, 35, 0.15);
  --flame:         #FF6B35;
  --ash:           #E8E8E8;
  --smoke:         #666666;
  --unlocked:      #4ADE80;
  --unlocked-glow: rgba(74, 222, 128, 0.12);
  --radius-sm:     6px;
  --radius-md:     10px;
  --radius-lg:     14px;
  --font-display:  'Space Grotesk', sans-serif;
  --font-ui:       'Inter', sans-serif;
  --font-mono:     'JetBrains Mono', monospace;
}

html, body, #root { height: 100%; background: var(--bg); color: var(--ash); }

body {
  font-family: var(--font-ui);
  font-size: 14px;
  line-height: 1.5;
  -webkit-font-smoothing: antialiased;
  user-select: none;
  overflow: hidden;
}

/* Scrollbar */
::-webkit-scrollbar { width: 6px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: var(--border); border-radius: 3px; }
::-webkit-scrollbar-thumb:hover { background: var(--smoke); }

/* Section labels */
.label-mono {
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: var(--smoke);
}

/* Transitions */
* { transition: background-color 0.15s ease, border-color 0.15s ease, color 0.15s ease; }
button, [role="button"] { cursor: pointer; }
```

---

## 21. Keyboard Layout CSS — `keyboard.css`

```css
.kb-row { display: flex; gap: 4px; margin-bottom: 4px; justify-content: center; }

.kb-key {
  display: flex; align-items: center; justify-content: center;
  height: 36px; min-width: 36px;
  border-radius: 5px;
  font-family: var(--font-mono); font-size: 9px; font-weight: 500;
  color: rgba(232,232,232,0.5);
  border: 1px solid rgba(255,255,255,0.05);
  cursor: pointer;
  transition: all 0.2s ease;
  flex-shrink: 0;
  position: relative;
}

.kb-key:hover { transform: translateY(-1px); filter: brightness(1.2); }

/* Width variants */
.kb-w-1    { min-width: 36px; }
.kb-w-15   { min-width: 54px; }
.kb-w-20   { min-width: 72px; }
.kb-w-25   { min-width: 90px; }
.kb-w-sp   { min-width: 200px; }

/* Heat levels */
.h0 { background: #1A1A1A; }
.h1 { background: rgba(245,166,35,0.08); color: rgba(245,166,35,0.45); }
.h2 { background: rgba(245,166,35,0.18); color: rgba(245,166,35,0.75); border-color: rgba(245,166,35,0.2); }
.h3 { background: rgba(245,166,35,0.32); color: #F5A623; border-color: rgba(245,166,35,0.45); box-shadow: 0 0 8px rgba(245,166,35,0.2); }
.h4 { background: rgba(255,107,53,0.38); color: #FF6B35; border-color: rgba(255,107,53,0.55); box-shadow: 0 0 14px rgba(255,107,53,0.25); }
.h5 { background: rgba(255,70,20,0.60);  color: #fff;    border-color: rgba(255,70,20,0.75);  box-shadow: 0 0 20px rgba(255,70,20,0.4); }
```

---

## 22. `ShortcutRow.tsx` — Key Parsing Component

```tsx
// Parses "Ctrl+Shift+P" → renders [KeyChip]Ctrl[/KeyChip] + [KeyChip]Shift[/KeyChip] + [KeyChip]P[/KeyChip]
// Between chips: render a small "+" in var(--smoke) color
// Props: shortcut (string), count? (number), showBar? (boolean), maxCount? (number)
// If showBar: render a thin progress bar (width = count/maxCount * 100%) in ember color below chips
```

---

## 23. `ScoreRing.tsx`

SVG circle ring. Props: `score: number` (0–100).
- Outer circle: stroke `var(--border)`, stroke-width 6
- Inner arc: stroke `url(#scoreGrad)` (linear gradient ember→flame), stroke-width 6, stroke-linecap round
- Dasharray = `2 * PI * r = 2 * 3.14159 * 40 = 251.3`
- Dashoffset = `251.3 * (1 - score/100)`
- Rotate -90deg so arc starts at top
- Centered text: score number in Space Grotesk 700, color ember
- Sub-text: "/ 100" in mono smoke

---

## 24. Recharts Theming

All charts must use the Ember Dark palette:
- `fill`: `var(--ember)` for bars
- `stroke`: `var(--ember)` for lines
- Area fill: `var(--ember-glow)`
- Grid lines: `var(--border)`
- Axis text: `var(--smoke)` via `tick={{ fill: 'var(--smoke)', fontSize: 11 }}`
- Tooltip: custom component with `--surface2` background, `--border` border, `--ash` text, keycap chips for shortcut display
- No default recharts styling — override everything

---

## 25. `main.rs` — Full Setup

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .manage(DbState(Arc::new(Mutex::new(
            db::init(&get_app_data_dir()).expect("DB init failed")
        ))))
        .invoke_handler(tauri::generate_handler![
            commands::get_top_shortcuts,
            commands::get_heatmap_data,
            commands::get_apps_list,
            commands::get_today_stats,
            commands::get_efficiency_score,
            commands::get_streak,
            commands::get_day_activity,
            commands::get_unique_shortcuts,
            commands::get_milestones,
            commands::get_setting,
            commands::set_setting,
            commands::get_shortcut_of_day,
            commands::get_per_app_breakdown,
        ])
        .setup(|app| {
            // 1. Start keyboard hook in background thread
            let (tx, rx) = std::sync::mpsc::channel::<String>();
            hook::start_hook(tx);

            // 2. Spawn receiver task
            let db_state = app.state::<DbState>().inner().clone();
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                while let Ok(shortcut) = rx.recv() {
                    let app_name = apps::get_foreground_app().0;
                    // Skip if app is "Keyflex" itself
                    if app_name == "Keyflex" { continue; }
                    let conn = db_state.0.lock();
                    let _ = db::insert_shortcut(&conn, &shortcut, &app_name, "");
                    let streak = db::update_streak(&conn, &today_date_string()).ok();
                    let milestones = analytics::check_milestones(&conn, &shortcut, &app_name);
                    
                    // Emit events
                    let _ = app_handle.emit("shortcut-recorded", json!({ "shortcut": shortcut, "app": app_name }));
                    for m in milestones {
                        let _ = app_handle.emit("milestone-achieved", &m);
                        send_notification(&app_handle, &format!("🏆 {}", m.label), &m.description);
                    }
                    if let Some(s) = streak {
                        let _ = app_handle.emit("streak-updated", &s);
                    }
                }
            });

            // 3. Setup tray
            setup_tray(app)?;

            // 4. Send startup notification (3s delay)
            // 5. Check autostart setting and apply
            
            Ok(())
        })
        .on_window_event(|window, event| {
            // On close-requested: hide window instead of exiting
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                window.hide().ok();
            }
        })
        .run(tauri::generate_context!())
        .expect("error running Keyflex");
}
```

---

## 26. GitHub Actions — `.github/workflows/release.yml`

```yaml
name: Build & Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release-windows:
    runs-on: windows-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache Rust
        uses: Swatinem/rust-cache@v2
        with:
          workspaces: src-tauri

      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: npm

      - name: Install deps
        run: npm ci

      - name: Build
        run: npm run tauri build

      - name: Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            src-tauri/target/release/bundle/nsis/*.exe
            src-tauri/target/release/bundle/msi/*.msi
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

---

## 27. README.md

Full content — write verbatim:

```markdown
<div align="center">

<!-- Social preview image: amber keycap icon, "Keyflex" in Space Grotesk, dark bg -->

# 🔥 Keyflex

**Flex your keys. Own your machine.**

[![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tauri v2](https://img.shields.io/badge/Tauri_v2-24C8D8?style=flat&logo=tauri&logoColor=white)](https://tauri.app)
[![TypeScript](https://img.shields.io/badge/TypeScript-3178C6?style=flat&logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![Windows](https://img.shields.io/badge/Windows-0078D4?style=flat&logo=windows&logoColor=white)](https://www.microsoft.com/windows)
[![License MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

</div>

---

I realized I'd been using computers for years but had **zero visibility** into how I actually used them. I'd reach for the mouse constantly, forget shortcuts I once knew, and have no idea which ones I was missing entirely. Every existing tool either logged everything — a privacy nightmare — or just counted keystrokes, which is useless. I wanted something that watched for the *smart stuff, the combos* — and showed me the gap between who I am at the keyboard and who I could be. So I built Keyflex.

## What it does

- **⌨️ Privacy-safe capture** — Tracks only modifier combinations (`Ctrl+C`, `Alt+Tab`). If no modifier key is held, the event is discarded immediately. Your keystrokes are never logged.
- **🔥 Live keyboard heatmap** — A full keyboard visualization where every key glows hotter the more you use it in shortcuts.
- **🔓 Gap analysis** — Surfaces high-value shortcuts you've never pressed, framed as unlockable abilities — not failures.
- **📊 Per-app breakdown** — Chrome, VS Code, Windows system, any app — tracked separately with its own charts and stats.
- **⚡ Efficiency score** — A 0–100 score combining variety, volume, and streak consistency.
- **🎮 Streaks & milestones** — Daily streaks, achievement unlocks, and a contribution grid for your shortcut history.
- **🪟 System tray** — Runs silently in the background. Opens a dashboard when you want it, disappears when you don't.

## Privacy

Keyflex's privacy is **architectural**, not a promise:

- The hook callback checks for a modifier key being held **before** any data is saved. Non-modifier keypresses are discarded in the same stack frame — no buffer, no log.
- All data is stored in a local SQLite database at `%APPDATA%/keyflex/keyflex.db`. Nothing leaves your machine.
- No telemetry, no analytics, no network calls.

## Installation

Download the latest installer from [Releases](https://github.com/HarshalPatel1972/keyflex/releases):

- `Keyflex_0.1.0_x64_en-US.msi` — Windows Installer
- `Keyflex_0.1.0_x64-setup.exe` — NSIS Installer

## Building from source

```bash
# Prerequisites: Rust stable, Node 20+
git clone https://github.com/HarshalPatel1972/keyflex.git
cd keyflex
npm install
npm run tauri dev        # development
npm run tauri build      # production installer
```

## Tech Stack

| Layer | Choice |
|---|---|
| Shell | Tauri v2 |
| Core / Hook | Rust + windows-rs |
| Storage | SQLite via rusqlite (bundled) |
| UI | React 18 + TypeScript |
| Charts | Recharts |
| Build | Vite |

## Screenshots

<!-- Add after build: Overview, Heatmap, Gap Analysis, History tabs -->

## Roadmap

- [ ] Mouse escape rate tracking (WH_MOUSE_LL frequency analysis)
- [ ] Weekly shareable stats card (PNG export)
- [ ] Custom shortcut definitions (import VS Code keybindings.json)
- [ ] macOS support
- [ ] Shortcut suggestion engine (AI-powered, local)

## Contributing

Issues and PRs welcome. Please follow conventional commits: `type(scope): description`.

---

Built with frustration and too many mouse clicks by [Harshal Patel](https://github.com/HarshalPatel1972).
```

---

## 28. CHANGELOG.md

```markdown
# Changelog

All notable changes to Keyflex are documented here.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
Versioning: [Semantic Versioning](https://semver.org/)

## [0.1.0] - 2026-05-XX

### Added
- Global `WH_KEYBOARD_LL` keyboard hook capturing modifier combinations only
- Per-app attribution via `QueryFullProcessImageName` for 35+ apps
- SQLite local database — shortcuts, streaks, milestones, settings
- Overview tab: efficiency score ring, streak display, today's top shortcuts
- Heatmap tab: full QWERTY keyboard with ember gradient heat overlay
- Per App tab: bar chart + line chart + full shortcut list per app
- Gap Analysis tab: 60+ known shortcuts across VS Code, Chrome, Windows, General — framed as unlockable abilities
- History tab: GitHub-style contribution grid (365 days)
- Shortcut of the Day — tray notification + overview card
- Milestone system: 10 achievement types with OS notifications
- Daily streak tracking with longest streak record
- System tray with live stats, shortcut of the day, window toggle
- First-run onboarding overlay
- Autostart with Windows (toggleable)
- Custom frameless window with drag region titlebar
- GitHub Actions release pipeline (NSIS + MSI)
```

---

## 29. package.json

```json
{
  "name": "keyflex",
  "version": "0.1.0",
  "private": true,
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "tauri": "tauri"
  },
  "dependencies": {
    "@tauri-apps/api": "^2",
    "@tauri-apps/plugin-notification": "^2",
    "@tauri-apps/plugin-autostart": "^2",
    "react": "^18",
    "react-dom": "^18",
    "recharts": "^2.12"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2",
    "@types/react": "^18",
    "@types/react-dom": "^18",
    "@vitejs/plugin-react": "^4",
    "typescript": "^5",
    "vite": "^5"
  }
}
```

---

## 30. App Icon SVG — `public/keyflex-icon.svg`

```svg
<svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
  <!-- Keycap body -->
  <rect x="10" y="22" width="80" height="56" rx="14" fill="#F5A623"/>
  <!-- Keycap inner recess -->
  <rect x="16" y="28" width="68" height="44" rx="10" fill="#D4891A"/>
  <!-- Reflex arc: curved path representing muscle reflex signal -->
  <path d="M25 58 Q34 30 50 46 Q66 62 75 36"
        stroke="#0D0D0D" stroke-width="7" stroke-linecap="round"
        stroke-linejoin="round" fill="none"/>
  <!-- Terminal dot at arc end -->
  <circle cx="75" cy="36" r="5.5" fill="#0D0D0D"/>
</svg>
```

Generate all required icon sizes from this SVG using `tauri icon public/keyflex-icon.svg` CLI command.

---

## 31. Build Order for Agent

Execute in this exact order. Do not skip phases. Commit after each phase using conventional commits.

**Phase 1 — Foundation**
1. Init Tauri v2 project: `npm create tauri-app@latest keyflex -- --template react-ts`
2. Replace `Cargo.toml` with spec above
3. Replace `tauri.conf.json` with spec above
4. Implement `db.rs` — all schema + all query functions
5. Implement `apps.rs` — foreground window resolver + full APP_NAME_MAP
6. Implement `hook.rs` — WH_KEYBOARD_LL hook, modifier detection, vk_to_name
7. Implement basic `main.rs` — hook startup, DB receiver thread, tray stub
8. Implement `commands.rs` — all Tauri commands wired to db functions
9. Test: launch app, press Ctrl+C in VS Code, verify row appears in DB
10. Commit: `feat(core): keyboard hook, db schema, app resolver, ipc commands`

**Phase 2 — UI Shell**
1. Implement `globals.css` + `keyboard.css` — full design system
2. Implement `Layout.tsx` — custom titlebar + sidebar + main panel routing
3. Implement `Sidebar.tsx` — nav items, mini streak indicator
4. Implement `ipc.ts` — all typed wrappers
5. Implement `KeyChip.tsx`, `ShortcutRow.tsx`, `StatCard.tsx`
6. Wire up tab routing (React state-based, no router needed)
7. Commit: `feat(ui): layout shell, design system, keychip components`

**Phase 3 — Overview + Heatmap**
1. Implement `ScoreRing.tsx`, `StreakDisplay.tsx`
2. Implement `Overview.tsx` — all sections, live updates via event subscription
3. Implement `Heatmap.tsx` — full keyboard SVG, heat computation, tooltips
4. Implement `HeatLegend.tsx`
5. Commit: `feat(ui): overview tab, heatmap tab with live data`

**Phase 4 — Per App + Gap Analysis + History**
1. Implement `gaps-data.ts` — full 60+ shortcut dataset
2. Implement `GapAnalysis.tsx` — gap diff logic, unlock/locked states
3. Implement `PerApp.tsx` — recharts bar + line charts, themed
4. Implement `History.tsx` — contribution grid, popover on click
5. Commit: `feat(ui): per-app charts, gap analysis, history grid`

**Phase 5 — Polish**
1. Implement `Onboarding.tsx`
2. Implement analytics milestone detection in `analytics.rs`
3. Wire up all notifications (startup, milestones, streaks)
4. Implement tray menu with live stats
5. Implement autostart toggle
6. Implement `AppDropdown.tsx` — global app filter, persisted in React state
7. Window close → hide (not quit)
8. Commit: `feat(polish): onboarding, milestones, notifications, autostart, tray`

**Phase 6 — Release**
1. Run `tauri icon public/keyflex-icon.svg` to generate all icon sizes
2. `npm run tauri build` — verify both NSIS and MSI are generated
3. Write `README.md` verbatim from spec
4. Write `CHANGELOG.md` verbatim from spec
5. Tag `v0.1.0`, push — GitHub Actions builds and publishes release
6. Commit: `chore(release): v0.1.0`

---

## 32. Non-Negotiable Quality Rules

1. **No placeholder UI.** Every screen must have real data from the DB on mount. If DB is empty (fresh install), show "Press some shortcuts to get started" empty states — never blank white space or `undefined`.
2. **No TypeScript `any`.** All IPC return types are fully typed via the structs in Section 6.
3. **No hardcoded colors.** Every color in React/CSS comes from CSS variables. No hex literals in component files.
4. **Hook never blocks.** The keyboard hook callback does exactly two things: detect modifier, send to channel. All DB/analytics work is on the receiver thread.
5. **Keyflex ignores its own shortcuts.** The receiver thread checks if `app_name == "Keyflex"` and skips those events to avoid feedback loops.
6. **All charts are themed.** No default Recharts gray/blue. Every chart uses the Ember Dark palette as defined in Section 24.
7. **Keycap chips everywhere.** No shortcut is ever shown as plain text. All shortcut display uses `ShortcutRow` component.
8. **Font loading.** Google Fonts link must be in `index.html <head>`. Do not use `@font-face` with local files. Do not let any text fall back to system fonts.
9. **Window is 1100×720, min 900×600.** The layout must work at minimum size without horizontal scrolling.
10. **Commits are atomic.** One commit per logical unit. Format: `type(scope): description`. No merge commits. No "fix stuff" commits.
