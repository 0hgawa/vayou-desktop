# Vayou

A native desktop video player built on **libmpv**, with a Svelte 5 frontend and a Rust + Tauri 2 backend. Optimized for fast startup, low memory footprint, and a small binary.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
![Tauri 2](https://img.shields.io/badge/Tauri-2-FFC131)
![Svelte 5](https://img.shields.io/badge/Svelte-5-FF3E00)
![Rust](https://img.shields.io/badge/Rust-stable-CE422B)
![Platform](https://img.shields.io/badge/platform-Windows-blue)

---

## Features

### Playback
- Play / pause / seek (relative & absolute)
- Speed control from 0.25x to 4x with a quick-pick dropdown
- Frame-by-frame stepping (forward and backward)
- Screenshot capture
- A–B loop (set point A → point B → clear)
- Chapter navigation
- Open from URL (network streams)
- Resume position across sessions (top 20 recent files)
- Sleep timer

### Audio
- Multi-track audio with per-file persistence
- 10-band equalizer
- Loudness normalization (`dynaudnorm`)
- Volume boost up to 200%
- Audio delay adjustment

### Subtitles
- Multi-track embedded + external (SRT / ASS / SSA)
- Per-file selection persistence
- Customizable style: font, size, color, border color, border size, bold, vertical position
- **OpenSubtitles** integration: search, preview, download
- **Automatic translation** of the active subtitle into 12 languages, preserving ASS styling. Falls back to SRT for non-styled sources. Source-track switching mid-playback re-translates from the new track.
- Subtitle delay adjustment

### Video
- Brightness / contrast / saturation
- Aspect ratio cycling (auto, 16:9, 4:3, 21:9, 2.35:1)
- Software deinterlace toggle
- Zoom and pan (numpad-controlled)

### Window & UX
- Frameless transparent window with custom title bar
- Always-on-top toggle
- Drag & drop files onto the window to play
- Custom keybindings (rebindable from settings)
- 13 UI languages: English, Português, Español, Français, Deutsch, Italiano, 日本語, 한국어, 中文, Русский, العربية, हिन्दी
- File associations registered for 11 video and 8 audio extensions, each with its own per-extension icon

---

## Building from source

### Prerequisites

- **Rust** (stable, 2021 edition or later) — install via [rustup](https://rustup.rs/)
- **Node.js** 18+ and **pnpm** 8+
- **Microsoft Visual Studio Build Tools** with the *Desktop development with C++* workload (Windows only)
- **NSIS** (auto-installed by Tauri on first build)

### Native dependencies

`libmpv-2.dll` and `ffmpeg.exe` are not committed to the repository (they total ~220 MB and exceed GitHub's per-file limit). The published installer already bundles them — **only contributors building from source need to fetch them manually**.

Download both into `src-tauri/binaries/`:

| File | Source | Notes |
|---|---|---|
| `libmpv-2.dll` | [mpv-player-windows builds](https://sourceforge.net/projects/mpv-player-windows/files/libmpv/) | Pick the latest `mpv-dev-x86_64-vXX.zip`, extract `libmpv-2.dll` |
| `ffmpeg.exe` | [gyan.dev builds](https://www.gyan.dev/ffmpeg/builds/) | "essentials" build is enough; only the `ffmpeg.exe` is needed |

After placing the two files, the standard `pnpm install && pnpm tauri dev` flow works.

### Development

```bash
pnpm install
pnpm tauri dev
```

The dev build hot-reloads the frontend on save. Rust code changes trigger a full rebuild.

### Release build

```bash
pnpm tauri build
```

Outputs:
- **Standalone executable**: `src-tauri/target/release/vayou-desktop.exe` (~5 MB after LTO + strip)
- **NSIS installer**: `src-tauri/target/release/bundle/nsis/Vayou_<version>_x64-setup.exe`

The release profile uses `lto = true`, `codegen-units = 1`, `opt-level = "s"`, and `strip = true` for the smallest possible binary at the cost of compile time.

---

## Architecture

```
src/                       # SvelteKit frontend (adapter-static)
├── lib/
│   ├── bindings/          # Typed wrappers around Tauri invoke() per domain
│   ├── components/        # UI components (TitleBar, VideoControls, panels…)
│   ├── i18n/              # 13 locale files + reactive t() helper
│   ├── stores/            # Svelte 5 runes-based stores ($state classes)
│   └── utils/             # Pure helpers (time format, lang names)
└── routes/+page.svelte    # Single-route shell

src-tauri/
├── src/
│   ├── mpv/               # libmpv FFI wrapper (libloading-based)
│   ├── services/          # Pure logic: playback, tracks, settings, translate, …
│   ├── commands/          # Thin #[tauri::command] handlers — delegate to services
│   ├── state/             # MpvState + AppState (settings, current file, pending resume)
│   └── error.rs           # AppError / MpvError via thiserror
├── binaries/              # libmpv-2.dll, ffmpeg.exe (bundled)
├── icons/                 # App icons + per-extension association icons
└── installer-hooks.nsh    # NSIS hook overriding per-extension DefaultIcon
```

**State flow**: events flow upward (`mpv:*` events emitted from a dedicated event-loop thread); commands flow downward (frontend → command → service → mpv). UI never touches mpv directly.

**libmpv integration**: loaded dynamically at runtime via `libloading` from `binaries/libmpv-2.dll`. The native window's HWND is passed to mpv via the `wid` property, so mpv renders directly into the Tauri window with zero copy.

**Subtitle translation**: extracted via ffmpeg, chunked by character count, fanned out through a `tokio::sync::Semaphore` (max 8 concurrent requests) to avoid rate-limiting, then re-assembled and added to mpv as an external subtitle track. A monotonic run-id cancels stale translations when the user switches language mid-flight.

---

## Keyboard shortcuts

All shortcuts are rebindable from **Settings → Shortcuts**. Defaults:

| Action | Default |
|---|---|
| Play / pause | `Space` |
| Seek ±5 s | `← / →` |
| Seek ±30 s | `Shift + ← / →` |
| Volume ±5 | `↑ / ↓` |
| Mute | `M` |
| Fullscreen | `F` / `F11` |
| Frame step | `. / ,` |
| Speed up / down | `] / [` |
| A–B loop cycle | `L` |
| Cycle subtitles | `J` |
| Cycle audio track | `#` |
| Screenshot | `S` |
| Aspect ratio | `A` |
| Open file | `Ctrl + O` |
| Open URL | `Ctrl + U` |
| Media info | `I` |
| Next / previous in playlist | `>` / `<` |
| Zoom in / out | `* / /` (numpad) |
| Pan | `8 / 2 / 4 / 6` (numpad) |
| Reset zoom & pan | `5` (numpad) |

---

## File associations

The installer registers Vayou as a handler for the following extensions, each with a custom colored icon:

- **Video**: `.mp4` `.mkv` `.avi` `.mov` `.webm` `.flv` `.wmv` `.m4v` `.ts` `.mpg` `.mpeg`
- **Audio**: `.mp3` `.flac` `.ogg` `.wav` `.aac` `.wma` `.m4a` `.opus`

Files associated with Vayou can be opened via *Open With* in Explorer, double-click (when set as default), or by passing the path as a command-line argument.

---

## Configuration & data location

- **Settings**: `%APPDATA%\Vayou\config.json`
- **Cached translations**: `%TEMP%\vayou-translate\` (cleared by the OS periodically)
- **Logs**: stdout (set `RUST_LOG=vayou=debug` for verbose output)

---

## Third-party components

Vayou bundles or talks to the following third-party software and services. Their licenses and terms apply to the redistributed binaries / network usage and are independent of this project's MIT license:

| Component | License / Terms |
|---|---|
| **libmpv** (`libmpv-2.dll`) | LGPL-2.1-or-later. Source available at [github.com/mpv-player/mpv](https://github.com/mpv-player/mpv) |
| **FFmpeg** (`ffmpeg.exe`) | LGPL-2.1-or-later or GPL-2.0-or-later, depending on the bundled build |
| **Tauri** | MIT / Apache-2.0 |
| **OpenSubtitles** | Subject to the [OpenSubtitles terms of service](https://www.opensubtitles.org/en/terms) |
| **Google Translate (unofficial endpoint)** | Subject to Google Cloud Translation [terms](https://cloud.google.com/translate/terms). The endpoint used (`translate.googleapis.com/translate_a/single`) is undocumented and may rate-limit or block client IPs without notice. Translation is best-effort and not guaranteed to be available. |

---

## License

[MIT](LICENSE) — see the LICENSE file for the full text.
