# Keyflex privacy policy

_Last updated: 8 October 2026_

Keyflex watches how you use your PC so it can suggest keyboard shortcuts. This
page says exactly what it looks at, what it keeps, and what it never does.

## In short

- Everything Keyflex does happens on your PC.
- Keyflex has no account, no analytics and no advertising, and it makes no
  network connections of its own.
- It never records what you type, and never reads the content of web pages,
  documents or files.

## What Keyflex looks at

To notice that you did something "the long way", Keyflex observes three things
while it is running:

1. **The button or menu item you click.** When you click, Keyflex asks Windows
   what kind of control is under the pointer. If it is a button or a menu item
   that belongs to the app's own interface, Keyflex reads its name (for example
   "Downloads" or "Copy") and compares it with its list of tips. The name is
   then discarded.

   Keyflex does **not** read the names of anything else: text, links, list
   items, file names, browser tabs, or anything inside a web page or document.
   For those, the name is never requested from Windows at all.

2. **The shortcuts it teaches.** Keyflex notices when you press one of the
   specific key combinations in its tip list (for example Ctrl+J), so it can
   stop suggesting a shortcut you already use. No other key press leaves the
   part of Keyflex that receives keyboard input. For all other keys it updates
   only a timestamp ("a key was pressed just now"), with no record of which key.

3. **Which app's window opened, and how.** Keyflex notices when a new window
   appears so it can tell whether you opened an app with the mouse or the
   keyboard. It looks up the name of the program (for example `taskmgr.exe`)
   and the type of window. It does not read window contents. When you click a
   button or press a taught shortcut, it also reads the title of the active
   window, only to check whether it contains the words "Recycle Bin" (one tip
   depends on that). Titles are compared and discarded, never stored.

## What Keyflex stores

Keyflex keeps one small file on your PC:

`%APPDATA%\Keyflex\state.json`

It contains:

- your settings (theme, tips per day, which apps are switched off, whether
  Keyflex is paused);
- for each tip, how many times it has been shown, when it was last shown,
  whether you have used its shortcut, and whether you muted it;
- the times of recent tips, to limit how often they appear.

It does not contain anything you typed, clicked, opened or viewed.

Deleting that file resets Keyflex. Uninstalling Keyflex and deleting the
`Keyflex` folder removes everything.

## What Keyflex never does

- It never sends any information anywhere. There is no server.
- It never records keystrokes, text, passwords, clipboard contents or screen
  images.
- It never reads web pages, documents, emails, messages or file names.
- It never shares or sells data, because it has none to share.

## Your controls

- **Pause** stops all watching until you switch it back on (Settings, or the
  tray icon's menu).
- **Per-app switches** in Settings turn tips off for a single app, or for
  "Everywhere" (the tips that apply in all apps).
- **Don't show again** on any tip mutes that tip.
- **Quit** from the tray icon stops Keyflex completely.

## An optional diagnostic log

Keyflex has a diagnostic mode for checking tips against real apps. It is off
unless you start Keyflex yourself with the `KEYFLEX_LOG` setting pointing at a
file. While on, it writes the names of the buttons and menu items you click to
that file on your PC. Nothing is sent anywhere. Do not switch it on unless you
intend to.

## Children

Keyflex does not collect personal information from anyone, including children.

## Changes

If this policy changes, the new version will be published at this address with
a new date at the top.

## Contact

Questions about this policy: open an issue at
<https://github.com/HarshalPatel1972/keyflex/issues>.
