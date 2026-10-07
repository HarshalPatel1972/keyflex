"""The character's expressions.

Each mood is a face drawn on the keycap body, in the colour that suits the
feeling (as emoji do): green is the everyday colour, blue is sad, red is
grumpy, gold is star-struck, and so on.

Run from the repository root:  python design/moods.py
It writes one SVG per mood into core/assets/moods/ and public/moods/.
Then run design/render-moods.ps1 to turn them into the PNGs the app embeds.
"""
import math
import os

PUPIL = "#0B1A14"

#          top-light   top-dark   sides      bottom     eyebrow    eyelid     cheek      mouth
PALETTES = {
    "green":  ("#4FE39A", "#12B76A", "#0E9F5B", "#0A7A45", "#0A6E3E", "#2FCB80", "#FFC53D", "#053D22"),
    "teal":   ("#7BE8E0", "#22C7BC", "#14A69C", "#0B7D76", "#0A6E68", "#4FD6CC", "#FF9DB0", "#06443F"),
    "orange": ("#FFC58A", "#FF9440", "#F07A1A", "#C25A08", "#A8480A", "#FFAB63", "#FF6F6F", "#6B2A00"),
    "gold":   ("#FFE27A", "#FFC02E", "#F0A400", "#C27F00", "#9C6500", "#FFD154", "#FF8A5C", "#6B4300"),
    "purple": ("#C4A6FF", "#8E6BFF", "#7250E6", "#5236B8", "#45289E", "#A98BFF", "#FF8FC7", "#2A1670"),
    "lilac":  ("#E3C8FF", "#B58CF2", "#9A6FE0", "#7A50BF", "#6A41AD", "#CBA8F8", "#FF9DC8", "#3D2270"),
    "grey":   ("#C6CFD6", "#9AA6B2", "#7F8C99", "#5E6B78", "#55616D", "#B0BBC6", "#E6A5A5", "#2E3740"),
    "red":    ("#FF9A8A", "#FF5A4F", "#E0423A", "#B32B26", "#8F1F1B", "#FF7A6E", "#FF3D3D", "#5E100D"),
    "blue":   ("#8CCBFF", "#4A97FF", "#2F78E6", "#1E57B8", "#1A4A9E", "#6BB2FF", "#FF9DB0", "#0D2E66"),
    "pink":   ("#FFB0D6", "#FF6FB0", "#E65597", "#BA3676", "#A52B68", "#FF8FC4", "#FF4F7A", "#6B1240"),
    "indigo": ("#A3AEFF", "#6573EE", "#4D5AD0", "#3643A6", "#2F3A94", "#8490F7", "#C9A0FF", "#1A2260"),
}


class Face:
    """Drawing helpers that know the mood's colours."""

    def __init__(self, palette):
        self.hi, self.lo, self.side, self.edge, self.brow_color, self.lid, self.cheek, self.mouth_color = PALETTES[palette]

    def body(self):
        return f'''  <defs>
    <linearGradient id="top" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="{self.hi}"/>
      <stop offset="1" stop-color="{self.lo}"/>
    </linearGradient>
    <clipPath id="left-eye"><ellipse cx="186" cy="214" rx="62" ry="70"/></clipPath>
    <clipPath id="right-eye"><ellipse cx="326" cy="214" rx="62" ry="70"/></clipPath>
  </defs>
  <rect x="32" y="60" width="448" height="420" rx="112" fill="{self.edge}"/>
  <rect x="32" y="44" width="448" height="408" rx="112" fill="{self.side}"/>
  <rect x="66" y="44" width="380" height="350" rx="88" fill="url(#top)"/>
  <ellipse cx="124" cy="296" rx="30" ry="20" fill="{self.cheek}" opacity="0.5"/>
  <ellipse cx="388" cy="296" rx="30" ry="20" fill="{self.cheek}" opacity="0.5"/>
'''

    def stroke(self, d, width=20, color=None):
        color = color or self.brow_color
        return f'  <path d="{d}" fill="none" stroke="{color}" stroke-width="{width}" stroke-linecap="round" stroke-linejoin="round"/>\n'

    def dark(self, d, width=26):
        return self.stroke(d, width, PUPIL)

    def mouth(self, d, width=18):
        return self.stroke(d, width, self.mouth_color)

    def open_mouth(self, d, tongue=None):
        out = f'  <path d="{d}" fill="{self.mouth_color}" stroke="{self.mouth_color}" stroke-width="14" stroke-linejoin="round"/>\n'
        if tongue:
            x, y = tongue
            out += f'  <ellipse cx="{x}" cy="{y}" rx="24" ry="13" fill="#FF8A9A"/>\n'
        return out

    def eye(self, x, dx=12, dy=-6, pupil=36, rx=62, ry=70):
        return f'''  <ellipse cx="{x}" cy="214" rx="{rx}" ry="{ry}" fill="#FFFFFF"/>
  <circle cx="{x + dx}" cy="{214 + dy}" r="{pupil}" fill="{PUPIL}"/>
  <circle cx="{x + dx + 14}" cy="{214 + dy - 14}" r="{max(8, pupil // 3 + 1)}" fill="#FFFFFF"/>
'''

    def eyes(self, **kwargs):
        return self.eye(186, **kwargs) + self.eye(326, **kwargs)

    def lids(self, down=78):
        """Eyelids lowered over the top of both eyes, to `down` pixels below their top."""
        line = 130 + down
        return (
            f'  <rect x="110" y="130" width="152" height="{down}" fill="{self.lid}" clip-path="url(#left-eye)"/>\n'
            f'  <rect x="250" y="130" width="152" height="{down}" fill="{self.lid}" clip-path="url(#right-eye)"/>\n'
            + self.dark(f"M126 {line} L246 {line}", 12) + self.dark(f"M266 {line} L386 {line}", 12)
        )


