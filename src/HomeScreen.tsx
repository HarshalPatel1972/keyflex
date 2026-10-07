import { useState } from "react";
import { appName, Status, Tip } from "./api";
import { Keys } from "./Keys";
import { Mascot } from "./Mascot";

interface Props {
  tips: Tip[];
  status: Status;
  onSeeAll: () => void;
}

/** Things the character says when you poke it, by mood. */
function chatter(status: Status, hidden: number): string[] {
  switch (status.mood) {
    case "sleepy":
      return ["Zzz... tips are paused.", "Five more minutes.", "Wake me from the tray when you want me back."];
    case "proud":
      return ["You used a shortcut. I saw. I'm still not over it.", "That's my human.", "Do it again. I dare you."];
    case "pleading":
      return [
        `Just try ${status.waiting_on ?? "it"}. Once. For me.`,
        "I'm not sad. This is just my face now.",
        "It's fine. I'll wait. I'm very good at waiting.",
      ];
    default:
      return [
        "Go on, do something the long way. I dare you.",
        hidden > 0
          ? `I know ${hidden} ${hidden === 1 ? "trick" : "tricks"} you haven't found yet.`
          : "You've found every trick I know. For now.",
        "Click a menu. Any menu. I'll be right there.",
        "Hey! That tickles.",
      ];
  }
}

export function HomeScreen({ tips, status, onSeeAll }: Props) {
  const [pokes, setPokes] = useState(0);

  const learned = tips.filter((tip) => tip.learned).length;
  const found = tips.filter((tip) => tip.shown > 0 || tip.learned).length;
  const hidden = tips.length - found;
  const lines = chatter(status, hidden);
  const latest = tips
    .filter((tip) => tip.shown > 0)
    .sort((a, b) => b.last_shown - a.last_shown)
    .slice(0, 3);

  return (
    <div className="page home">
      <section className="hero">
        <button
          // Re-mounting on each poke restarts the squish animation.
          key={pokes}
          className={pokes > 0 ? "hero-face poked" : "hero-face"}
          aria-label="Poke the Keyflex character"
          onClick={() => setPokes(pokes + 1)}
        >
          <Mascot mood={status.mood} size={168} />
        </button>
        <div className="bubble" aria-live="polite">
          {lines[pokes % lines.length]}
        </div>
      </section>

      <section className="card progress">
        <div className="progress-head">
          <h2>
            <span className="big">{learned}</span> of {tips.length} shortcuts are yours
          </h2>
          <span className="muted">{found - learned} in progress</span>
        </div>
        <div className="bar" role="img" aria-label={`${learned} learned, ${found - learned} in progress, ${hidden} not found yet`}>
          <span className="bar-found" style={{ width: `${(found / tips.length) * 100}%` }} />
          <span className="bar-learned" style={{ width: `${(learned / tips.length) * 100}%` }} />
        </div>
        <div className="legend">
          <span className="dot-learned">Yours</span>
          <span className="dot-found">Found, not used yet</span>
          <span className="dot-hidden">{hidden} still hiding</span>
        </div>
      </section>

      <section>
        <div className="section-head">
          <h2>Latest finds</h2>
          <button className="link" onClick={onSeeAll}>
            See all shortcuts
          </button>
        </div>
        {latest.length === 0 ? (
          <div className="card empty">
            <strong>Nothing found yet.</strong>
            <span>Open a menu in Chrome or File Explorer the long way, and see who shows up.</span>
          </div>
        ) : (
          <div className="finds">
            {latest.map((tip) => (
              <div key={tip.id} className={tip.learned ? "card find learned" : "card find"}>
                <Keys keys={tip.keys} />
                <span className="find-line">{tip.line}</span>
                <span className="muted">{tip.learned ? "Yours now" : appName(tip.apps[0])}</span>
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}
