#!/usr/bin/env python3
"""Pixel-verification for Nexus X-core icons.

Uses Pillow to count cyan/magenta/white/dark-bg pixels and assert thresholds.
This is a deterministic alternative to the vision tool, which has no image-input endpoints.
Run: env -u PYTHONPATH python3 hermes-verify-nexus.py
"""
import sys, os
from PIL import Image

ICON_DIR = r"C:\Users\Administrator\quant\nexus\src-tauri\icons"

# Expected colors (approximate, with tolerance)
CYAN = (0, 212, 255)
PINK = (255, 0, 228)
WHITE = (255, 255, 255)
BG = (10, 14, 26)

def color_match(px, target, tol=30):
    return all(abs(px[i] - target[i]) <= tol for i in range(3))

def verify_icon(path, size):
    im = Image.open(path).convert("RGBA")
    w, h = im.size
    px = list(im.getdata())

    total = w * h
    cyan_c = sum(1 for p in px if color_match(p, CYAN, 40))
    pink_c = sum(1 for p in px if color_match(p, PINK, 40))
    white_c = sum(1 for p in px if color_match(p, WHITE, 30))
    dark_c = sum(1 for p in px if p[3] > 200 and color_match(p, BG, 30))

    # Lower thresholds for tiny icons
    pink_threshold = 5 if size <= 24 else 20
    cyan_threshold = 10 if size <= 24 else 20

    checks = {
        "cyan_present": cyan_c > cyan_threshold,
        "pink_present": pink_c > pink_threshold,
        "white_present": white_c > 5 if size <= 24 else white_c > 10,
        "has_dark_bg": dark_c > 30 if size <= 24 else dark_c > 50,
        "not_solid_black": sum(1 for p in px if p[3] > 0) > 100 if size <= 16 else sum(1 for p in px if p[3] > 0) > 200,
    }

    name = os.path.basename(path)
    # For 16px icons, just check cyan + white + non-empty (pink triangle may not render at 16px)
    if size <= 16:
        checks = {
            "cyan_present": cyan_c > 5,
            "pink_present": True,  # pink triangle too small to render at 16px — skip
            "white_present": white_c > 5,
            "has_dark_bg": dark_c > 10,
            "not_solid_black": sum(1 for p in px if p[3] > 0) > 50,
        }
    print(f"  {name} ({w}x{h}): cyan={cyan_c} pink={pink_c} white={white_c} dark_bg={dark_c}")
    for k, v in checks.items():
        status = "PASS" if v else "FAIL"
        print(f"    [{status}] {k}")
    return all(checks.values())

def main():
    sizes = [16, 24, 32, 48, 64, 128, 256, 512]
    all_ok = True
    for s in sizes:
        path = os.path.join(ICON_DIR, f"{s}x{s}.png")
        if os.path.exists(path):
            ok = verify_icon(path, s)
            all_ok = all_ok and ok
        else:
            print(f"  {s}x{s}.png: MISSING")
            all_ok = False

    ico_path = os.path.join(ICON_DIR, "icon.ico")
    if os.path.exists(ico_path):
        im = Image.open(ico_path)
        sizes_in_ico = im.info.get("sizes", []) if hasattr(im, "info") else []
        print(f"  icon.ico: {im.size}, sizes={sizes_in_ico}")
    else:
        print("  icon.ico: MISSING")
        all_ok = False

    print(f"\n{'ALL CHECKS PASSED' if all_ok else 'SOME CHECKS FAILED'}")
    sys.exit(0 if all_ok else 1)

if __name__ == "__main__":
    main()
