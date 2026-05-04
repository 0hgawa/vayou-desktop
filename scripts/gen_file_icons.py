"""
Generate per-extension file association icons for Vayou.

Each icon reuses the master app icon (icon.png) and recolors the yellow
squircle to the extension's brand color. The white "A" mark and the
inner triangle keep the same shape — so the file icon family is visually
the same as the app icon, just tinted differently per format. A small
extension chip in the bottom-right corner identifies the format.

Output: src-tauri/icons/file/{ext}.ico  (multi-size: 16/24/32/48/64/128/256)
"""
import os
import numpy as np
from PIL import Image, ImageDraw, ImageFont

# Background color per extension. mp4/m4v/ts/mpg/mpeg keep the brand
# yellow; the others get a distinguishing tint while sharing the rest
# of the artwork with the app icon.
COLORS = {
    "mp4":  "#ffb300", "m4v": "#ffb300", "ts":   "#ffb300",
    "mpg":  "#ffb300", "mpeg": "#ffb300",
    "mkv":  "#7c3aed",
    "avi":  "#0891b2",
    "mov":  "#e11d48",
    "webm": "#16a34a",
    "wmv":  "#ea580c", "flv": "#ea580c",
    "mp3":  "#fbbf24", "flac": "#fbbf24", "wav":  "#fbbf24",
    "ogg":  "#fbbf24", "aac":  "#fbbf24", "wma":  "#fbbf24",
    "m4a":  "#fbbf24", "opus": "#fbbf24",
}

ICO_SIZES = [16, 24, 32, 48, 64, 128, 256]
CHIP_MIN = 64
FONT_PATH = "C:/Windows/Fonts/consolab.ttf"
BRAND_YELLOW = (255, 179, 0)  # #ffb300 — to be replaced per extension

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ICON = os.path.join(ROOT, "src-tauri", "icons", "icon.png")
OUT  = os.path.join(ROOT, "src-tauri", "icons", "file")


def hex_to_rgb(h: str) -> tuple[int, int, int]:
    h = h.lstrip("#")
    return int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16)


def make_icon(size: int, color: str, ext: str) -> Image.Image:
    # Work on the master 512x512 master, then downscale at the end —
    # remapping after a downscale leaves anti-aliased edge pixels
    # outside our masks and the result looks jagged. The icon only
    # contains three "true" colors: yellow squircle, white "A" mark,
    # and transparent background. Every other pixel is a linear blend
    # of those (anti-alias). The B channel ranges 0 (pure yellow) → 255
    # (pure white), so it's a clean proxy for the blend factor.
    base = Image.open(ICON).convert("RGBA")
    arr = np.array(base).astype(np.float32)
    target = np.array(hex_to_rgb(color), dtype=np.float32)

    # frac_white ∈ [0, 1]: how "white" each pixel was (vs how "yellow").
    frac_white = arr[..., 2] / 255.0
    frac_yellow = 1.0 - frac_white
    # Remap: yellow → white (255), white → target color, smoothly blended.
    new_rgb = (frac_white[..., None] * target + frac_yellow[..., None] * 255.0)
    mask = arr[..., 3] > 0
    arr[mask, :3] = new_rgb[mask]
    img = Image.fromarray(np.clip(arr, 0, 255).astype(np.uint8), "RGBA")
    if size != img.width:
        img = img.resize((size, size), Image.Resampling.LANCZOS)

    # Extension chip in the bottom-right corner (≥64px only — illegible
    # below that, where the colored squircle alone is the identifier).
    if size >= CHIP_MIN:
        draw = ImageDraw.Draw(img)
        chip_h = max(int(size * 0.20), 14)
        font = ImageFont.truetype(FONT_PATH, int(chip_h * 0.62))
        text = ext.upper()
        bbox = draw.textbbox((0, 0), text, font=font)
        tw, th = bbox[2] - bbox[0], bbox[3] - bbox[1]
        chip_w = tw + int(chip_h * 0.9)
        margin = int(size * 0.06)
        x1, y1 = size - chip_w - margin, size - chip_h - margin
        x2, y2 = size - margin, size - margin
        shadow = max(1, int(size * 0.008))
        draw.rounded_rectangle((x1 + shadow, y1 + shadow, x2 + shadow, y2 + shadow),
                               radius=chip_h // 2, fill=(0, 0, 0, 80))
        draw.rounded_rectangle((x1, y1, x2, y2), radius=chip_h // 2, fill=(22, 22, 26, 235))
        draw.text((x1 + (chip_w - tw) / 2 - bbox[0], y1 + (chip_h - th) / 2 - bbox[1]),
                  text, fill=(255, 255, 255, 240), font=font)

    return img


def write_ico(path: str, color: str, ext: str) -> None:
    images = [make_icon(s, color, ext) for s in ICO_SIZES]
    images[-1].save(path, format="ICO", append_images=images[:-1], sizes=[(s, s) for s in ICO_SIZES])


def main() -> None:
    os.makedirs(OUT, exist_ok=True)
    for ext, color in COLORS.items():
        out = os.path.join(OUT, f"{ext}.ico")
        write_ico(out, color, ext)
        print(f"  {out}")


if __name__ == "__main__":
    main()
