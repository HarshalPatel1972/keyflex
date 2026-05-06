import React from "react";

interface StatCardProps {
  label: string;
  value: string | number;
  subValue?: string;
  icon?: React.ReactNode;
}

export const StatCard: React.FC<StatCardProps> = ({ label, value, subValue, icon }) => {
  return (
    <div className="stat-card">
      <div className="stat-header">
        <span className="label-mono">{label}</span>
        {icon && <span className="stat-icon">{icon}</span>}
      </div>
      <div className="stat-value">{value}</div>
      {subValue && <div className="stat-subvalue">{subValue}</div>}
    </div>
  );
};
