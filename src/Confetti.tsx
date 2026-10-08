import { useEffect, useState } from "react";

const COLORS = ["#3ddc84", "#ffc53d", "#ff8a9a", "#7fb2f0", "#fbf8f1"];
const PIECES = 70;
const LIFETIME_MS = 2600;

/** A burst of confetti over the whole window, once. Purely decorative. */
export function Confetti() {
  const [alive, setAlive] = useState(true);
  // Fixed per burst, so re-renders do not reshuffle pieces mid-flight.
  const [pieces] = useState(() =>
    Array.from({ length: PIECES }, (_, index) => ({
      left: Math.random() * 100,
      delay: Math.random() * 0.5,
      duration: 1.5 + Math.random() * 1.0,
      drift: (Math.random() - 0.5) * 220,
      spin: (Math.random() - 0.5) * 1080,
      color: COLORS[index % COLORS.length],
      wide: index % 3 === 0,
    })),
  );

  useEffect(() => {
    const timer = setTimeout(() => setAlive(false), LIFETIME_MS);
    return () => clearTimeout(timer);
  }, []);

  if (!alive) return null;
  return (
    <div className="confetti" aria-hidden="true">
      {pieces.map((piece, index) => (
        <span
          key={index}
          className={piece.wide ? "wide" : ""}
          style={
            {
              left: `${piece.left}%`,
              background: piece.color,
              animationDelay: `${piece.delay}s`,
              animationDuration: `${piece.duration}s`,
              "--drift": `${piece.drift}px`,
              "--spin": `${piece.spin}deg`,
            } as React.CSSProperties
          }
        />
      ))}
    </div>
  );
}
