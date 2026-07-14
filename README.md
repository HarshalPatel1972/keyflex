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
