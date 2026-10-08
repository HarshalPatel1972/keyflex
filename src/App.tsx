import { getCurrentWindow } from "@tauri-apps/api/window";
import { useCallback, useEffect, useRef, useState } from "react";
import { api, onChanged, Settings, Status, Tip } from "./api";
import { Confetti } from "./Confetti";
import { HomeScreen } from "./HomeScreen";
import { Intro } from "./Intro";
import { Mascot } from "./Mascot";
import { SettingsScreen } from "./SettingsScreen";
import { applyTheme } from "./theme";
import { TipsScreen } from "./TipsScreen";

type Screen = "home" | "tips" | "intro" | "settings";

/** The screen named in the address (`#tips`), if any: lets the UI be opened on a given screen. */
function requestedScreen(): Screen | null {
  const name = window.location.hash.slice(1);
  return name === "home" || name === "tips" || name === "intro" || name === "settings" ? name : null;
}

/** `icon` is the path data of a 20 x 20 outline icon. */
const TABS: { id: Screen; label: string; icon: string }[] = [
  { id: "home", label: "Home", icon: "M3.5 9.5 10 4l6.5 5.5V16h-4.5v-4h-4v4H3.5z" },
  { id: "tips", label: "Shortcuts", icon: "M3 6.5h14v8H3zM6 9.5h1m2.5 0h1m2.5 0h1M6.5 12h7" },
  { id: "intro", label: "How it works", icon: "M7 4.5v11l9-5.5z" },
  { id: "settings", label: "Settings", icon: "M3 6h8m4 0h2M3 14h2m4 0h8M13 4v4M7 12v4" },
];

function App() {
  const [screen, setScreen] = useState<Screen | null>(null);
  const [settings, setSettings] = useState<Settings | null>(null);
  const [tips, setTips] = useState<Tip[]>([]);
  const [status, setStatus] = useState<Status>({ mood: "knowing", about: null });
  // Counts up each time a shortcut becomes "yours" while the window is open.
  const [parties, setParties] = useState(0);
  const learnedBefore = useRef<number | null>(null);

  const refresh = useCallback(async () => {
    const [nextSettings, nextTips, nextStatus] = await Promise.all([
      api.getSettings(),
      api.getTips(),
      api.getStatus(),
    ]);
    setSettings(nextSettings);
    setTips(nextTips);
    setStatus(nextStatus);

    const learned = nextTips.filter((tip) => tip.learned).length;
    if (learnedBefore.current !== null && learned > learnedBefore.current) {
      setParties((count) => count + 1);
    }
    learnedBefore.current = learned;
    return nextSettings;
  }, []);

  useEffect(() => {
    refresh().then((loaded) => setScreen(loaded.intro_seen ? (requestedScreen() ?? "home") : "intro"));
    const unlisten = onChanged(() => void refresh());
    return () => void unlisten.then((stop) => stop());
  }, [refresh]);

  useEffect(() => {
    if (settings) applyTheme(settings.theme);
  }, [settings?.theme]);

  const saveSettings = (next: Settings) => {
    setSettings(next);
    void api.setSettings(next);
  };

  if (!screen || !settings) return null;

  const firstRun = !settings.intro_seen;
  const finishIntro = () => {
    if (firstRun) saveSettings({ ...settings, intro_seen: true });
    setScreen("home");
  };

  return (
    <div className="app">
      {/* The window has no native title bar: this bar is the title bar. Empty space drags the window. */}
      <header className="titlebar" data-tauri-drag-region>
        <span className="brand" data-tauri-drag-region>
          <img src="/keyflex-icon.svg" alt="" width="24" height="24" />
          Keyflex
        </span>
        <span className="titlebar-space" data-tauri-drag-region />
        <WindowControls />
      </header>
      <div className="body">
        {!firstRun && (
          <nav className="sidebar" aria-label="Sections">
            {TABS.map((tab) => (
              <button
                key={tab.id}
                className={screen === tab.id ? "tab active" : "tab"}
                aria-current={screen === tab.id ? "page" : undefined}
                onClick={() => setScreen(tab.id)}
              >
                <svg width="20" height="20" viewBox="0 0 20 20" aria-hidden="true">
                  <path
                    d={tab.icon}
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="1.8"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  />
                </svg>
                {tab.label}
              </button>
            ))}
            {screen !== "home" && (
              <button className="mood" onClick={() => setScreen("home")} title="Go home">
                <Mascot mood={status.mood} size={46} follow={false} />
                <span>{moodLine(status)}</span>
              </button>
            )}
          </nav>
        )}
        {/* Keyed by screen so each one slides in fresh. */}
        <main key={screen} className="screen">
          {screen === "home" && <HomeScreen tips={tips} status={status} onSeeAll={() => setScreen("tips")} />}
          {screen === "tips" && <TipsScreen tips={tips} />}
          {screen === "intro" && <Intro firstRun={firstRun} onDone={finishIntro} />}
          {screen === "settings" && <SettingsScreen settings={settings} onChange={saveSettings} />}
        </main>
      </div>
      {parties > 0 && <Confetti key={parties} />}
    </div>
  );
}

/** What the character in the sidebar is thinking. */
function moodLine(status: Status): string {
  switch (status.mood) {
    case "sleepy":
      return "Napping. Wake me when you want tips.";
    case "proud":
      return "Still proud of you.";
    case "pleading":
      return `Still hoping you'll try ${status.about ?? "that shortcut"}...`;
    case "sad":
      return `I've let ${status.about ?? "that one"} go. It's fine.`;
    case "zipped":
      return "Lips sealed.";
    case "sideeye":
      return "I saw that. Saying nothing.";
    case "amazed":
      return "You knew that one already?";
    case "tired":
      return "All talked out for today.";
    case "hello":
      return "Hi. I'm new here.";
    case "zen":
      return "Nothing to mention. Bliss.";
    case "confused":
      return "I can't read these menus.";
    default:
      return "Keeping an eye out.";
  }
}

function WindowControls() {
  const appWindow = getCurrentWindow();
  return (
    <div className="window-controls">
      <button aria-label="Minimize" onClick={() => void appWindow.minimize()}>
        <svg width="10" height="10" viewBox="0 0 10 10">
          <path d="M0 5h10" stroke="currentColor" />
        </svg>
      </button>
      <button aria-label="Maximize" onClick={() => void appWindow.toggleMaximize()}>
        <svg width="10" height="10" viewBox="0 0 10 10">
          <rect x="0.5" y="0.5" width="9" height="9" fill="none" stroke="currentColor" />
        </svg>
      </button>
      <button aria-label="Close" className="close" onClick={() => void appWindow.close()}>
        <svg width="10" height="10" viewBox="0 0 10 10">
          <path d="M0 0l10 10M10 0L0 10" stroke="currentColor" />
        </svg>
      </button>
    </div>
  );
}

export default App;
