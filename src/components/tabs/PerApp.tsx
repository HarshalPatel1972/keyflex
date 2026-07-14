import { useState, useEffect } from "react";
import { api, ShortcutEntry, onShortcutRecorded } from "../../lib/ipc";
import { ShortcutRow } from "../ui/ShortcutRow";
import {
  BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer,
  AreaChart, Area
} from 'recharts';

export const PerApp: React.FC = () => {
  const [apps, setApps] = useState<string[]>([]);
  const [selectedApp, setSelectedApp] = useState("VS Code");
  const [topShortcuts, setTopShortcuts] = useState<ShortcutEntry[]>([]);
  const [activity, setActivity] = useState<{ date: string; count: number }[]>([]);

  const fetchData = async () => {
    try {
      const [appList, top, hist] = await Promise.all([
        api.getAppsList(),
        api.getPerAppBreakdown(selectedApp, 30),
        api.getDayActivity(7) // Simplified for app-specific view later
      ]);
      setApps(appList.filter(a => a !== "All Apps"));
      setTopShortcuts(top.slice(0, 10));
      setActivity(hist.map(d => ({ date: d.date.split('-').slice(1).join('/'), count: d.count })));
    } catch (err) {
      console.error("PerApp fetch error:", err);
    }
  };

  useEffect(() => {
    fetchData();
    const unlisten = onShortcutRecorded(() => fetchData());
    return () => { unlisten.then(fn => fn()); };
  }, [selectedApp]);

  return (
    <div className="per-app-tab">
      <div className="app-selector">
        {apps.map(app => (
          <button 
            key={app} 
            className={`app-pill ${selectedApp === app ? "active" : ""}`}
            onClick={() => setSelectedApp(app)}
          >
            {app}
          </button>
        ))}
      </div>

      <div className="charts-grid">
        <section className="section-card chart-card">
          <h3 className="label-mono">Top 10 Shortcuts</h3>
          <div className="chart-container">
            <ResponsiveContainer width="100%" height={300}>
              <BarChart layout="vertical" data={topShortcuts} margin={{ left: 40, right: 20 }}>
                <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" horizontal={false} />
                <XAxis type="number" hide />
                <YAxis 
                  dataKey="shortcut" 
                  type="category" 
                  tick={{ fill: 'var(--smoke)', fontSize: 10, fontFamily: 'var(--font-mono)' }} 
                  width={100}
                />
                <Tooltip 
                  cursor={{ fill: 'var(--surface2)' }}
                  contentStyle={{ background: 'var(--surface2)', border: '1px solid var(--border)', borderRadius: '8px' }}
                />
                <Bar dataKey="count" fill="var(--ember)" radius={[0, 4, 4, 0]} />
              </BarChart>
            </ResponsiveContainer>
          </div>
        </section>

        <section className="section-card chart-card">
          <h3 className="label-mono">7-Day Activity</h3>
          <div className="chart-container">
            <ResponsiveContainer width="100%" height={300}>
              <AreaChart data={activity}>
                <defs>
                  <linearGradient id="colorCount" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="var(--ember)" stopOpacity={0.3}/>
                    <stop offset="95%" stopColor="var(--ember)" stopOpacity={0}/>
                  </linearGradient>
                </defs>
                <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" vertical={false} />
                <XAxis 
                  dataKey="date" 
                  tick={{ fill: 'var(--smoke)', fontSize: 10 }} 
                />
                <YAxis 
                  tick={{ fill: 'var(--smoke)', fontSize: 10 }} 
                />
                <Tooltip 
                  contentStyle={{ background: 'var(--surface2)', border: '1px solid var(--border)', borderRadius: '8px' }}
                />
                <Area type="monotone" dataKey="count" stroke="var(--ember)" fillOpacity={1} fill="url(#colorCount)" strokeWidth={2} />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        </section>
      </div>

      <section className="section-card full-list-section">
        <h3 className="label-mono">Full Shortcut History</h3>
        <div className="full-list">
          {topShortcuts.map((s, i) => (
            <div key={i} className="list-item">
              <ShortcutRow shortcut={s.shortcut} />
              <span className="count-badge">{s.count} times</span>
            </div>
          ))}
        </div>
      </section>

    </div>
  );
};
