import { useState, useEffect } from "react";
import { api, onShortcutRecorded, TodayStats, EfficiencyScore, ShortcutEntry, GapItem } from "../../lib/ipc";
import { ScoreRing } from "../ui/ScoreRing";
import { StreakDisplay } from "../ui/StreakDisplay";
import { StatCard } from "../ui/StatCard";
import { ShortcutRow } from "../ui/ShortcutRow";

export const Overview: React.FC = () => {
  const [stats, setStats] = useState<TodayStats>({ total: 0, unique: 0, new_unlocks: 0, mouse_escapes: 0 });
  const [score, setScore] = useState<EfficiencyScore>({ score: 0, shortcuts_today: 0, unique_today: 0, mouse_escapes: 0 });
  const [topShortcuts, setTopShortcuts] = useState<ShortcutEntry[]>([]);
  const [gapOfDay, setGapOfDay] = useState<GapItem | null>(null);

  const fetchData = async () => {
    try {
      const [s, esc, top, gap] = await Promise.all([
        api.getTodayStats("All Apps"),
        api.getEfficiencyScore(),
        api.getTopShortcuts("All Apps", 1, 5),
        api.getShortcutOfDay()
      ]);
      setStats(s);
      setScore(esc);
      setTopShortcuts(top);
      setGapOfDay(gap);
    } catch (err) {
      console.error("Overview fetch error:", err);
    }
  };

  useEffect(() => {
    fetchData();
    const unlisten = onShortcutRecorded(() => fetchData());
    return () => { unlisten.then(fn => fn()); };
  }, []);

  const maxCount = topShortcuts.length > 0 ? topShortcuts[0].count : 1;

  return (
    <div className="overview-tab">
      <div className="overview-grid">
        {/* Left Column: Score & Streak */}
        <div className="overview-col left">
          <section className="section-card">
            <h3 className="label-mono">Efficiency Score</h3>
            <ScoreRing score={score.score} />
            <div className="sub-stats">
              <div className="sub-stat-row">
                <span>Shortcuts today</span>
                <span>{score.shortcuts_today}</span>
              </div>
              <div className="sub-stat-row">
                <span>Unique combos</span>
                <span>{score.unique_today}</span>
              </div>
            </div>
          </section>

          <StreakDisplay streak={score.shortcuts_today > 0 ? 1 : 0} /> {/* Simplified for now */}
        </div>

        {/* Right Column: Stats & Top List */}
        <div className="overview-col right">
          <div className="stats-row">
            <StatCard label="Total Shortcuts" value={stats.total} />
            <StatCard label="Unique Combos" value={stats.unique} />
            <StatCard label="New Unlocks" value={stats.new_unlocks} />
          </div>

          <section className="section-card top-list-section">
            <h3 className="label-mono">Top Shortcuts Today</h3>
            <div className="top-list">
              {topShortcuts.length > 0 ? (
                topShortcuts.map((s, i) => (
                  <div key={i} className="top-shortcut-item">
                    <ShortcutRow 
                      shortcut={s.shortcut} 
                      count={s.count} 
                      showBar 
                      maxCount={maxCount} 
                    />
                    <span className="count-badge">{s.count}</span>
                  </div>
                ))
              ) : (
                <div className="empty-state">Press some shortcuts to see them here!</div>
              )}
            </div>
          </section>

          {gapOfDay && (
            <section className="section-card gap-day-card">
              <div className="gap-header">
                <h3 className="label-mono">Shortcut of the Day</h3>
                <span className="power-pct">{gapOfDay.power_user_pct}% of power users use this</span>
              </div>
              <div className="gap-content">
                <ShortcutRow shortcut={gapOfDay.shortcut} />
                <p className="gap-desc">{gapOfDay.description}</p>
                <div className="gap-footer">
                  <span className="gap-app-tag">{gapOfDay.app}</span>
                  <button className="cta-button">Try it!</button>
                </div>
              </div>
            </section>
          )}
        </div>
      </div>

    </div>
  );
};
