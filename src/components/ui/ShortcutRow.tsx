import React from "react";
import { KeyChip } from "./KeyChip";

interface ShortcutRowProps {
  shortcut: string;
  count?: number;
  showBar?: boolean;
  maxCount?: number;
}

export const ShortcutRow: React.FC<ShortcutRowProps> = ({
  shortcut,
  count,
  showBar,
  maxCount,
}) => {
  const keys = shortcut.split("+");

  return (
    <div className="shortcut-row-container">
      <div className="shortcut-row">
        {keys.map((key, i) => (
          <React.Fragment key={i}>
            <KeyChip>{key}</KeyChip>
            {i < keys.length - 1 && <span className="plus">+</span>}
          </React.Fragment>
        ))}
      </div>
      {showBar && count !== undefined && maxCount !== undefined && (
        <div className="count-bar-container">
          <div
            className="count-bar"
            style={{ width: `${(count / maxCount) * 100}%` }}
          />
        </div>
      )}
    </div>
  );
};