def sparkle(x, y, size, color="#FFE27A"):
    s, t = size, size * 0.22
    return (f'  <path d="M{x} {y - s} C {x + t} {y - t}, {x + t} {y - t}, {x + s} {y} C {x + t} {y + t}, {x + t} {y + t}, {x} {y + s} '
            f'C {x - t} {y + t}, {x - t} {y + t}, {x - s} {y} C {x - t} {y - t}, {x - t} {y - t}, {x} {y - s} Z" fill="{color}"/>\n')


def star(x, y, radius, color):
    points = []
    for i in range(10):
        r = radius if i % 2 == 0 else radius * 0.45
        angle = -math.pi / 2 + i * math.pi / 5
        points.append(f"{x + r * math.cos(angle):.1f},{y + r * math.sin(angle):.1f}")
    return f'  <polygon points="{" ".join(points)}" fill="{color}" stroke="{color}" stroke-width="8" stroke-linejoin="round"/>\n'


def heart(x, y, size, color="#E11D48"):
    s = size
    return (f'  <path d="M{x} {y + s * 0.9} C {x - s * 1.5} {y - s * 0.1}, {x - s * 0.8} {y - s * 1.2}, {x} {y - s * 0.35} '
            f'C {x + s * 0.8} {y - s * 1.2}, {x + s * 1.5} {y - s * 0.1}, {x} {y + s * 0.9} Z" fill="{color}"/>\n')


def tear(x, y):
    return f'  <path d="M{x} {y} Q{x + 18} {y + 34} {x} {y + 46} Q{x - 18} {y + 34} {x} {y} Z" fill="#A9DCFF"/>\n'


HAPPY_EYES = "M132 228 Q186 158 240 228", "M272 228 Q326 158 380 228"
SMILE = "M198 290 Q256 384 314 290 Z"


def knowing(f):  # "I know a trick." The logo.
    return (f.eyes() + f.stroke("M140 124 L224 128") + f.stroke("M288 112 L372 92")
            + f.mouth("M218 316 Q268 346 306 300"))


def wink(f):  # "psst"
    return (f.eye(186) + f.dark("M272 222 Q326 168 380 222") + f.stroke("M140 118 L224 112")
            + f.open_mouth("M212 304 Q256 364 300 304 Z"))


def curious(f):  # "ooh, what's this?"
    return (f.eyes(dx=14, dy=-16) + f.stroke("M140 128 L224 126") + f.stroke("M288 106 L372 84")
            + f'  <ellipse cx="262" cy="324" rx="15" ry="17" fill="{f.mouth_color}"/>\n'
            + f.stroke("M368 96 Q368 66 394 66 Q422 66 422 92 Q422 110 396 118 L396 128", 12, "#FFFFFF")
            + '  <circle cx="396" cy="150" r="8" fill="#FFFFFF"/>\n')


