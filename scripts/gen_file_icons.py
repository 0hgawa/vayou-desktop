"""
Generate per-extension file association icons for Vayou.

Each icon is a colored squircle with the Vayou play mark and a small
extension chip in the bottom-right corner. The chip is omitted for sizes
< 64 because it would be illegible.

Output: src-tauri/icons/file/{ext}.ico  (multi-size: 16/24/32/48/64/128/256)
"""
import os
from PIL import Image, ImageDraw, ImageFont

# (color, chip-color) per extension. Background dictates the family;
# chip color stays constant (charcoal) for cohesion.
COLORS = {
    # video — default amber for the dominant family
    "mp4":  "#ffb300",
    "m4v":  "#ffb300",
    "ts":   "#ffb300",
    "mpg":  "#ffb300",
    "mpeg": "#ffb300",
    # video — distinguishing colors
    "mkv":  "#7c3aed",  # deep purple — anime/MKVToolNix association
    "avi":  "#0891b2",  # cyan — legacy/Windows feel
    "mov":  "#e11d48",  # rose — Quicktime/Apple
    "webm": "#16a34a",  # green — web/Google
    "wmv":  "#ea580c",  # orange — secondary
    "flv":  "#ea580c",  # orange — secondary
    # audio — single warm tone, slightly different from video amber
    "mp3":  "#fbbf24",
    "flac": "#fbbf24",
    "wav":  "#fbbf24",
    "ogg":  "#fbbf24",
    "aac":  "#fbbf24",
    "wma":  "#fbbf24",
    "m4a":  "#fbbf24",
    "opus": "#fbbf24",
}

ICO_SIZES = [16, 24, 32, 48, 64, 128, 256]
CHIP_MIN = 64
FONT_PATH = "C:/Windows/Fonts/consolab.ttf"

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT_DIR = os.path.join(ROOT, "src-tauri", "icons", "file")


def make_icon(size: int, color: str, ext: str) -> Image.Image:
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # Squircle (rounded square ~24% radius)
    radius = max(int(size * 0.24), 2)
    draw.rounded_rectangle((0, 0, size - 1, size - 1), radius=radius, fill=color)

    # Play mark — outer white triangle with inner triangle the same color
    # as the squircle, giving a "stencil" / outlined look.
    cx, cy = size / 2, size / 2
    w = size * 0.40
    h = w * 1.10
    offset_x = size * 0.05  # nudge right for optical balance
    outer = [
        (cx - w / 2 + offset_x, cy - h / 2),
        (cx - w / 2 + offset_x, cy + h / 2),
        (cx + w / 2 + offset_x, cy),
    ]
    draw.polygon(outer, fill="white")
    iw, ih = w * 0.48, h * 0.48
    inner = [
        (cx - iw / 2 + offset_x, cy - ih / 2),
        (cx - iw / 2 + offset_x, cy + ih / 2),
        (cx + iw / 2 + offset_x, cy),
    ]
    draw.polygon(inner, fill=color)

    # Extension chip in the bottom-right corner — only readable from 64+.
    if size >= CHIP_MIN:
        chip_h = max(int(size * 0.20), 14)
        font_px = int(chip_h * 0.62)
        font = ImageFont.truetype(FONT_PATH, font_px)
        text = ext.upper()
        bbox = draw.textbbox((0, 0), text, font=font)
        tw = bbox[2] - bbox[0]
        th = bbox[3] - bbox[1]
        pad_x = int(chip_h * 0.45)
        chip_w = tw + 2 * pad_x
        margin = int(size * 0.06)
        x1 = size - chip_w - margin
        y1 = size - chip_h - margin
        x2 = size - margin
        y2 = size - margin
        # Subtle drop shadow under the chip
        shadow_offset = max(1, int(size * 0.008))
        draw.rounded_rectangle(
            (x1 + shadow_offset, y1 + shadow_offset, x2 + shadow_offset, y2 + shadow_offset),
            radius=chip_h // 2,
            fill=(0, 0, 0, 80),
        )
        draw.rounded_rectangle((x1, y1, x2, y2), radius=chip_h // 2, fill=(22, 22, 26, 235))
        tx = x1 + (chip_w - tw) / 2 - bbox[0]
        ty = y1 + (chip_h - th) / 2 - bbox[1]
        draw.text((tx, ty), text, fill=(255, 255, 255, 240), font=font)

    return img


def write_ico(path: str, color: str, ext: str) -> None:
    images = [make_icon(s, color, ext) for s in ICO_SIZES]
    # Pillow writes a multi-size ICO when given append_images of different sizes.
    images[-1].save(path, format="ICO", append_images=images[:-1], sizes=[(s, s) for s in ICO_SIZES])


def main() -> None:
    os.makedirs(OUT_DIR, exist_ok=True)
    for ext, color in COLORS.items():
        out = os.path.join(OUT_DIR, f"{ext}.ico")
        write_ico(out, color, ext)
        print(f"  {out}")


if __name__ == "__main__":
    main()
