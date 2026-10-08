# Changelog

All notable changes to Keyflex are documented here.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
Versioning: [Semantic Versioning](https://semver.org/)

## [0.2.0] - Unreleased

Keyflex is now a different app. It no longer tracks and scores shortcut usage;
it notices when you do something the long way and shows the shortcut for it.

### Added
- Tips that appear beside the cursor when a manual action has a shortcut
- 44 tips: universal commands in every app, browsers, File Explorer, Windows,
  Microsoft Office, Notepad and VS Code
- A rule engine driven by `core/tips.toml`: single clicks, ordered multi-step
  sequences, "only while typing" and "opened with the mouse" triggers
- The Keyflex character, with six moods: tips escalate over three showings,
  and using a taught shortcut is celebrated
- A tray icon that wears the character's mood
- Limits on how often tips appear: a daily cap, a quiet gap, growing cooldowns,
  and retirement once a shortcut is learned
- "Don't show again" on every tip, per-app switches and Pause
- Home, Shortcuts, How it works and Settings screens
- Light and dark themes, following Windows by default
- Microsoft Store (MSIX) packaging, including Store-compatible start with Windows
- A notice when Windows is not in English, since tips match English names
- A privacy policy

### Changed
- Click inspection reads only the names of buttons and menu items in an app's
  own interface; page content, documents, tabs and file names are never read
- Only the shortcuts named in the tip file leave the keyboard hook
- Tip history is one JSON file instead of a SQLite database
- The window is created when opened and destroyed when closed

### Removed
- Usage tracking, the keyboard heatmap, the efficiency score, streaks,
  milestones and the per-app charts
- The SQLite database
- Web font downloads: the app makes no network connections

### Fixed
- AltGr typing on international keyboards is no longer mistaken for shortcuts
- Holding a key down counts as one press, not one per repeat

## [0.1.0] - 2026-07-15

Initial release: a shortcut-usage tracker with a heatmap, per-app statistics,
an efficiency score, streaks and milestones.