def excited(f):  # "ooh! ooh! I know this one!"
    eyes = "".join(
        f'''  <ellipse cx="{x}" cy="212" rx="64" ry="74" fill="#FFFFFF"/>
  <circle cx="{x}" cy="208" r="42" fill="{PUPIL}"/>
  <circle cx="{x + 16}" cy="190" r="16" fill="#FFFFFF"/>
  <circle cx="{x - 16}" cy="228" r="8" fill="#FFFFFF"/>
''' for x in (186, 326))
    return (eyes + f.stroke("M134 112 Q180 90 226 112") + f.stroke("M286 112 Q332 90 378 112")
            + f.open_mouth("M200 294 Q256 372 312 294 Z", tongue=(256, 334)))


def shocked(f):  # "you did WHAT the long way?"
    return (f.eyes(dx=0, dy=0, pupil=20) + f.stroke("M140 100 L224 90") + f.stroke("M288 90 L372 100")
            + f'  <ellipse cx="256" cy="332" rx="26" ry="34" fill="{f.mouth_color}"/>\n')


def cheeky(f):  # "we talked about this"
    return (f.eyes(dx=16, dy=14, pupil=32) + f.lids(78)
            + f.stroke("M136 116 L228 122") + f.stroke("M284 122 L376 116")
            + f.mouth("M204 312 Q272 356 320 294"))


def smug(f):  # looking away, one corner of the mouth up
    return (f.eyes(dx=24, dy=18, pupil=30) + f.lids(86)
            + f.stroke("M136 132 L226 140") + f.stroke("M286 122 L374 98")
            + f.mouth("M214 318 Q274 344 320 288"))


def deadpan(f):  # "..."
    return (f.eyes(dx=0, dy=20, pupil=30) + f.lids(84)
            + f.stroke("M136 122 L228 122") + f.stroke("M284 122 L376 122")
            + f.mouth("M218 324 L294 324", 16))


def grumpy(f):  # "hmph"
    return (f.eyes(dx=6, dy=10, pupil=32)
            + f.stroke("M122 134 L238 178", 28) + f.stroke("M390 134 L274 178", 28)
            + f.mouth("M214 336 Q256 300 298 336"))


def pleading(f):  # the last plea
    eyes = "".join(
        f'''  <ellipse cx="{x}" cy="218" rx="66" ry="76" fill="#FFFFFF"/>
  <circle cx="{x}" cy="216" r="48" fill="{PUPIL}"/>
  <circle cx="{x + 18}" cy="194" r="17" fill="#FFFFFF"/>
  <circle cx="{x - 16}" cy="240" r="9" fill="#FFFFFF"/>
''' for x in (186, 326))
    return (eyes + f.stroke("M134 124 L222 96") + f.stroke("M290 96 L378 124")
            + f.mouth("M228 336 Q256 314 284 336", 16) + tear(392, 262))


def crying(f):  # full waterworks
    return ('  <rect x="158" y="230" width="34" height="126" rx="17" fill="#C5E8FF"/>\n'
            '  <rect x="320" y="230" width="34" height="126" rx="17" fill="#C5E8FF"/>\n'
            + f.dark("M146 184 L226 216 L146 248", 22) + f.dark("M366 184 L286 216 L366 248", 22)
            + f.stroke("M138 132 L220 110") + f.stroke("M292 110 L374 132")
            + f'  <ellipse cx="256" cy="336" rx="36" ry="30" fill="{f.mouth_color}"/>\n'
            + '  <ellipse cx="256" cy="352" rx="20" ry="10" fill="#FF8A9A"/>\n')


def dizzy(f):  # "I can't watch"
    return (f.dark("M150 180 L222 250", 22) + f.dark("M222 180 L150 250", 22)
            + f.dark("M290 180 L362 250", 22) + f.dark("M362 180 L290 250", 22)
            + f.mouth("M202 328 Q220 306 238 328 T274 328 T310 328", 14))


def proud(f):  # you used the shortcut
    return (f.dark(HAPPY_EYES[0]) + f.dark(HAPPY_EYES[1]) + f.open_mouth(SMILE, tongue=(256, 336))
            + sparkle(410, 106, 36) + sparkle(112, 116, 20))


def starstruck(f):
    stars = "".join(
        f'  <ellipse cx="{x}" cy="214" rx="62" ry="70" fill="#FFFFFF"/>\n' + star(x, 216, 44, "#FF9F1C")
        for x in (186, 326))
    return (stars + f.stroke("M134 112 Q180 92 226 112") + f.stroke("M286 112 Q332 92 378 112")
            + f.open_mouth("M200 294 Q256 376 312 294 Z", tongue=(256, 336))
            + sparkle(420, 92, 26, "#FFFFFF") + sparkle(96, 104, 16, "#FFFFFF"))


