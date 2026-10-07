import { useEffect, useId, useRef } from "react";
import type { Mood } from "./api";

// The same drawing as design/moods.py, but live: it bobs, blinks and watches the cursor.
const DARK = "#062A18";
const MOUTH = "#053D22";
const BROW = "#0A6E3E";

/** How far, in drawing units, the pupils can travel towards the cursor. */
const LOOK_REACH = 13;

interface Props {
  mood: Mood;
  size?: number;
  /** Pupils follow the cursor. Off for small or purely decorative faces. */
  follow?: boolean;
  className?: string;
}

export function Mascot({ mood, size = 120, follow = true, className = "" }: Props) {
  const id = useId();
  const ref = useRef<SVGSVGElement>(null);

  useEffect(() => {
    if (!follow) return;
    let frame = 0;
    const look = (event: MouseEvent) => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        const svg = ref.current;
        if (!svg) return;
        const box = svg.getBoundingClientRect();
        const dx = event.clientX - (box.left + box.width / 2);
        const dy = event.clientY - (box.top + box.height * 0.42);
        const distance = Math.hypot(dx, dy) || 1;
        const reach = LOOK_REACH * Math.min(1, distance / 200);
        svg.style.setProperty("--look-x", `${(dx / distance) * reach}px`);
        svg.style.setProperty("--look-y", `${(dy / distance) * reach}px`);
      });
    };
    window.addEventListener("mousemove", look);
    return () => {
      window.removeEventListener("mousemove", look);
      cancelAnimationFrame(frame);
    };
  }, [follow]);

  const stroke = (d: string, width = 20, color = BROW) => (
    <path d={d} fill="none" stroke={color} strokeWidth={width} strokeLinecap="round" strokeLinejoin="round" />
  );
  const eye = (x: number, dx = 12, dy = -6, pupil = 36) => (
    <g>
      <ellipse cx={x} cy={214} rx={62} ry={70} fill="#fff" />
      <g className="pupil">
        <circle cx={x + dx} cy={214 + dy} r={pupil} fill={DARK} />
        <circle cx={x + dx + 14} cy={214 + dy - 14} r={13} fill="#fff" />
      </g>
    </g>
  );

  return (
    <svg
      ref={ref}
      className={`mascot ${className}`}
      width={size}
      height={size}
      viewBox="0 0 512 512"
      aria-hidden="true"
    >
      <defs>
        <linearGradient id={`${id}top`} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stopColor="#4FE39A" />
          <stop offset="1" stopColor="#12B76A" />
        </linearGradient>
        <clipPath id={`${id}left`}>
          <ellipse cx="186" cy="214" rx="62" ry="70" />
        </clipPath>
        <clipPath id={`${id}right`}>
          <ellipse cx="326" cy="214" rx="62" ry="70" />
        </clipPath>
      </defs>

      <rect x="32" y="60" width="448" height="420" rx="112" fill="#0A7A45" />
      <rect x="32" y="44" width="448" height="408" rx="112" fill="#0E9F5B" />
      <rect x="66" y="44" width="380" height="350" rx="88" fill={`url(#${id}top)`} />
      <ellipse cx="124" cy="296" rx="30" ry="20" fill="#FFC53D" opacity="0.5" />
      <ellipse cx="388" cy="296" rx="30" ry="20" fill="#FFC53D" opacity="0.5" />

      {mood === "knowing" && (
        <>
          <g className="eyes">
            {eye(186)}
            {eye(326)}
          </g>
          {stroke("M140 124 L224 128")}
          {stroke("M288 112 L372 92")}
          {stroke("M218 316 Q268 346 306 300", 18, MOUTH)}
        </>
      )}

      {mood === "wink" && (
        <>
          <g className="eyes">{eye(186)}</g>
          {stroke("M272 222 Q326 168 380 222", 26, DARK)}
          {stroke("M140 118 L224 112")}
          <path d="M212 304 Q256 364 300 304 Z" fill={MOUTH} stroke={MOUTH} strokeWidth="14" strokeLinejoin="round" />
        </>
      )}

      {mood === "cheeky" && (
        <>
          {eye(186, 16, 14, 32)}
          {eye(326, 16, 14, 32)}
          <rect x="110" y="130" width="152" height="78" fill="#2FCB80" clipPath={`url(#${id}left)`} />
          <rect x="250" y="130" width="152" height="78" fill="#2FCB80" clipPath={`url(#${id}right)`} />
          {stroke("M126 208 L246 208", 12, DARK)}
          {stroke("M266 208 L386 208", 12, DARK)}
          {stroke("M136 116 L228 122")}
          {stroke("M284 122 L376 116")}
          {stroke("M204 312 Q272 356 320 294", 18, MOUTH)}
        </>
      )}

      {mood === "pleading" && (
        <>
          <g className="eyes">
            {[186, 326].map((x) => (
              <g key={x}>
                <ellipse cx={x} cy={218} rx={66} ry={76} fill="#fff" />
                <g className="pupil small">
                  <circle cx={x} cy={216} r={48} fill={DARK} />
                  <circle cx={x + 18} cy={194} r={17} fill="#fff" />
                  <circle cx={x - 16} cy={240} r={9} fill="#fff" />
                </g>
              </g>
            ))}
          </g>
          {stroke("M134 124 L222 96")}
          {stroke("M290 96 L378 124")}
          {stroke("M228 336 Q256 314 284 336", 16, MOUTH)}
          <path className="tear" d="M392 262 Q410 296 392 308 Q374 296 392 262 Z" fill="#A9DCFF" />
        </>
      )}

      {mood === "proud" && (
        <>
          {stroke("M132 228 Q186 158 240 228", 26, DARK)}
          {stroke("M272 228 Q326 158 380 228", 26, DARK)}
          <path d="M198 290 Q256 384 314 290 Z" fill={MOUTH} stroke={MOUTH} strokeWidth="14" strokeLinejoin="round" />
          <ellipse cx="256" cy="336" rx="24" ry="13" fill="#FF8A9A" />
          <path
            className="sparkle"
            d="M410 70 C 414 96, 420 102, 446 106 C 420 110, 414 116, 410 142 C 406 116, 400 110, 374 106 C 400 102, 406 96, 410 70 Z"
            fill="#FFE27A"
          />
          <path
            className="sparkle late"
            d="M112 96 C 114 110, 118 114, 132 116 C 118 118, 114 122, 112 136 C 110 122, 106 118, 92 116 C 106 114, 110 110, 112 96 Z"
            fill="#FFE27A"
          />
        </>
      )}

      {mood === "sleepy" && (
        <>
          {stroke("M132 212 Q186 258 240 212", 24, DARK)}
          {stroke("M272 212 Q326 258 380 212", 24, DARK)}
          <ellipse className="snore" cx="256" cy="322" rx="16" ry="20" fill={MOUTH} />
          <g className="zzz">{stroke("M372 62 h46 l-46 50 h46", 13, "#fff")}</g>
          <g className="zzz late">{stroke("M318 104 h28 l-28 30 h28", 10, "#fff")}</g>
        </>
      )}
    </svg>
  );
}
