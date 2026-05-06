import React from "react";

interface KeyChipProps {
  children: React.ReactNode;
  active?: boolean;
  hot?: boolean;
}

export const KeyChip: React.FC<KeyChipProps> = ({ children, active, hot }) => {
  return (
    <span className={`key-chip ${active ? "active" : ""} ${hot ? "hot" : ""}`}>
      {children}
    </span>
  );
};