def love(f):
    return (heart(186, 210, 54) + heart(326, 210, 54)
            + '  <circle cx="166" cy="186" r="12" fill="#FFFFFF" opacity="0.8"/>\n'
            + '  <circle cx="306" cy="186" r="12" fill="#FFFFFF" opacity="0.8"/>\n'
            + f.open_mouth("M214 300 Q256 358 298 300 Z") + heart(418, 96, 22, "#FFFFFF"))


def cool(f):  # sunglasses
    shades = "#0B1A14"
    return (f'  <rect x="112" y="170" width="140" height="92" rx="34" fill="{shades}"/>\n'
            f'  <rect x="260" y="170" width="140" height="92" rx="34" fill="{shades}"/>\n'
            f'  <rect x="100" y="182" width="312" height="18" rx="9" fill="{shades}"/>\n'
            + f.stroke("M140 236 L176 196", 10, "#FFFFFF") + f.stroke("M288 236 L324 196", 10, "#FFFFFF")
            + f.mouth("M214 320 Q264 350 314 302"))


def party(f):  # graduation day
    confetti = "".join(
        f'  <rect x="{x}" y="{y}" width="20" height="10" rx="3" fill="{c}" transform="rotate({r} {x} {y})"/>\n'
        for x, y, c, r in ((54, 40, "#FF5FA8", 30), (440, 30, "#FFC53D", -24), (470, 150, "#7FB2F0", 50),
                           (24, 170, "#FFC53D", -40), (330, 16, "#FF5FA8", 10), (470, 250, "#FF5FA8", -10)))
    hat = ('  <polygon points="150,84 214,2 262,92" fill="#FF5FA8"/>\n'
           '  <polygon points="172,58 196,26 246,64 232,88" fill="#FFC53D" opacity="0.9"/>\n'
           '  <circle cx="214" cy="14" r="13" fill="#FFFFFF"/>\n')
    return (f.dark(HAPPY_EYES[0]) + f.dark(HAPPY_EYES[1]) + f.open_mouth(SMILE, tongue=(256, 336)) + hat + confetti)


def laughing(f):  # tears of joy
    return (f.dark(HAPPY_EYES[0]) + f.dark(HAPPY_EYES[1])
            + tear(116, 222) + tear(396, 222)
            + f.open_mouth("M186 284 Q256 400 326 284 Z", tongue=(256, 344)))


def sleepy(f):  # paused
    return (f.dark("M132 212 Q186 258 240 212", 24) + f.dark("M272 212 Q326 258 380 212", 24)
            + f'  <ellipse cx="256" cy="322" rx="16" ry="20" fill="{f.mouth_color}"/>\n'
            + f.stroke("M372 62 h46 l-46 50 h46", 13, "#FFFFFF") + f.stroke("M318 104 h28 l-28 30 h28", 10, "#FFFFFF"))


# mood name -> (colour, face)
MOODS = {
    "knowing": ("green", knowing),
    "wink": ("green", wink),
    "curious": ("teal", curious),
    "excited": ("orange", excited),
    "shocked": ("gold", shocked),
    "cheeky": ("green", cheeky),
    "smug": ("purple", smug),
    "deadpan": ("grey", deadpan),
    "grumpy": ("red", grumpy),
    "pleading": ("blue", pleading),
    "crying": ("blue", crying),
    "dizzy": ("lilac", dizzy),
    "proud": ("green", proud),
    "starstruck": ("gold", starstruck),
    "love": ("pink", love),
    "cool": ("teal", cool),
    "party": ("green", party),
    "laughing": ("orange", laughing),
    "sleepy": ("indigo", sleepy),
}


def inner(mood):
    """The drawing for `mood` without the surrounding <svg> element."""
    palette, face = MOODS[mood]
    f = Face(palette)
    return f.body() + face(f)


def svg(mood):
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512">\n{inner(mood)}</svg>\n'


if __name__ == "__main__":
    for folder in (os.path.join("core", "assets", "moods"), os.path.join("public", "moods")):
        os.makedirs(folder, exist_ok=True)
        for name in MOODS:
            with open(os.path.join(folder, f"{name}.svg"), "w", encoding="utf-8", newline="\n") as out:
                out.write(svg(name))
    print("wrote", ", ".join(MOODS))
