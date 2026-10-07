import { useEffect, useState } from "react";
import { api, Mood } from "./api";
import { Mascot } from "./Mascot";

/** Stage coordinates, in pixels of the 680 x 380 mock screen. */
type Point = [number, number];

const REST: Point = [190, 310];
const MENU_BUTTON: Point = [651, 49];
const DOWNLOADS_ITEM: Point = [560, 183];

interface Beat {
  scene: number;
  /** How long the beat lasts; 0 waits for the user. */
  ms: number;
  cursor?: Point;
  click?: boolean;
  menu?: boolean;
  hot?: boolean;
  noticed?: boolean;
  tip?: boolean;
  keys?: boolean;
  pressed?: boolean;
  downloads?: boolean;
}

const BEATS: Beat[] = [
  { scene: 0, ms: 900, cursor: REST },
  { scene: 0, ms: 1000, cursor: MENU_BUTTON },
  { scene: 0, ms: 800, cursor: MENU_BUTTON, click: true, menu: true },
  { scene: 0, ms: 1100, cursor: DOWNLOADS_ITEM, menu: true },
  { scene: 0, ms: 1300, cursor: DOWNLOADS_ITEM, click: true, menu: true, hot: true },
  { scene: 1, ms: 2800, cursor: DOWNLOADS_ITEM, menu: true, hot: true, noticed: true },
  { scene: 2, ms: 4400, cursor: DOWNLOADS_ITEM, menu: true, hot: true, tip: true },
  { scene: 3, ms: 1500, cursor: REST, keys: true },
  { scene: 3, ms: 3400, cursor: REST, keys: true, pressed: true, downloads: true },
  { scene: 4, ms: 0 },
];

const SCENES = [
  {
    title: "You take the long way",
    text: "Menu, then Downloads. Two clicks and a little hunting. Nothing wrong with it. There's just a shorter road.",
  },
  {
    title: "Keyflex notices",
    text: "It sees which button or menu item you clicked. Not what you type. Not what you read.",
  },
  {
    title: "A tip appears right there",
    text: "Beside your cursor, where you're already looking. One line, then it gets out of your way.",
  },
  {
    title: "Next time, two keys",
    text: "Use the shortcut a couple of times and that tip retires for good. No nagging.",
  },
  {
    title: "Private by design",
    text: "Everything happens on this PC. Here is exactly what Keyflex looks at.",
  },
];

const LAST_SCENE = SCENES.length - 1;
const MENU_ITEMS = ["New tab", "New window", "History", "Downloads", "Bookmarks", "Settings"];

const firstBeatOf = (scene: number) => BEATS.findIndex((beat) => beat.scene === scene);

export function Intro({ firstRun, onDone }: { firstRun: boolean; onDone: () => void }) {
  const [beatIndex, setBeatIndex] = useState(0);
  const [autostart, setAutostart] = useState(true);
  const beat = BEATS[beatIndex];
  const scene = SCENES[beat.scene];

  useEffect(() => {
    if (beat.ms === 0) return;
    const timer = setTimeout(() => setBeatIndex((index) => index + 1), beat.ms);
    return () => clearTimeout(timer);
  }, [beatIndex, beat.ms]);

  const goToScene = (index: number) => setBeatIndex(firstBeatOf(index));
  const finish = () => {
    if (firstRun && autostart) void api.setAutostart(true);
    onDone();
  };

  return (
    <div className="intro">
      {beat.scene === LAST_SCENE ? (
        <div className="stage privacy">
          <ul>
            <li>
              <strong>Only button and menu names.</strong> Never web pages, documents, file names or what you type.
            </li>
            <li>
              <strong>Only where you allow it.</strong> Switch tips off for any app, or everywhere, in Settings.
            </li>
            <li>
              <strong>Only the shortcuts it teaches.</strong> No other key ever leaves your keyboard.
            </li>
            <li>
              <strong>Nothing leaves this PC.</strong> No account, no analytics, no network calls.
            </li>
          </ul>
          <button className="ghost" onClick={() => void api.previewTip()}>
            Show me a real tip
          </button>
        </div>
      ) : (
        <Stage beat={beat} beatIndex={beatIndex} />
      )}

      <div className="caption" aria-live="polite">
        <h1>{scene.title}</h1>
        <p>{scene.text}</p>
      </div>

      {firstRun && beat.scene === LAST_SCENE && (
        <label className="check">
          <input type="checkbox" checked={autostart} onChange={(event) => setAutostart(event.target.checked)} />
          Start Keyflex when I sign in to Windows
        </label>
      )}

      <div className="intro-controls">
        <div className="dots">
          {SCENES.map((item, index) => (
            <button
              key={item.title}
              className={index === beat.scene ? "dot active" : "dot"}
              aria-label={`Step ${index + 1}: ${item.title}`}
              onClick={() => goToScene(index)}
            />
          ))}
        </div>
        <div className="intro-buttons">
          {beat.scene === LAST_SCENE ? (
            <>
              <button className="ghost" onClick={() => goToScene(0)}>
                Watch again
              </button>
              <button className="primary" onClick={finish}>
                {firstRun ? "Get started" : "Done"}
              </button>
            </>
          ) : (
            <>
              <button className="ghost" onClick={firstRun ? () => goToScene(LAST_SCENE) : onDone}>
                Skip
              </button>
              <button className="primary" onClick={() => goToScene(beat.scene + 1)}>
                Next
              </button>
            </>
          )}
        </div>
      </div>
    </div>
  );
}

