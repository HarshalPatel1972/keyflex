import React from "react";

export const HeatLegend: React.FC = () => {
  const levels = [
    { label: "Never", class: "h0" },
    { label: "Rare", class: "h1" },
    { label: "Occasional", class: "h2" },
    { label: "Common", class: "h3" },
    { label: "Frequent", class: "h4" },
    { label: "Blazing", class: "h5" },
  ];

  return (
    <div className="heat-legend">
      {levels.map((level, i) => (
        <div key={i} className="legend-item">
          <div className={`legend-box ${level.class}`} />
          <span className="legend-label">{level.label}</span>
        </div>
      ))}
    </div>
  );
};
