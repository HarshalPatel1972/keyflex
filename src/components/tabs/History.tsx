import { useState, useEffect } from "react";
import { api, DayActivity, onShortcutRecorded } from "../../lib/ipc";

export const History: React.FC = () => {
  const [activity, setActivity] = useState<DayActivity[]>([]);
  const [hoveredDay, setHoveredDay] = useState<DayActivity | null>(null);

  const fetchData = async () => {
    try {
      const data = await api.getDayActivity(90); // 90 days
      setActivity(data);
    } catch (err) {
      console.error("History fetch error:", err);
    }
  };

  useEffect(() => {
    fetchData();
    const unlisten = onShortcutRecorded(() => fetchData());
    return () => { unlisten.then(fn => fn()); };
  }, []);

  // Map activity to a 7x13 grid (approx 90 days)
  const grid: (DayActivity | null)[][] = Array.from({ length: 7 }, () => Array(13).fill(null));
  
  activity.forEach((day, i) => {
    const col = Math.floor(i / 7);
    const row = i % 7;
    if (col < 13) grid[row][col] = day;
  });

  const daysLabels = ["Mon", "Wed", "Fri", "Sun"];

  return (
    <div className="history-tab">
      <div className="history-header">
        <h2 className="tab-title">Shortcut Consistency</h2>
        <p className="tab-subtitle">Your activity over the last 90 days</p>
      </div>

      <section className="section-card grid-card">
        <div className="grid-wrapper">
          <div className="day-labels">
            {daysLabels.map(l => <span key={l}>{l}</span>)}
          </div>
          <div className="contribution-grid">
            {grid.map((row, i) => (
              <div key={i} className="grid-row">
                {row.map((day, j) => (
                  <div 
                    key={j} 
                    className={`grid-cell level-${day?.level || 0}`}
                    onMouseEnter={() => day && setHoveredDay(day)}
                    onMouseLeave={() => setHoveredDay(null)}
                  />
                ))}
              </div>
            ))}
          </div>
        </div>

        {hoveredDay && (
          <div className="grid-popover">
            <span className="popover-count">{hoveredDay.count} shortcuts</span>
            <span className="popover-date">{hoveredDay.date}</span>
          </div>
        )}

        <div className="grid-legend">
          <span>Less</span>
          <div className="grid-cell level-0" />
          <div className="grid-cell level-1" />
          <div className="grid-cell level-2" />
          <div className="grid-cell level-3" />
          <div className="grid-cell level-4" />
          <span>More</span>
        </div>
      </section>

      <div className="milestones-history">
        <h3 className="label-mono">Recent Milestones</h3>
        <div className="milestone-timeline">
          <div className="timeline-item">
            <div className="timeline-dot" />
            <div className="timeline-content">
              <span className="timeline-date">Yesterday</span>
              <p>🏆 <strong>First Blood</strong>: Recorded your first 10 shortcuts.</p>
            </div>
          </div>
          <div className="timeline-item">
            <div className="timeline-dot" />
            <div className="timeline-content">
              <span className="timeline-date">2 days ago</span>
              <p>🏆 <strong>Hot Streak</strong>: Maintained a 3-day streak.</p>
            </div>
          </div>
        </div>
      </div>

    </div>
  );
};
