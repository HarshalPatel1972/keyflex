import React, { useState, useEffect } from "react";
import { Layout } from "./components/Layout";
import { Overview } from "./components/tabs/Overview";
import { Heatmap } from "./components/tabs/Heatmap";
import { PerApp } from "./components/tabs/PerApp";
import { GapAnalysis } from "./components/tabs/GapAnalysis";
import { History } from "./components/tabs/History";
import { api, onShortcutRecorded, onStreakUpdated, onMilestoneAchieved, MilestoneRecord } from "./lib/ipc";

function App() {
  const [activeTab, setActiveTab] = useState("overview");
  const [streak, setStreak] = useState(0);
  const [milestone, setMilestone] = useState<MilestoneRecord | null>(null);

  useEffect(() => {
    api.getStreak().then((data) => setStreak(data.current)).catch(console.error);

    const unlistenShortcut = onShortcutRecorded((data) => {
      console.log("Shortcut recorded:", data);
    });

    const unlistenStreak = onStreakUpdated((data) => {
      setStreak(data.current);
    });

    const unlistenMilestone = onMilestoneAchieved((data) => {
      setMilestone(data);
      setTimeout(() => setMilestone(null), 5000);
    });

    return () => {
      unlistenShortcut.then(fn => fn());
      unlistenStreak.then(fn => fn());
      unlistenMilestone.then(fn => fn());
    };
  }, []);

  const renderTab = () => {
    switch (activeTab) {
      case "overview": return <Overview />;
      case "heatmap": return <Heatmap />;
      case "per-app": return <PerApp />;
      case "gap-analysis": return <GapAnalysis />;
      case "history": return <History />;
      default: return <Overview />;
    }
  };

  return (
    <Layout activeTab={activeTab} setActiveTab={setActiveTab} streak={streak}>
      {renderTab()}
      
      {milestone && (
        <div className="milestone-toast">
          <div className="milestone-toast-icon">🏆</div>
          <div className="milestone-toast-content">
            <div className="milestone-toast-title">Milestone Achieved!</div>
            <div className="milestone-toast-label">{milestone.label}</div>
            <div className="milestone-toast-desc">{milestone.description}</div>
          </div>
          <button className="milestone-toast-close" onClick={() => setMilestone(null)}>×</button>
        </div>
      )}
    </Layout>
  );
}

export default App;
