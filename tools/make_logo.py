"""Cut the SaltyShade 3D logo out of its black background and build every icon size.

Usage: python tools/make_logo.py
Reads  overlay/assets/source/logo-source.jpg
Writes overlay/assets/logo.png            (transparent, square, 1024)
       overlay/assets/logo-glow.png       (same, with the violet/cyan neon glow)
       overlay/assets/icons/<n>.png        (16..512, what the app embeds)
       overlay/assets/saltyshade.ico       (Windows exe icon)
"""

from collections import deque
from pathlib import Path

from PIL import Image, ImageChops, ImageFilter

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "overlay/assets/source/logo-source.jpg"
OUT = ROOT / "overlay/assets"

BG_MAX = 10  # a pixel whose brightest channel is at most this is background black
EDGE_LO, EDGE_HI = 8, 34  # soft alpha ramp for anti-aliased edge pixels
MIN_REGION = 150  # enclosed black pockets smaller than this stay (specks inside the art)


def cut_out(im: Image.Image) -> Image.Image:
    im = im.convert("RGB")
    w, h = im.size
    px = im.load()
    dark = bytearray(w * h)
    for y in range(h):
        for x in range(w):
            if max(px[x, y]) <= BG_MAX:
                dark[y * w + x] = 1
    # Every large connected black region is background: the outside and the slits between the S
    # and the globe.
    bg = bytearray(w * h)
    seen = bytearray(w * h)
    for start in range(w * h):
        if not dark[start] or seen[start]:
            continue
        region = []
        q = deque([start])
        seen[start] = 1
        while q:
            i = q.popleft()
            region.append(i)
            x, y = i % w, i // w
            for j in (i - 1 if x else -1, i + 1 if x < w - 1 else -1, i - w if y else -1, i + w if y < h - 1 else -1):
                if j >= 0 and dark[j] and not seen[j]:
                    seen[j] = 1
                    q.append(j)
        if len(region) >= MIN_REGION:
            for i in region:
                bg[i] = 1
    # Pixels within 2 px of the background get a soft alpha from their brightness, and their
    # colour is divided back out of the black they were blended with (no dark fringe).
    near = Image.frombytes("L", (w, h), bytes(v * 255 for v in bg)).filter(ImageFilter.MaxFilter(5)).load()
    out = Image.new("RGBA", (w, h))
    op = out.load()
    for y in range(h):
        for x in range(w):
            i = y * w + x
            r, g, b = px[x, y]
            if bg[i]:
                op[x, y] = (0, 0, 0, 0)
            elif near[x, y]:
                a = min(255, max(0, round((max(r, g, b) - EDGE_LO) * 255 / (EDGE_HI - EDGE_LO))))
                if a == 0:
                    op[x, y] = (0, 0, 0, 0)
                else:
                    k = 255 / a
                    op[x, y] = (min(255, round(r * k)), min(255, round(g * k)), min(255, round(b * k)), a)
            else:
                op[x, y] = (r, g, b, 255)
    return out


def square(im: Image.Image, pad: float = 0.04) -> Image.Image:
    box = im.getbbox()
    im = im.crop(box)
    side = round(max(im.size) * (1 + 2 * pad))
    sq = Image.new("RGBA", (side, side))
    sq.paste(im, ((side - im.width) // 2, (side - im.height) // 2))
    return sq.resize((1024, 1024), Image.LANCZOS)


def glow(im: Image.Image) -> Image.Image:
    """The Neon Cyber-Studio halo: a cyan inner glow and a wide violet outer one behind the art."""
    a = im.getchannel("A")
    out = Image.new("RGBA", im.size)
    for color, radius, strength in (((0x9D, 0x4E, 0xDD), 48, 0.85), ((0x00, 0xF5, 0xD4), 16, 0.7)):
        halo = a.filter(ImageFilter.GaussianBlur(radius)).point(lambda v, s=strength: round(v * s))
        layer = Image.new("RGBA", im.size, color + (0,))
        layer.putalpha(halo)
        out = Image.alpha_composite(out, layer)
    return Image.alpha_composite(out, im)


def main() -> None:
    logo = square(cut_out(Image.open(SRC)))
    logo.save(OUT / "logo.png", optimize=True)
    # The glow needs room: shrink the art inside the square first.
    small = Image.new("RGBA", logo.size)
    inner = logo.resize((820, 820), Image.LANCZOS)
    small.paste(inner, (102, 102))
    glow(small).save(OUT / "logo-glow.png", optimize=True)
    icons = OUT / "icons"
    icons.mkdir(exist_ok=True)
    for n in (16, 24, 32, 48, 64, 128, 256, 512):
        logo.resize((n, n), Image.LANCZOS).save(icons / f"{n}.png", optimize=True)
    logo.save(OUT / "saltyshade.ico", sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
    # The installer's side banner (Inno Setup's wizard image): the glowing logo on obsidian.
    banner = Image.new("RGBA", (328, 628), (0x0B, 0x09, 0x14, 255))
    mark = glow(small).resize((300, 300), Image.LANCZOS)
    banner.alpha_composite(mark, (14, 150))
    banner.convert("RGB").save(ROOT / "installer/wizard.png", optimize=True)
    print("logo written to", OUT)


if __name__ == "__main__":
    main()
