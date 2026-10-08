"""Draws the full-body character in a few poses, as a preview sheet.

Run from the repository root:  python design/mascot.py
Writes design/mascot-poses.html (open it in a browser, or render it to PNG).
"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import moods  # noqa: E402  (the faces live there)

LIMB = "#0E9F5B"
LIMB_DARK = "#0A7A45"
HAND = "#2FCB80"
SHOE = "#FFC53D"
SHOE_DARK = "#E0A100"


def limb(d, width=40):
    return f'<path d="{d}" fill="none" stroke="{LIMB}" stroke-width="{width}" stroke-linecap="round" stroke-linejoin="round"/>'


def hand(x, y):
    return f'<circle cx="{x}" cy="{y}" r="27" fill="{HAND}"/>'


def legs(lift=0):
    """Two short legs and chunky shoes. `lift` tucks them up for a jump."""
    top, foot = 372 - lift, 468 - lift * 1.6
    parts = []
    for x, toe in ((196, -14), (316, 14)):
        parts.append(f'<path d="M{x} {top} L{x} {foot - 8}" stroke="{LIMB_DARK}" stroke-width="40" stroke-linecap="round"/>')
        parts.append(f'<ellipse cx="{x + toe}" cy="{foot + 6}" rx="50" ry="24" fill="{SHOE_DARK}"/>')
        parts.append(f'<ellipse cx="{x + toe}" cy="{foot}" rx="50" ry="22" fill="{SHOE}"/>')
    return "".join(parts)


def figure(mood, behind, in_front, lift=0, tilt=0):
    """The keycap body with a face, limbs behind and in front of it."""
    face = moods.inner(mood)
    shadow_width = 150 - lift * 1.2
    return f'''<svg viewBox="0 0 512 540" width="300" height="316">
  <ellipse cx="256" cy="506" rx="{shadow_width}" ry="16" fill="#000" opacity="0.22"/>
  <g transform="translate(0 {-lift}) rotate({tilt} 256 300)">
    {legs()}
    {behind}
    <g transform="translate(62 26) scale(0.758)">{face}</g>
    {in_front}
  </g>
</svg>'''


POSES = {
    "Waving hello": figure(
        "knowing",
        behind=limb("M100 250 Q52 280 58 338") + hand(58, 338),
        in_front=limb("M414 226 Q470 200 466 124") + hand(466, 118),
        tilt=-3,
    ),
    "Pointing at a tip": figure(
        "wink",
        behind=limb("M100 250 Q60 292 96 330") + hand(98, 332),
        in_front=limb("M414 250 Q462 250 498 232") + hand(500, 230),
        tilt=3,
    ),
    "You did it!": figure(
        "proud",
        behind="",
        in_front=limb("M100 216 Q44 188 40 110") + hand(40, 104) + limb("M414 216 Q470 188 474 110") + hand(474, 104),
        lift=34,
    ),
    "Still waiting...": figure(
        "pleading",
        behind=limb("M100 262 Q70 312 120 350") + hand(124, 352) + limb("M414 262 Q444 312 394 350") + hand(390, 352),
        in_front="",
    ),
    "Paused": figure(
        "sleepy",
        behind=limb("M100 262 Q56 300 70 350") + hand(70, 352) + limb("M414 262 Q458 300 444 350") + hand(444, 352),
        in_front="",
        tilt=6,
    ),
}

cells = "".join(f'<div><div class="pose">{svg}</div><p>{name}</p></div>' for name, svg in POSES.items())
html = f'''<!doctype html>
<meta charset="utf-8">
<style>
  body {{ margin: 0; background: #111815; color: #e4ede6; font: 700 18px "Segoe UI", sans-serif; }}
  .row {{ display: flex; }}
  .row > div {{ padding: 20px 14px 10px; text-align: center; }}
  .light {{ background: #f3eee3; color: #2a2f28; }}
  p {{ margin: 6px 0 0; }}
</style>
<div class="row">{cells}</div>
<div class="row light">{cells}</div>
'''
out = os.path.join("design", "mascot-poses.html")
with open(out, "w", encoding="utf-8", newline="\n") as f:
    f.write(html)
print("wrote", out)
