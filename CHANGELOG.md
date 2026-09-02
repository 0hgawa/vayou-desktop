# Changelog

All notable changes to Vayou are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.3] - 2026-09-02

### Upgrading from 0.1.2 or earlier

The update signing key was rotated in this release, and builds up to 0.1.2 carry
the previous public key compiled in. They reject anything signed by the new one
and **cannot update themselves**: the app reports the failure and offers *Open
releases page*. Download and run the installer once and automatic updates resume
from there.

### Added

- Unit tests for the pure logic that had none: subtitle time parsing (SRT and ASS), tag stripping, SRT/ASS block parsing, ASS header extraction, the A–B loop predicate, recent-file bookkeeping, and natural sort ordering. These are the deterministic, edge-case-heavy functions where a silent regression would corrupt a translated subtitle or scramble episode order without ever failing to compile.
- Vitest, and frontend tests for the logic that had none: keybinding resolution and conflict handling, track labelling, language-name mapping, and time formatting.
- A `CI` workflow running `svelte-check`, `vitest`, `cargo clippy` and `cargo test` on every push and pull request. Until now `release.yml` only fired on a tag, so nothing was checked on an ordinary commit. Clippy runs with `-D warnings`, so a new warning fails the build.
- A Clippy quality gate (`pedantic` + `nursery`) in `src-tauri/Cargo.toml`, with the allowances this codebase earns — the cast family is noise in FFI and coordinate math, and every `#[tauri::command]` takes its `State`/`AppHandle` by value because the macro hands it over that way.
- A `rust-toolchain.toml` pinning the compiler, so local builds, CI and the published installer all use the same one. Without it the `-D warnings` gate above turns every new Rust release into a red build on code nobody touched — which is exactly how the gate failed on its first CI run, with 1.95.0 locally and 1.98.0 on the runner.
- Tests for `compute_hash`, the OpenSubtitles file hash, which had none. The expected values are derivable without running the implementation: 128 KiB of zeros hashes to the file size alone, one byte set to 1 pins the sum to little-endian order, and at exactly 64 KiB the head and tail windows overlap so that byte is counted twice — which is what would catch an off-by-one in the seek-from-end arithmetic.

### Fixed

- **Subtitle translation never finished on a large film.** Collecting an embedded track means demuxing the whole container, so the time it takes scales with the size of the file and not with how much text the track holds — but the cap was a flat 30 s, justified by a comment claiming a two-hour SRT tops out around 5 s. Measured: 34 s for a 10 GB film, 134 s for a 22 GB remux. Neither could ever have succeeded. The budget is now 50 MB/s of source, with a 30 s floor so a wrong track index still fails fast and a 10 min ceiling so a pathological file still ends in an error rather than an endless spinner.
- **A finished translation was applied to whichever film was playing by then.** A 2m11s extraction easily outlives the file it was started for: one session shows the result of a 22 GB remux being attached to an episode of a different series the viewer had moved on to. The path mpv reports is now checked against the one that was translated, and a mismatch keeps the result without applying it.
- **The translation panel locked itself for the whole job.** The Translate row, the Off entry and every language carried `disabled` while a translation ran, so on a large file there were two minutes with no way to open the page, stop the job or change the language. All three are live now: Off stands the run down, and picking another language stands the old one down and starts the new one instead of dropping the choice.
- **Image-based subtitles were a dead end.** PGS is the default track on most remuxes, so an automatic translation landed on it every time and was refused — one log shows two attempts rejected before the viewer found a text track by clicking through the list. A PGS selection now falls back to a text track, preferring one in the same language, and only a file with no text subtitle at all is refused.
- **Speed up (`+`) never fired from the main keyboard row.** On most layouts `+` is Shift-`=`, so the browser reports `key: "+"` with `shiftKey: true`; the resolver folded that Shift into the combo and looked up `shift++`, which no binding can match. Only a numpad `+`, which arrives unshifted, worked. Shift is now dropped for symbol keys and kept for named keys and letters, matching the Slint build. Found by the first run of the new keybinding tests.
- **The seek bar and volume bar are now operable by keyboard and visible to screen readers.** Both were a plain `<div>` carrying `onmousedown`, with the accessibility warning suppressed: assistive technology saw no control at all, and neither did anyone navigating without a mouse. Each is now a `role="slider"` with `aria-valuenow`/`min`/`max` and a spoken `aria-valuetext` ("3:24 / 10:00", "70%"), operated with the arrows (±5s, ±30s with Shift; ±5 volume) and Home/End. The icon-only playlist and mute buttons gained an `aria-label`, and the mute tooltip is localised instead of hardcoded English.
- The subtitle search no longer loses its results a second after a file opens. `player.title` had two writers that disagreed — an observer set mpv's `media-title`, then the one-second poll overwrote it with the file name — and `SubtitlePanel` clears its OpenSubtitles results whenever the title changes. There is now a single definition of the title in the backend, reported identically by the load event and by the poll, so it cannot flip. The title also appears on load instead of up to a second later.
- Playlist ordering no longer depends on how a folder pads its episode numbers. `natural_sort_key` built its digit-run key with `format!("{:>020}", …)`, but the `0` flag is numeric-only and is silently ignored for strings — the runs were padded with spaces, and since a space sorts below every digit, a shorter run always won. `ep8.mkv` came before `ep007.mkv`, and a folder mixing `E08` with `E9` played out of order. The fill is now spelled explicitly (`{:0>20}`). Found by the first run of the new sort tests.

