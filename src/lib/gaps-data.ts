export interface KnownShortcut {
  shortcut: string;
  app: string;
  description: string;
  powerUserPct: number;
  priority: 1 | 2 | 3;
  category: string;
  tip?: string;
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