/** A pretend browser that acts out the story. Decorative: the caption carries the meaning. */
function Stage({ beat, beatIndex }: { beat: Beat; beatIndex: number }) {
  const [x, y] = beat.cursor ?? REST;
  // The character joins the story once it has noticed something.
  const mood: Mood | null = beat.pressed ? "proud" : beat.noticed || beat.keys ? "knowing" : null;

  return (
    <div className="stage" aria-hidden="true">
      <div className="mock-tabs">
        <span className="mock-tab">Holiday photos</span>
        <span className="mock-tab dim">Inbox</span>
      </div>
      <div className="mock-toolbar">
        <span className="mock-nav" />
        <span className="mock-nav" />
        <span className="mock-address">photos.example.com</span>
        <span className={beat.menu ? "mock-menu-button open" : "mock-menu-button"}>⋮</span>
      </div>
      <div className="mock-page">
        <span className="block wide" />
        <span className="block" />
        <span className="block" />
        <span className="block short" />
      </div>

      <div className={beat.menu ? "mock-menu open" : "mock-menu"}>
        {MENU_ITEMS.map((item) => {
          const isDownloads = item === "Downloads";
          const classes = ["mock-item"];
          if (isDownloads && beat.hot) classes.push("hot");
          if (isDownloads && beat.noticed) classes.push("noticed");
          return (
            <div key={item} className={classes.join(" ")}>
              {item}
              {isDownloads && <span className="hint">Ctrl+J</span>}
            </div>
          );
        })}
      </div>

      <div className={beat.downloads ? "mock-downloads open" : "mock-downloads"}>
        <strong>Downloads</strong>
        <span>beach-day.zip</span>
        <span>tickets.pdf</span>
      </div>

      <div className={beat.tip ? "mock-tip open" : "mock-tip"}>
        <Mascot mood="wink" size={48} follow={false} className="still" />
        <div>
          <span className="mock-tip-keys">
            <kbd>Ctrl</kbd>
            <kbd>J</kbd>
          </span>
          <span>Psst. Two clicks to reach Downloads? Ctrl+J just walks in the front door.</span>
          <span className="mock-tip-mute">Don't show again</span>
        </div>
      </div>

      <div className={mood ? "stage-mascot open" : "stage-mascot"}>
        {mood && <Mascot mood={mood} size={84} follow={false} />}
      </div>

      <div className={beat.keys ? "mock-keys open" : "mock-keys"}>
        <kbd className={beat.pressed ? "pressed" : ""}>Ctrl</kbd>
        <span>+</span>
        <kbd className={beat.pressed ? "pressed" : ""}>J</kbd>
      </div>

      {beat.click && <span key={beatIndex} className="ripple" style={{ left: x, top: y }} />}
      <svg className="mock-cursor" style={{ left: x, top: y }} width="18" height="22" viewBox="0 0 18 22">
        <path d="M1 1v17l4.6-4.2 3 7 2.6-1.1-3-6.9H15z" fill="#fff" stroke="#000" strokeWidth="1.2" />
      </svg>
    </div>
  );
}
