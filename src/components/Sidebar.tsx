import React from "react";

interface SidebarProps {
  activeTab: string;
  setActiveTab: (tab: string) => void;
  streak: number;
}

export const Sidebar: React.FC<SidebarProps> = ({ activeTab, setActiveTab, streak }) => {
  const navItems = [
    { id: "overview", label: "Overview", icon: "📊" },
    { id: "heatmap", label: "Heatmap", icon: "🔥" },
    { id: "per-app", label: "Per App", icon: "📱" },
    { id: "gap-analysis", label: "Gap Analysis", icon: "🔓" },
    { id: "history", label: "History", icon: "📅" },
  ];

  return (
    <div className="sidebar">
      <div className="nav-list">
        {navItems.map((item) => (
          <button
            key={item.id}
            className={`nav-item ${activeTab === item.id ? "active" : ""}`}
            onClick={() => setActiveTab(item.id)}
          >
            <span className="nav-icon">{item.icon}</span>
            <span className="nav-label">{item.label}</span>
          </button>
        ))}
      </div>

      <div className="sidebar-footer">
        <div className="mini-streak">
          <span className="streak-emoji">🔥</span>
          <span className="streak-count">{streak} day streak</span>
        </div>
      </div>
    </div>
  );
};
