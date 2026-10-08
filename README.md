<div align="center">

<img src="public/keyflex-icon.svg" width="96" alt="The Keyflex character: a green keycap with a raised eyebrow" />

# Keyflex

**There's a shorter way. Keyflex shows you, right when you need it.**

[![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tauri v2](https://img.shields.io/badge/Tauri_v2-24C8D8?style=flat&logo=tauri&logoColor=white)](https://tauri.app)
[![Windows](https://img.shields.io/badge/Windows-0078D4?style=flat&logo=windows&logoColor=white)](https://www.microsoft.com/windows)
[![License MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

</div>

---

Keyflex sits in your system tray and watches for the moment you do something
the long way: opening Downloads through the browser menu, right-clicking to
copy, deleting a file and then emptying it from the Recycle Bin. Right then, a
small popup appears beside your cursor with the shortcut that does the same
thing in one move.

It is not a cheat sheet and it does not track or score you. It teaches one
shortcut at a time, at the moment it is useful, and then gets out of the way.

## How it works

1. **You take the long way.** Menu, then Downloads.
2. **Keyflex notices.** It sees which button or menu item you clicked. Not what
   you type, not what you read.
3. **A tip appears right there.** Beside your cursor, with the keys and one line.
4. **Next time, two keys.** Use the shortcut a couple of times and that tip
   retires for good.

## The character

Tips come from a small green keycap with opinions. Each tip is shown at most
three times, and the tone escalates:

| Showing | Face | Example |
|---|---|---|
| 1st | Wink | "Psst. Two clicks to reach Downloads? Ctrl+J just walks in the front door." |
| 2nd | Cheeky | "The menu again? Ctrl+J is right there. I'm not mad. I'm just... watching." |
| 3rd | Pleading | "Last time I'll say it: Ctrl+J opens Downloads. After this, I suffer in silence." |

When you actually use a shortcut it taught you, it celebrates. The tray icon
wears its current mood.

## What it covers

44 tips so far:

| Area | Examples |
|---|---|
| Everywhere | Copy, Paste, Cut, Undo, Redo, Select all, Save, Find, Print (when reached through a menu) |
| Chrome, Edge, Brave | Downloads, History, reopen closed tab, private window, developer tools |
| File Explorer | Rename, new folder, Properties, permanent delete, copy as path |
| Windows | Task Manager, Settings, screenshots |
| Word, Excel, PowerPoint, OneNote | Bold, Italic, Underline, Link, Replace, AutoSum, Filter, New slide |
| Notepad, VS Code | Time stamp, Go to line, Command Palette, terminal, format document |

A tip is only added when the shortcut is a real improvement. A one-click
toolbar button is already as quick as its shortcut, so those tips appear only
if you were typing a moment before and the mouse was the detour.

**Current limits**

- Windows 10 and 11 only.
- Tips match the English names of buttons and menus, so most of them do not
  appear when Windows is set to another language. Keyflex says so on its Home
  screen when that is the case.

## Privacy

Everything happens on your PC. Keyflex has no account, no analytics and makes
no network connections.

- **Only button and menu names** are read, and only for the control you
  clicked. Web pages, documents, file names and browser tabs are never read.
- **Only the shortcuts it teaches** are noticed. No other key press is recorded.
- **Only a small settings file** is kept, at `%APPDATA%\Keyflex\state.json`.

The full statement is in [PRIVACY.md](PRIVACY.md).

## Building from source

Prerequisites: Rust (stable, MSVC toolchain), Node 20 or later, and Visual
Studio Build Tools with the "Desktop development with C++" workload.

```bash
git clone https://github.com/HarshalPatel1972/keyflex.git
cd keyflex
npm install
npm run tauri dev                    # run with live reload
npx tauri build --no-bundle          # release build: src-tauri/target/release/keyflex.exe
```

Run the core's tests with:

```bash
cd core
cargo test
```

### Packaging for the Microsoft Store

```powershell
pwsh packaging/build-msix.ps1 -IdentityName "<from Partner Center>" -Publisher "CN=<from Partner Center>" -PublisherDisplayName "<your publisher name>"
```

See [packaging/README.md](packaging/README.md) for where to find those values.

## Project layout

| Path | What it is |
|---|---|
| `core/` | The engine, in Rust: rules, tip history, click inspection, input hooks and the popup. No UI framework. |
| `core/tips.toml` | Every tip: the apps, the shortcut, the trigger and the three lines. |
| `src-tauri/` | The app shell: tray icon, window, settings commands. |
| `src/` | The window's UI, in React. |
| `packaging/` | Microsoft Store (MSIX) packaging. |
| `design/` | Logo and character sources and previews. |

## Adding a tip

Tips live in [`core/tips.toml`](core/tips.toml), and the top of that file
documents the format. A tip names its apps, its shortcut, what the user has to
do for it to fire, and three hand-written lines. `cargo test` checks the file
is valid.

Useful switches while writing tips:

| Setting | Effect |
|---|---|
| `KEYFLEX_DEMO=1` | No limits: every match shows a tip. |
| `KEYFLEX_LOG=<file>` | Appends the name of each button or menu item you click to a file, so you can see what an app really calls things. |

## Tech

| Layer | Choice |
|---|---|
| Engine and popup | Rust with `windows-rs` (UI Automation, low-level hooks, GDI) |
| Shell | Tauri v2 |
| Window UI | React 18, TypeScript, Vite |
| Storage | One JSON file |

## Contributing

Issues and pull requests are welcome, especially new tips and corrections to
existing ones. Please use conventional commits: `type(scope): description`.

## License

[MIT](LICENSE)
