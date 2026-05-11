# Changelog

All notable changes to Vayou are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] — 2026-05-04

### Changed

- Subtitle translation now uses the Chrome-extension endpoint at `clients5.google.com` first and falls back to the public `gtx` endpoint, sends a real Chrome User-Agent, and retries up to three times with backoff on 429/403. After v0.1.0 went live a number of users hit Google's rate limit on the public endpoint; the new path is what extensions like the official Translate one use and absorbs much higher request volume.
- Translation failures are now surfaced. Previously, when chunks were rate-limited, the writer silently produced an SRT containing the original text and reported success — the panel showed "translated" but the content was unchanged. The pipeline now counts failed chunks, emits a real error when every chunk fails, and warns when only some succeeded.
- Subtitle search UX: the search button is always visible (instead of being replaced by the clear button when text is present), long subtitle filenames wrap to two lines so the episode tag, resolution and release group are visible at a glance, and the panel resets when the playing video changes.
- Subtitle search now retries case-insensitively on a miss. The legacy OpenSubtitles REST endpoint matches inconsistently across case; if the original query returns zero results, the app re-runs it lowercased before giving up.

### Fixed

- Switching to a different sub track while a translation was active and clicking Translate again no longer reverts to the original source. The previous code unconditionally restored the source `sid` after removing the translation, ignoring the user's new selection. The restore now only happens as a fallback when no other sub is selected.
- Subtitle search input padding now adapts when buttons are shown, so the typed text no longer overflows behind the search/clear icons.
- ffmpeg subtitle extraction logs start and finish (with elapsed time) and times out after 30 s instead of 60 s, so a hung extract is visible in the log instead of looking like a network stall.

[0.1.1]: https://github.com/0hgawa/vayou-desktop/releases/tag/v0.1.1

## [0.1.0] — 2026-05-04

First public release.

### Added

#### Playback
- Play, pause, seek (relative and absolute), and resume position across sessions (top 20 recent files)
- Speed control from 0.25x to 4x with a quick-pick dropdown
- Frame-by-frame stepping (forward and backward)
- Screenshot capture
- A–B loop (set point A → point B → clear)
- Chapter navigation (next, previous, list)
- Open from URL (network streams)
- Sleep timer

#### Audio
- Multi-track audio with per-file persistence
- 10-band equalizer
- Loudness normalization
- Volume boost up to 200%
- Audio delay adjustment

#### Subtitles
- Multi-track embedded and external (SRT / ASS / SSA)
- Per-file selection persistence
- Customizable style: font, size, color, border color, border size, bold, vertical position
- OpenSubtitles search and download
- Automatic translation into 12 languages (Portuguese, English, Spanish, French, German, Italian, Japanese, Korean, Chinese, Russian, Arabic, Hindi), preserving ASS styling
- Subtitle delay adjustment

#### Video
- Brightness, contrast, saturation
- Aspect ratio cycling (auto, 16:9, 4:3, 21:9, 2.35:1, plus extras)
- Software deinterlace toggle
- Zoom and pan (numpad-controlled)

#### Window and UX
- Frameless transparent window with custom title bar
- Always-on-top toggle
- Drag and drop files onto the window to play
- Custom keybindings (rebindable from settings)
- 13 UI languages
- File associations registered for 11 video and 8 audio extensions, each with its own colored per-extension icon

#### Build and packaging
- NSIS installer (Windows x64), per-machine install mode
- Release profile with LTO, single codegen unit, opt-level "s", and strip — produces a ~5 MB executable

[0.1.0]: https://github.com/0hgawa/vayou-desktop/releases/tag/v0.1.0