### Changed

- **The update signing key was rotated.** The passphrase protecting the key that
  signed 0.1.0 through 0.1.2 was lost, leaving it unable to sign anything, so a
  new pair replaces it. Signing now happens in CI from a `release` environment
  that requires a reviewer's approval before the job can reach the secrets, and
  the workflow verifies its own signature against the embedded public key before
  publishing — a mismatch fails the build instead of shipping a feed every
  client would reject. The release also carries a build provenance attestation.
- Small secondary text across the context menu, playlist, settings and subtitle
  panels moved from 11px to 12px, and the settings card narrowed to 670px.
- Settings text inputs are an underline rather than a filled box, taking the
  accent colour while focused.
- Subtitle size now defaults to 38 instead of 55, which is mpv's own documented
  default. The 55 it carried was 45% above that with nothing behind the number.
- Toolchain refresh: vite 8, TypeScript 6, SvelteKit 2.70, Svelte 5.57, Tauri
  2.11, and Node 24 in CI. Clears 25 advisories reported by `pnpm audit`, all of
  them reachable through the outdated SvelteKit.
- The track list is read in one call instead of 1 + 8×N. mpv serialises node-typed properties as JSON when they are read as a string, so the whole list arrives in a single FFI round-trip; reading it field by field cost 161 crossings on a 20-track file, each allocating a `CString` going in and a `String` coming back, and the subtitle panel refreshes on open, on every track selection and after every translation.
- A translation is reused instead of being recomputed. The result is deterministic for a given video, source track and language, and removing then re-adding a translation is routine, so the expensive path used to run several times per film. A hit now skips straight to loading the file, which also means the work is not lost when a run is cancelled or its film is switched away — the file is written before either check.
- An embedded ASS track is extracted in one ffmpeg pass rather than two. The header and the dialogue come out of the same stream, and asking for them separately demuxed the container twice to read identical bytes.
- ffmpeg runs at below-normal priority. An extraction reads the container as fast as the disk allows while mpv decodes from that same disk; on a 22 GB remux the two competed and playback stuttered for the whole run.
- The translation progress event names its phase, so the panel reads "Extracting subtitles…" and then "Translating 40%". Extraction has no percentage to report and is the long half, so the previous display was a spinner beside a "0%" that never moved.
- `dev.bat` defaults `RUST_LOG` to `vayou=debug`. At `info` the log showed startup and little else, which is the wrong default for the script whose purpose is diagnosing a running app.
- The build pins its compiler through `rust-toolchain.toml`, so local builds, CI and the published installer use the same one, and `-D warnings` cannot be tripped by a Rust release nobody asked for.
- `ab_loop::check` now delegates its range and look-ahead rules to a pure `should_loop` predicate. Same behaviour — the split exists so those rules can be tested without the atomics or the wall clock.
- Builds with Rust 1.98. The newer Clippy had exactly two complaints about this tree, both the same lint, and acting on them removed code rather than adding it: `as_chunks::<8>()` yields `[u8; 8]` directly, so the two `try_into().unwrap()` calls in the OpenSubtitles hash — on a slice that could only ever be eight bytes long — are gone, with the length now proven at compile time. Verified clean under both 1.95.0 and 1.98.0 before the pin was raised.
- The `media-title` property is no longer observed. It changed several times per file as mpv parsed metadata and every one of those was overwritten within a second, so nothing ever consumed it.

### Removed

- Dead bindings with no callers: `play`, `stop`, `openUrl`, `toggleSubtitles`
  and `getLocale`. The Tauri commands behind them stay registered — dropping
  those removes backend capability rather than dead weight.

## [0.1.2] — 2026-07-04

### Added

- Subtitle drop-shadow control (offset plus a black shadow colour) in the subtitle style panel.

### Changed

- Hybrid seek: dragging the seek bar seeks by keyframe (fast, responsive) and the release seeks exactly, so scrubbing stays fluid without giving up frame accuracy at the landing point.
- The volume OSD is now driven by a tick bumped only on user volume and mute actions, instead of reacting to the `volume` property itself. mpv echoes the volume on every file load, which used to pop the OSD on screen for a change the user never made.
- Reset and restore actions confirm in place — the button briefly swaps its icon for a check (800 ms) — instead of raising a toast. The subtitle Style header got a circular reset icon to match.
- Translation errors are now shown in full: the toast carries the whole message, is dismissable, and is localized, rather than being truncated English text.
- The settings panel has a modest minimum height so the sparser tabs are not cramped.
- The NSIS installer switched to a per-user install (`currentUser`, `%LOCALAPPDATA%`). The in-app updater replaces the running executable in place, which needs a writable install directory — a per-machine install required elevation the updater cannot request.

### Removed

- The software deinterlace toggle. Cycling it rebuilt the hardware-decode pipeline mid-playback and froze the video; there is no safe way to toggle it live, so the control is gone rather than shipping a switch that breaks playback.
- Linux and macOS targets, and the release matrix that built them. The Linux build embedded mpv through an X11 Window ID, which forced `GDK_BACKEND=x11` and refused to start on a native Wayland session; native Wayland needs `mpv_render_context`, which this build does not have. Windows-only for now.

[0.1.2]: https://github.com/0hgawa/vayou-desktop/releases/tag/v0.1.2

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
