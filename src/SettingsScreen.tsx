import { useEffect, useState } from "react";
import { api, appName, Settings, Theme } from "./api";

const TIPS_PER_DAY = [1, 3, 5, 10];
const THEMES: { id: Theme; label: string }[] = [
  { id: "system", label: "Match Windows" },
  { id: "light", label: "Light" },
  { id: "dark", label: "Dark" },
];

interface Props {
  settings: Settings;
  onChange: (settings: Settings) => void;
}

export function SettingsScreen({ settings, onChange }: Props) {
  const [apps, setApps] = useState<string[]>([]);
  const [autostart, setAutostart] = useState(false);
  const [autostartError, setAutostartError] = useState("");

  useEffect(() => {
    void api.getApps().then(setApps);
    void api.getAutostart().then(setAutostart);
  }, []);

  const toggleAutostart = async (enabled: boolean) => {
    try {
      await api.setAutostart(enabled);
      setAutostart(enabled);
      setAutostartError("");
    } catch (error) {
      setAutostartError(`Couldn't change this: ${error}`);
    }
  };

  const toggleApp = (app: string, enabled: boolean) => {
    const disabled = settings.disabled_apps.filter((item) => item !== app);
    onChange({ ...settings, disabled_apps: enabled ? disabled : [...disabled, app] });
  };

  return (
    <div className="page">
      <header className="page-header">
        <h1>Settings</h1>
      </header>

      <section>
        <h2>Tips</h2>
        <Row title="Pause Keyflex" detail="Stops watching completely until you switch it back on.">
          <Switch
            label="Pause Keyflex"
            checked={settings.paused}
            onChange={(paused) => onChange({ ...settings, paused })}
          />
        </Row>
        <Row title="Tips per day" detail="The most you'll see in any 24 hours. They also stay at least 10 minutes apart.">
          <div className="segmented" role="radiogroup" aria-label="Tips per day">
            {TIPS_PER_DAY.map((count) => (
              <button
                key={count}
                role="radio"
                aria-checked={settings.tips_per_day === count}
                className={settings.tips_per_day === count ? "active" : ""}
                onClick={() => onChange({ ...settings, tips_per_day: count })}
              >
                {count}
              </button>
            ))}
          </div>
        </Row>
        <Row title="See what a tip looks like" detail="Pops one up beside your cursor.">
          <button className="ghost" onClick={() => void api.previewTip()}>
            Show a sample
          </button>
        </Row>
      </section>

      <section>
        <h2>Appearance</h2>
        <Row title="Theme" detail="Applies to this window and to the tip popup.">
          <div className="segmented" role="radiogroup" aria-label="Theme">
            {THEMES.map((theme) => (
              <button
                key={theme.id}
                role="radio"
                aria-checked={settings.theme === theme.id}
                className={settings.theme === theme.id ? "active" : ""}
                onClick={() => onChange({ ...settings, theme: theme.id })}
              >
                {theme.label}
              </button>
            ))}
          </div>
        </Row>
      </section>

      <section>
        <h2>Apps</h2>
        {apps.map((app) => (
          <Row
            key={app}
            title={appName(app)}
            detail={app === "*" ? "Copy, paste, undo, save and friends, in any app." : undefined}
          >
            <Switch
              label={`Tips in ${appName(app)}`}
              checked={!settings.disabled_apps.includes(app)}
              onChange={(enabled) => toggleApp(app, enabled)}
            />
          </Row>
        ))}
      </section>

      <section>
        <h2>Startup</h2>
        <Row title="Start when I sign in to Windows" detail={autostartError || "Keyflex starts quietly in the tray."}>
          <Switch label="Start when I sign in to Windows" checked={autostart} onChange={toggleAutostart} />
        </Row>
      </section>

      <p className="footnote">
        Keyflex only looks at button and menu names in the apps switched on above, and only at the shortcuts it
        teaches. Nothing leaves this PC.
      </p>
    </div>
  );
}

function Row({ title, detail, children }: { title: string; detail?: string; children: React.ReactNode }) {
  return (
    <div className="row">
      <div>
        <div className="row-title">{title}</div>
        {detail && <div className="row-detail">{detail}</div>}
      </div>
      {children}
    </div>
  );
}

function Switch({ label, checked, onChange }: { label: string; checked: boolean; onChange: (on: boolean) => void }) {
  return (
    <label className="switch">
      <input type="checkbox" checked={checked} aria-label={label} onChange={(event) => onChange(event.target.checked)} />
      <span />
    </label>
  );
}
