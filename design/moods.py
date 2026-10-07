"""Writes the character's expressions as SVG files into core/assets/moods/.

Run from the repository root:  python design/moods.py
Then render each to PNG (see design/render-moods.ps1).
"""
import os

DARK = "#062A18"
MOUTH = "#053D22"
BROW = "#0A6E3E"

BODY = '''  <defs>
    <linearGradient id="top" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#4FE39A"/>
      <stop offset="1" stop-color="#12B76A"/>
    </linearGradient>
    <clipPath id="left-eye"><ellipse cx="186" cy="214" rx="62" ry="70"/></clipPath>
    <clipPath id="right-eye"><ellipse cx="326" cy="214" rx="62" ry="70"/></clipPath>
  </defs>
  <rect x="32" y="60" width="448" height="420" rx="112" fill="#0A7A45"/>
  <rect x="32" y="44" width="448" height="408" rx="112" fill="#0E9F5B"/>
  <rect x="66" y="44" width="380" height="350" rx="88" fill="url(#top)"/>
  <ellipse cx="124" cy="296" rx="30" ry="20" fill="#FFC53D" opacity="0.5"/>
  <ellipse cx="388" cy="296" rx="30" ry="20" fill="#FFC53D" opacity="0.5"/>
'''


def eye(x, dx=12, dy=-6, pupil=36):
    return f'''  <ellipse cx="{x}" cy="214" rx="62" ry="70" fill="#FFFFFF"/>
  <circle cx="{x + dx}" cy="{214 + dy}" r="{pupil}" fill="{DARK}"/>
  <circle cx="{x + dx + 14}" cy="{214 + dy - 14}" r="13" fill="#FFFFFF"/>
'''


def stroke(d, width=20, color=BROW):
    return f'  <path d="{d}" fill="none" stroke="{color}" stroke-width="{width}" stroke-linecap="round" stroke-linejoin="round"/>\n'


MOODS = {
    # "I know a trick." The logo.
    "knowing": eye(186) + eye(326)
    + stroke("M140 124 L224 128") + stroke("M288 112 L372 92")
    + stroke("M218 316 Q268 346 306 300", 18, MOUTH),

    # First time a tip is shown: "psst".
    "wink": eye(186)
    + stroke("M272 222 Q326 168 380 222", 26, DARK)
    + stroke("M140 118 L224 112")
    + f'  <path d="M212 304 Q256 364 300 304 Z" fill="{MOUTH}" stroke="{MOUTH}" stroke-width="14" stroke-linejoin="round"/>\n',

    # Second time: half-closed eyes, "we talked about this".
    "cheeky": eye(186, 16, 14, 32) + eye(326, 16, 14, 32)
    + '  <rect x="110" y="130" width="152" height="78" fill="#2FCB80" clip-path="url(#left-eye)"/>\n'
    + '  <rect x="250" y="130" width="152" height="78" fill="#2FCB80" clip-path="url(#right-eye)"/>\n'
    + stroke("M126 208 L246 208", 12, DARK) + stroke("M266 208 L386 208", 12, DARK)
    + stroke("M136 116 L228 122") + stroke("M284 122 L376 116")
    + stroke("M204 312 Q272 356 320 294", 18, MOUTH),

    # Third and last time: huge wet eyes.
    "pleading": ''.join(
        f'''  <ellipse cx="{x}" cy="218" rx="66" ry="76" fill="#FFFFFF"/>
  <circle cx="{x}" cy="216" r="48" fill="{DARK}"/>
  <circle cx="{x + 18}" cy="194" r="17" fill="#FFFFFF"/>
  <circle cx="{x - 16}" cy="240" r="9" fill="#FFFFFF"/>
''' for x in (186, 326))
    + stroke("M134 124 L222 96") + stroke("M290 96 L378 124")
    + stroke("M228 336 Q256 314 284 336", 16, MOUTH)
    + '  <path d="M392 262 Q410 296 392 308 Q374 296 392 262 Z" fill="#A9DCFF"/>\n',

    # You used the shortcut: eyes squeezed shut with joy.
    "proud": stroke("M132 228 Q186 158 240 228", 26, DARK)
    + stroke("M272 228 Q326 158 380 228", 26, DARK)
    + f'  <path d="M198 290 Q256 384 314 290 Z" fill="{MOUTH}" stroke="{MOUTH}" stroke-width="14" stroke-linejoin="round"/>\n'
    + '  <ellipse cx="256" cy="336" rx="24" ry="13" fill="#FF8A9A"/>\n'
    + '  <path d="M410 70 C 414 96, 420 102, 446 106 C 420 110, 414 116, 410 142 C 406 116, 400 110, 374 106 C 400 102, 406 96, 410 70 Z" fill="#FFE27A"/>\n'
    + '  <path d="M112 96 C 114 110, 118 114, 132 116 C 118 118, 114 122, 112 136 C 110 122, 106 118, 92 116 C 106 114, 110 110, 112 96 Z" fill="#FFE27A"/>\n',

    # Paused: asleep.
    "sleepy": stroke("M132 212 Q186 258 240 212", 24, DARK)
    + stroke("M272 212 Q326 258 380 212", 24, DARK)
    + f'  <ellipse cx="256" cy="322" rx="16" ry="20" fill="{MOUTH}"/>\n'
    + stroke("M372 62 h46 l-46 50 h46", 13, "#FFFFFF")
    + stroke("M318 104 h28 l-28 30 h28", 10, "#FFFFFF"),
}

out = os.path.join("core", "assets", "moods")
os.makedirs(out, exist_ok=True)
for name, face in MOODS.items():
    svg = f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512">\n{BODY}{face}</svg>\n'
    with open(os.path.join(out, f"{name}.svg"), "w", encoding="utf-8", newline="\n") as f:
        f.write(svg)
print("wrote", ", ".join(MOODS))
