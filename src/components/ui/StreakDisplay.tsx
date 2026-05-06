import React from "react";

interface StreakDisplayProps {
  streak: number;
}

export const StreakDisplay: React.FC<StreakDisplayProps> = ({ streak }) => {
  const days = ["M", "T", "W", "T", "F", "S", "S"];
  // For now, we'll just show the last 7 slots, ideally this would be real data
  const status = [true, true, true, true, true, false, false]; 

  const emojiSize = Math.max(1, Math.min(3, 1 + streak / 10)) + "em";

  return (
    <div className="streak-display">
      <div className="streak-emoji-container">
        <span className="streak-emoji" style={{ fontSize: emojiSize }}>🔥</span>
      </div>
      <div className="streak-value-container">
        <span className="streak-number">{streak}</span>
        <span className="streak-label">days in a row</span>
      </div>
      <div className="streak-dots">
        {days.map((day, i) => (
          <div key={i} className="streak-dot-item">
            <div className={`streak-dot ${status[i] ? "active" : ""}`} />
            <span className="dot-label">{day}</span>
          </div>
        ))}
      </div>
    </div>
  );
};
