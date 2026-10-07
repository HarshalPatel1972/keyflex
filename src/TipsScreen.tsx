import { useState } from "react";
import { api, appName, Tip } from "./api";
import { Keys } from "./Keys";
import { Mascot } from "./Mascot";

/** Tips grouped by the apps they belong to, in the order they first appear. */
function groupByApps(tips: Tip[]): [string, Tip[]][] {
  const groups = new Map<string, Tip[]>();
  for (const tip of tips) {
    const label = tip.apps.map(appName).join(", ");
    groups.set(label, [...(groups.get(label) ?? []), tip]);
  }
  return [...groups];
}

export function TipsScreen({ tips }: { tips: Tip[] }) {
  // Hidden cards the user chose to peek at, until the screen is left.
  const [peeked, setPeeked] = useState<Set<string>>(new Set());
  const learned = tips.filter((tip) => tip.learned).length;

  const peek = (id: string) => setPeeked(new Set(peeked).add(id));

  return (
    <div className="page">
      <header className="page-header">
        <h1>Shortcuts</h1>
        <p>
          {learned} of {tips.length} are yours. The rest are out there, waiting to be found.
        </p>
      </header>

      {groupByApps(tips).map(([label, group]) => (
        <section key={label}>
          <h2>{label}</h2>
          <div className="collection">
            {group.map((tip) => (
              <Card key={tip.id} tip={tip} where={label} peeked={peeked.has(tip.id)} onPeek={() => peek(tip.id)} />
            ))}
          </div>
        </section>
      ))}
    </div>
  );
}

function Card({ tip, where, peeked, onPeek }: { tip: Tip; where: string; peeked: boolean; onPeek: () => void }) {
  const found = tip.shown > 0 || tip.learned;

  if (!found && !peeked) {
    return (
      <button className="card shortcut hidden" onClick={onPeek}>
        <span className="keys">
          <kbd>?</kbd>
          <kbd>?</kbd>
        </span>
        <span className="shortcut-line">Hiding somewhere in {where}. Do it the long way and I'll show you.</span>
        <span className="shortcut-foot">
          <span className="chip">Not found yet</span>
          <span className="peek">Peek</span>
        </span>
      </button>
    );
  }

  const state = tip.muted ? "muted" : tip.learned ? "learned" : found ? "found" : "peeked";
  const chip = {
    muted: "Muted",
    learned: "Yours",
    found: tip.shown === 1 ? "Seen once" : `Seen ${tip.shown} times`,
    peeked: "Peeked",
  }[state];

  return (
    <div className={`card shortcut ${state}`}>
      <div className="shortcut-top">
        <Keys keys={tip.keys} />
        {state === "learned" && <Mascot mood="proud" size={34} follow={false} className="still" />}
        {state === "found" && (
          <Mascot mood={tip.shown > 1 ? "cheeky" : "wink"} size={34} follow={false} className="still" />
        )}
      </div>
      <span className="shortcut-line">{tip.line}</span>
      <span className="shortcut-foot">
        <span className="chip">{chip}</span>
        {!tip.learned && (
          <label className="switch" title={tip.muted ? "Show this tip again" : "Stop showing this tip"}>
            <input
              type="checkbox"
              checked={!tip.muted}
              aria-label={`Show the ${tip.keys} tip`}
              onChange={(event) => void api.setTipMuted(tip.id, !event.target.checked)}
            />
            <span />
          </label>
        )}
      </span>
    </div>
  );
}
