import React from "react";

interface ScoreRingProps {
  score: number;
}

export const ScoreRing: React.FC<ScoreRingProps> = ({ score }) => {
  const radius = 40;
  const circumference = 2 * Math.PI * radius;
  const offset = circumference - (score / 100) * circumference;

  return (
    <div className="score-ring-container">
      <svg width="100" height="100" viewBox="0 0 100 100">
        <defs>
          <linearGradient id="scoreGrad" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="var(--ember)" />
            <stop offset="100%" stopColor="var(--flame)" />
          </linearGradient>
        </defs>
        {/* Background track */}
        <circle
          cx="50"
          cy="50"
          r={radius}
          stroke="var(--border)"
          strokeWidth="6"
          fill="transparent"
        />
        {/* Progress arc */}
        <circle
          cx="50"
          cy="50"
          r={radius}
          stroke="url(#scoreGrad)"
          strokeWidth="6"
          fill="transparent"
          strokeDasharray={circumference}
          strokeDashoffset={offset}
          strokeLinecap="round"
          transform="rotate(-90 50 50)"
          style={{ transition: "stroke-dashoffset 0.8s ease-out" }}
        />
        <text
          x="50"
          y="48"
          textAnchor="middle"
          dominantBaseline="middle"
          className="score-text"
        >
          {Math.round(score)}
        </text>
        <text
          x="50"
          y="65"
          textAnchor="middle"
          dominantBaseline="middle"
          className="score-subtext"
        >
          / 100
        </text>
      </svg>
    </div>
  );
};
