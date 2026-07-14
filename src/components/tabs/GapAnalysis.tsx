import { useState, useEffect } from "react";
import { api, GapItem } from "../../lib/ipc";
import { ShortcutRow } from "../ui/ShortcutRow";

export const GapAnalysis: React.FC = () => {
  const [gaps, setGaps] = useState<GapItem[]>([]);
  const [appFilter, setAppFilter] = useState("All Apps");
  const [apps] = useState(["All Apps", "VS Code", "Chrome", "Windows", "General"]);

  const fetchData = async () => {
    try {
      const data = await api.getGapItems(appFilter);
      setGaps(data);
    } catch (err) {
      console.error("GapAnalysis fetch error:", err);
    }
  };

  useEffect(() => {
    fetchData();
  }, [appFilter]);

  return (
    <div className="gap-tab">
      <div className="gap-header-section">
        <h2 className="tab-title">Level Up Your Shortcuts</h2>
        <div className="tab-pills">
          {apps.map(app => (
            <button 
              key={app} 
              className={`tab-pill ${appFilter === app ? "active" : ""}`}
              onClick={() => setAppFilter(app)}
            >
              {app}
            </button>
          ))}
        </div>
      </div>

      <div className="gap-list">
        {gaps.map((gap, i) => (
          <div key={i} className={`gap-card ${gap.unlocked ? "unlocked" : ""}`}>
            <div className="gap-card-top">
              <ShortcutRow shortcut={gap.shortcut} />
              <div className="gap-card-meta">
                <span className="gap-app-tag">{gap.app}</span>
                {gap.unlocked ? (
                  <span className="unlocked-badge">✓ Unlocked</span>
                ) : gap.priority === 3 ? (
                  <span className="locked-badge">🔒 Locked</span>
                ) : (
                  <button className="unlock-btn">UNLOCK →</button>
                )}
              </div>
            </div>
            <div className="gap-card-body">
              <h4 className="gap-card-title">{gap.description}</h4>
              <div className="power-bar-container">
                <div className="power-bar-fill" style={{ width: `${gap.power_user_pct}%` }} />
                <span className="power-label">{gap.power_user_pct}% of power users</span>
              </div>
              <p className="gap-tip">"Your most powerful shortcut. Opens everything."</p>
            </div>
          </div>
        ))}
      </div>

    </div>
  );
};
