"""
Generate the NSIS installer header and sidebar BMPs from the current
icon.png. NSIS Modern UI 1 expects 24-bit BMPs at these exact sizes:
  - header:  150x57   (top of every installer page)
  - sidebar: 164x314  (left side of the welcome/finish pages)
"""
import os
from PIL import Image

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ICON = os.path.join(ROOT, "src-tauri", "icons", "icon.png")
OUT  = os.path.join(ROOT, "src-tauri", "icons")
BG   = (255, 179, 0)  # #ffb300 — Vayou accent


def composite(canvas_w: int, canvas_h: int, icon_size: int, pos_x: int, pos_y: int, name: str) -> None:
    icon = Image.open(ICON).convert("RGBA").resize((icon_size, icon_size), Image.Resampling.LANCZOS)
    canvas = Image.new("RGB", (canvas_w, canvas_h), BG)
    canvas.paste(icon, (pos_x, pos_y), mask=icon)
    canvas.save(os.path.join(OUT, name), format="BMP")
    print(f"  {name}")


# Header — small icon centered both horizontally and vertically
composite(150, 57, 40, (150 - 40) // 2, (57 - 40) // 2, "nsis-header.bmp")

# Sidebar — large icon centered horizontally, ~25% from top
composite(164, 314, 120, (164 - 120) // 2, 70, "nsis-sidebar.bmp")
