import React from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Sidebar } from "./Sidebar";

const appWindow = getCurrentWindow();

interface LayoutProps {
  children: React.ReactNode;
  activeTab: string;
  setActiveTab: (tab: string) => void;
  streak: number;
}

export const Layout: React.FC<LayoutProps> = ({
  children,
  activeTab,
  setActiveTab,
  streak,
}) => {
  return (
    <div className="layout">
      <div className="titlebar" data-tauri-drag-region>
        <div className="titlebar-left">
          <svg className="logo" viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
            <rect x="10" y="22" width="80" height="56" rx="14" fill="#F5A623"/>
            <rect x="16" y="28" width="68" height="44" rx="10" fill="#D4891A"/>
            <path d="M25 58 Q34 30 50 46 Q66 62 75 36" stroke="#0D0D0D" stroke-width="7" stroke-linecap="round" fill="none"/>
            <circle cx="75" cy="36" r="5.5" fill="#0D0D0D"/>
          </svg>
          <span className="app-name">Keyflex</span>
        </div>
        <div className="titlebar-right">
          <button className="window-control" onClick={() => appWindow.minimize()}>
            <svg width="12" height="12" viewBox="0 0 12 12"><rect fill="currentColor" x="1" y="5" width="10" height="1"/></svg>
          </button>
          <button className="window-control" onClick={() => appWindow.toggleMaximize()}>
            <svg width="12" height="12" viewBox="0 0 12 12"><rect fill="none" stroke="currentColor" x="1.5" y="1.5" width="9" height="9"/></svg>
          </button>
          <button className="window-control close" onClick={() => appWindow.hide()}>
            <svg width="12" height="12" viewBox="0 0 12 12"><path fill="currentColor" d="M10.5 1.5l-9 9m0-9l9 9"/></svg>
          </button>
        </div>
      </div>

      <div className="main-shell">
        <Sidebar activeTab={activeTab} setActiveTab={setActiveTab} streak={streak} />
        <main className="content-area">{children}</main>
      </div>
    </div>
  );
};
