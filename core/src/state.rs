//! What Keyflex remembers between runs: per tip, how often it was shown and
//! whether the user has picked the shortcut up. A handful of numbers, so it is
//! one small JSON file rather than a database.

use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TipState {
    /// Times the tip has been shown.
    pub shown: u32,
    /// When it was last shown, seconds since the Unix epoch.
    pub last_shown: u64,
    /// Times the user pressed the shortcut (counting stops once it is learned).
    pub used: u32,
    /// The user asked not to see this tip again.
    pub muted: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    /// Follow the Windows light/dark setting.
    #[default]
    System,
    Light,
    Dark,
}

/// What the user can change on the Settings screen.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Stop watching entirely until switched back on.
    pub paused: bool,
    /// Most tips in any 24 hours.
    pub tips_per_day: usize,
    /// Apps (lowercase exe names) the user switched tips off for.
    pub disabled_apps: BTreeSet<String>,
    /// The first-launch walkthrough has been watched or skipped.
    pub intro_seen: bool,
    /// Colours of the window and the tip popup.
    pub theme: Theme,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            paused: false,
            tips_per_day: 5,
            disabled_apps: BTreeSet::new(),
            intro_seen: false,
            theme: Theme::System,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct State {
    pub settings: Settings,
    pub tips: BTreeMap<String, TipState>,
    /// Times of tips shown in the last 24 hours, oldest first.
    pub recent_shows: Vec<u64>,
    /// How many times we have celebrated a shortcut being used.
    pub cheers: u32,
    /// When we last celebrated, seconds since the Unix epoch.
    pub last_cheer: u64,
}

impl State {
    /// `%APPDATA%\Keyflex\state.json`
    pub fn default_path() -> Option<PathBuf> {
        Some(PathBuf::from(std::env::var_os("APPDATA")?).join("Keyflex").join("state.json"))
    }

    /// A missing file is a fresh start. So is an unreadable one, but that file
    /// is first set aside as `state.unreadable.json` rather than overwritten,
    /// so the history in it is not lost for good.
    pub fn load(path: &Path) -> State {
        let Ok(bytes) = std::fs::read(path) else { return State::default() };
        // Some editors put a byte-order mark at the start; JSON does not allow one.
        let json = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes);
        serde_json::from_slice(json).unwrap_or_else(|_| {
            let _ = std::fs::rename(path, path.with_extension("unreadable.json"));
            State::default()
        })
    }

    /// Written to a temporary file first so a crash cannot leave half a file.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let temp = path.with_extension("json.tmp");
        std::fs::write(&temp, serde_json::to_vec_pretty(self)?)?;
        std::fs::rename(temp, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saves_and_loads() {
        let dir = std::env::temp_dir().join(format!("keyflex-state-test-{}", std::process::id()));
        let path = dir.join("state.json");

        assert_eq!(State::load(&path), State::default());

        let mut state = State::default();
        state.tips.insert("a".into(), TipState { shown: 2, last_shown: 99, used: 1, muted: true });
        state.recent_shows.push(99);
        state.settings.tips_per_day = 2;
        state.settings.disabled_apps.insert("chrome.exe".into());
        state.save(&path).unwrap();
        state.save(&path).unwrap(); // overwriting an existing file works
        assert_eq!(State::load(&path), state);

        // A byte-order mark in front is tolerated.
        let mut with_mark = b"\xEF\xBB\xBF".to_vec();
        with_mark.extend(std::fs::read(&path).unwrap());
        std::fs::write(&path, with_mark).unwrap();
        assert_eq!(State::load(&path), state);

        // Anything unreadable is a fresh start, with the old file kept aside.
        std::fs::write(&path, b"not json").unwrap();
        assert_eq!(State::load(&path), State::default());
        assert!(!path.exists());
        assert_eq!(std::fs::read(dir.join("state.unreadable.json")).unwrap(), b"not json");

        std::fs::remove_dir_all(dir).unwrap();
    }
}
