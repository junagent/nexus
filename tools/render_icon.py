#!/usr/bin/env python3
"""Render Nexus X-core brand icons (PNG + ICO) from scratch with Pillow.

Two intersecting triangles (cyan + magenta) forming an X, white glowing core.
Layered approach: draw each triangle upright on its own layer, rotate the layer,
composite. Avoids scanline-fill artifacts.
"""
import math, os
from PIL import Image, ImageDraw, ImageFilter

CYAN = (0, 212, 255)
PINK = (255, 0, 228)
WHITE = (255, 255, 255)
BG = (10, 14, 26)

def hexa(c, a):
    return (c[0], c[1], c[2], a)

def lerp(c1, c2, t):
    return tuple(int(c1[i] + (c2[i] - c1[i]) * t) for i in range(3))

def tri_layer(size, c_top, c_bot):
    """Upright isoceles triangle on transparent layer, gradient top->bottom."""
    layer = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(layer)
    top_w = size * 0.34
    top_y = size * 0.10
    bot_y = size * 0.86
    cx = size / 2
    pts = [(cx - top_w, top_y), (cx + top_w, top_y), (cx, bot_y)]
    # vertical gradient scanline
    for y in range(int(top_y), int(bot_y) + 1):
        t = (y - top_y) / max(1, (bot_y - top_y))
        col = lerp(c_top, c_bot, t)
        # half-width shrinks linearly toward apex (bottom point)
        hw = top_w * (1 - t)
        d.line([(int(cx - hw), y), (int(cx + hw), y)], fill=hexa(col, 240))
    return layer

def render(size):
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    pad = max(2, size // 16)
    r = max(4, size // 5)
    d.rounded_rectangle([pad, pad, size - pad, size - pad], radius=r, fill=BG)
    d.rounded_rectangle([pad, pad, size - pad, size - pad], radius=r,
                         outline=hexa(CYAN, 70), width=max(1, size // 128))

    # Build each triangle layer at high res, rotate, downscale-composite
    # Use minimum work size to ensure visibility at tiny resolutions
    work = max(size * 8, 256)
    left = tri_layer(work, CYAN, (0, 144, 255)).rotate(-18, resample=Image.BICUBIC, center=(work/2, work*0.30))
    right = tri_layer(work, PINK, (230, 0, 240)).rotate(18, resample=Image.BICUBIC, center=(work/2, work*0.30))
    # crop the working region back to size (rotation may push outside) -> use full then resize
    left = left.resize((size, size), Image.LANCZOS)
    right = right.resize((size, size), Image.LANCZOS)
    img = Image.alpha_composite(img, left)
    img = Image.alpha_composite(img, right)

    # white glowing core dot
    cxp = size / 2
    cyp = size * 0.30
    dot_r = max(2, int(size * 0.07))
    glow = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    gd = ImageDraw.Draw(glow)
    gd.ellipse([cxp - dot_r * 3, cyp - dot_r * 3, cxp + dot_r * 3, cyp + dot_r * 3], fill=hexa(CYAN, 90))
    glow = glow.filter(ImageFilter.GaussianBlur(dot_r))
    img = Image.alpha_composite(img, glow)
    d = ImageDraw.Draw(img)
    d.ellipse([cxp - dot_r, cyp - dot_r, cxp + dot_r, cyp + dot_r], fill=WHITE)
    return img

out = os.path.join(os.path.dirname(__file__), "..", "src-tauri", "icons")
os.makedirs(out, exist_ok=True)
sizes = [16, 24, 32, 48, 64, 128, 256, 512]
imgs = {}
for s in sizes:
    im = render(s)
    imgs[s] = im
    im.save(os.path.join(out, f"{s}x{s}.png"))
imgs[128].save(os.path.join(out, "128x128@2x.png"))
imgs[256].save(os.path.join(out, "icon.ico"), sizes=[(s, s) for s in [16, 24, 32, 48, 64, 128, 256]])
print("rendered", sizes, "->", out)
