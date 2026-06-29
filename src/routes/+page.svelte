<script lang="ts">
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { player } from "$lib/stores/player.svelte";
  import { settings } from "$lib/stores/settings.svelte";
  import { t } from "$lib/i18n/index.svelte";
  import {
    initPlayer, openFile, togglePause, seekRelative, setVolume, setSpeed, getPlaybackState,
    screenshot, frameStep, frameBackStep,
  } from "$lib/bindings/playback";
  import { setAspectRatio, getAspectRatio, setVideoZoom, setVideoPan, getVideoZoomPan, resetVideoZoomPan } from "$lib/bindings/video";
  import { toggleFullscreen } from "$lib/bindings/window";
  import { getTracks, selectSubtitle, selectAudioTrack } from "$lib/bindings/tracks";
  import { keybindings } from "$lib/stores/keybindings.svelte";
  import { abLoop } from "$lib/stores/abLoop.svelte";
  import { translate } from "$lib/stores/translate.svelte";
  import { playlistNext, playlistPrev } from "$lib/bindings/playlist";
  import TitleBar from "$lib/components/TitleBar.svelte";
  import VideoControls from "$lib/components/VideoControls.svelte";
  import ContextMenu from "$lib/components/ContextMenu.svelte";
  import MediaInfoPanel from "$lib/components/MediaInfoPanel.svelte";
  import Toast from "$lib/components/Toast.svelte";
  import VolumeOsd from "$lib/components/VolumeOsd.svelte";
  import { toast } from "$lib/stores/toast.svelte";

  let fileLoaded = $state(false);
  let dragOver = $state(false);
  let ctxShow = $state(false);
  let ctxX = $state(0);
  let ctxY = $state(0);
  let infoPanel = $state(false);
  let settingsOpen = $state(false);

  function openPanel(name: string) {
    if (name === "info") infoPanel = true;
    if (name === "settings") settingsOpen = true;
  }

  let cursorVisible = $state(true);
  let cursorTimer: ReturnType<typeof setTimeout> | null = null;
  /** Grace period before hiding the controls when the mouse leaves the
   * window in windowed mode. Long enough that briefly nudging past the
   * window edge doesn't trigger a hide. */
  const WINDOWED_HIDE_DELAY_MS = 3000;
  let windowedHideTimer: ReturnType<typeof setTimeout> | null = null;

  function cancelWindowedHide() {
    if (windowedHideTimer) { clearTimeout(windowedHideTimer); windowedHideTimer = null; }
  }

  function handleMouseMove(e: MouseEvent) {
    cursorVisible = true;
    if (cursorTimer) clearTimeout(cursorTimer);
    cancelWindowedHide();

    if (!player.fullscreen) { player.controlsVisible = true; return; }
    if (document.querySelector("[data-panel]")) { player.controlsVisible = true; return; }
    // Keep visible while the pointer is over a control bar (covers the taller
    // bar when the A-B loop row is shown), otherwise use the top/bottom reveal zones.
    const overControls = !!(e.target as Element | null)?.closest?.(".controls-overlay");
    player.controlsVisible = overControls || e.clientY <= 50 || e.clientY >= window.innerHeight - 80;
    if (player.playing) cursorTimer = setTimeout(() => { cursorVisible = false; }, 500);
  }

  function handleMouseLeave() {
    if (player.fullscreen) return;
    if (!player.playing) return;
    if (document.querySelector("[data-panel]")) return;
    cancelWindowedHide();
    windowedHideTimer = setTimeout(() => {
      // Re-check at fire time — user may have paused or opened a panel
      // during the grace period.
      if (player.playing && !document.querySelector("[data-panel]")) {
        player.controlsVisible = false;
      }
    }, WINDOWED_HIDE_DELAY_MS);
  }
  function handleMouseEnter() {
    cancelWindowedHide();
    if (!player.fullscreen) player.controlsVisible = true;
  }

  $effect(() => {
    if (!player.fullscreen) player.controlsVisible = true;
  });

  /** Auto-translate when a new file loads. Lives here (not in SubtitlePanel)
   * because the panel may be unmounted when the user opens a new video — the
   * effect would never fire and the translation would silently not start.
   *
   * Triggers off `fileEpoch` (incremented on `mpv:file-loaded`) instead of
   * `player.title` because mpv emits `media-title` multiple times per load
   * — once with the filename, again when metadata is parsed — which used
   * to fire the effect twice for the same file. */
  let lastTranslatedEpoch = -1;
  $effect(() => {
    const epoch = player.fileEpoch;
    if (epoch === 0 || epoch === lastTranslatedEpoch) return;
    translate.translationTrackPath = null;
    if (settings.translateLang === "off") return;
    // Wait briefly for mpv to register the embedded sub tracks before
    // querying — otherwise the backend lookup finds no selected sub.
    const timer = setTimeout(async () => {
      if (player.fileEpoch !== epoch) return;
      const tracks = await getTracks().catch(() => []);
      if (tracks.some((t) => t.track_type === "sub" && t.selected)) {
        lastTranslatedEpoch = epoch;
        translate.translate();
      }
    }, 800);
    return () => clearTimeout(timer);
  });

  $effect(() => {
    const cleanups: Array<Promise<() => void> | (() => void)> = [];

    initPlayer().catch(() => {});
    settings.load();
    getCurrentWebviewWindow().show();

    // Event-driven state updates
    cleanups.push(listen<number>("mpv:time-pos", (e) => { player.currentTime = e.payload; }));
    cleanups.push(listen<number>("mpv:duration", (e) => { player.duration = e.payload; }));
    cleanups.push(listen<boolean>("mpv:pause", (e) => { if (player.duration > 0) player.playing = !e.payload; }));
    cleanups.push(listen<number>("mpv:volume", (e) => { player.volume = e.payload; }));
    cleanups.push(listen<string>("mpv:media-title", (e) => { player.title = e.payload; }));
    cleanups.push(listen<void>("mpv:end-file", () => { player.playing = false; }));
    cleanups.push(listen<void>("mpv:file-loaded", () => {
      settings.applySubStyle();
      abLoop.reset();
      player.fileEpoch++;
    }));

    // Open files from CLI args ("Open with" from Explorer)
    cleanups.push(listen<string[]>("open-files", (e) => {
      if (e.payload.length > 0) { fileLoaded = true; openFile(e.payload[0]); }
    }));

    // Polling fallback
    const poll = setInterval(() => {
      getPlaybackState().then((s) => {
        player.currentTime = s.time_pos;
        player.duration = s.duration;
        player.playing = s.duration > 0 && !s.paused;
        player.title = s.title;
        player.volume = s.volume;
      }).catch(() => {});
    }, 1000);
    cleanups.push(() => clearInterval(poll));

    // Drag & drop
    getCurrentWebviewWindow().onDragDropEvent((event) => {
      if (event.payload.type === "enter" || event.payload.type === "over") {
        dragOver = true;
      } else if (event.payload.type === "leave") {
        dragOver = false;
      } else if (event.payload.type === "drop") {
        dragOver = false;
        const paths = event.payload.paths;
        if (paths.length > 0) {
          fileLoaded = true;
          openFile(paths[0]);
        }
      }
    }).then((fn) => cleanups.push(fn));

    return () => {
      for (const c of cleanups) {
        if (typeof c === "function") c();
        else c.then((fn) => fn());
      }
    };
  });

  const ratioList = ["-1", "16:9", "4:3", "21:9", "2.35:1"];
  async function cycleRatio() {
    try {
      const current = await getAspectRatio();
      const idx = ratioList.indexOf(current);
      const next = ratioList[(idx + 1) % ratioList.length];
      await setAspectRatio(next);
    } catch {}
  }

  async function cycleTrack(type: "sub" | "audio") {
    try {
      const tracks = await getTracks();
      const filtered = tracks.filter((t) => t.track_type === type);
      if (filtered.length === 0) return;
      const current = filtered.find((t) => t.selected);
      const idx = current ? filtered.indexOf(current) : -1;
      const next = filtered[(idx + 1) % filtered.length];
      if (type === "sub") selectSubtitle(next.id);
      else selectAudioTrack(next.id);
    } catch {}
  }

  const actionHandlers: Record<string, () => void> = {
    togglePause: () => { if (player.duration > 0) player.playing = !player.playing; togglePause(); },
    seekForward: () => seekRelative(5),
    seekForwardLong: () => seekRelative(30),
    seekBack: () => seekRelative(-5),
    seekBackLong: () => seekRelative(-30),
    nextFile: () => playlistNext().catch(() => {}),
    prevFile: () => playlistPrev().catch(() => {}),
    frameNext: () => frameStep().catch(() => {}),
    framePrev: () => frameBackStep().catch(() => {}),
    speedUp: () => { player.speed = Math.min(4, +(player.speed + 0.25).toFixed(2)); setSpeed(player.speed); },
    speedDown: () => { player.speed = Math.max(0.25, +(player.speed - 0.25).toFixed(2)); setSpeed(player.speed); },
    abLoop: () => abLoop.cycle(),
    volumeUp: () => { player.volume = Math.min(100, player.volume + 5); setVolume(player.volume); },
    volumeDown: () => { player.volume = Math.max(0, player.volume - 5); setVolume(player.volume); },
    mute: () => { player.muted = !player.muted; setVolume(player.muted ? 0 : player.volume); },
    fullscreen: () => toggleFullscreen(),
    screenshot: () => screenshot().then(() => toast.show(t().screenshotSaved)).catch(() => {}),
    aspectRatio: () => cycleRatio(),
    cycleSub: () => cycleTrack("sub"),
    cycleAudio: () => cycleTrack("audio"),
    openFile: () => handleOpenFile(),
    openUrl: () => handleOpenUrl(),
    mediaInfo: () => openPanel("info"),
  };

  function handleKeyDown(e: KeyboardEvent) {
    if (e.target instanceof HTMLInputElement) return;

    // Numpad pan & scan (not rebindable)
    if (e.location === 3) {
      switch (e.key) {
        case "8": getVideoZoomPan().then(s => setVideoPan(s.pan_x, s.pan_y - 0.02)); return;
        case "2": getVideoZoomPan().then(s => setVideoPan(s.pan_x, s.pan_y + 0.02)); return;
        case "4": getVideoZoomPan().then(s => setVideoPan(s.pan_x + 0.02, s.pan_y)); return;
        case "6": getVideoZoomPan().then(s => setVideoPan(s.pan_x - 0.02, s.pan_y)); return;
        case "5": resetVideoZoomPan(); return;
      }
    }
    if (e.key === "*") { getVideoZoomPan().then(s => setVideoZoom(s.zoom + 0.1)); return; }
    if (e.key === "/") { getVideoZoomPan().then(s => setVideoZoom(s.zoom - 0.1)); return; }
    if (e.key === "Escape") { if (infoPanel) { infoPanel = false; } else if (player.fullscreen) toggleFullscreen(); return; }
    if (e.key === "F11") { e.preventDefault(); toggleFullscreen(); return; }

    const action = keybindings.resolve(e);
    if (action && actionHandlers[action]) {
      e.preventDefault();
      actionHandlers[action]();
    }
  }

  function handleOpenUrl() {
    const url = prompt("URL:");
    if (url?.trim()) { fileLoaded = true; openFile(url.trim()); }
  }

  async function handleOpenFile() {
    const selected = await open({
      multiple: false,
      filters: [
        { name: "Video", extensions: ["mp4","mkv","avi","mov","wmv","flv","webm","mpg","mpeg","m4v","3gp","ts","vob"] },
        { name: "Audio", extensions: ["mp3","flac","wav","ogg","m4a","aac","opus","wma"] },
        { name: "All", extensions: ["*"] },
      ],
    });
    if (selected) {
      fileLoaded = true;
      openFile(selected as string);
    }
  }

  function handleContextMenu(e: MouseEvent) {
    e.preventDefault();
    if ((e.target as HTMLElement).closest("[data-panel]")) return;
    ctxX = e.clientX;
    ctxY = e.clientY;
    ctxShow = true;
    player.controlsVisible = true;
  }

  function handleDoubleClick(e: MouseEvent) {
    const el = e.target as HTMLElement;
    // Only toggle fullscreen when double-clicking the video area itself
    if (el.closest(".controls-overlay") || el.closest("[data-panel]")) return;
    toggleFullscreen();
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="w-screen h-screen relative overflow-hidden"
  style="background: {player.duration > 0 ? 'transparent' : 'black'}; cursor: {!player.fullscreen || cursorVisible ? 'default' : 'none'};"
  onmousemove={handleMouseMove}
  onmouseleave={handleMouseLeave}
  onmouseenter={handleMouseEnter}
  ondblclick={handleDoubleClick}
  oncontextmenu={handleContextMenu}
  onwheel={(e) => { if ((e.target as HTMLElement).closest("[data-panel]")) return; e.preventDefault(); const max = settings.volumeBoost ? 200 : 100; player.volume = Math.min(max, Math.max(0, player.volume + (e.deltaY < 0 ? 5 : -5))); setVolume(player.volume); }}
>
  {#if dragOver}
    <div class="absolute inset-0 z-90 flex items-center justify-center bg-black/60 border-2 border-dashed border-white/30 pointer-events-none">
      <p class="text-white/60 text-sm">{t().dropToPlay}</p>
    </div>
  {/if}

  {#if !fileLoaded && player.duration === 0}
    <div class="absolute inset-0 flex flex-col items-center justify-center text-white/40">
      <svg class="w-20 h-20 mb-4" fill="currentColor" viewBox="0 0 24 24">
        <path d="M10 16.5l6-4.5l-6-4.5zM12 2C6.48 2 2 6.48 2 12s4.48 10 10 10s10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8s8 3.59 8 8s-3.59 8-8 8z" />
      </svg>
      <p class="text-sm mb-1">{t().welcomeTitle}</p>
      <p class="text-xs text-white/25">
        {t().welcomeSubtitle} <kbd class="px-1.5 py-0.5 bg-white/10 rounded-md text-white/40">Ctrl+O</kbd> {t().toOpen}
      </p>
    </div>
  {/if}

  <TitleBar visible={player.controlsVisible} menuOpen={ctxShow} onmenu={(mx, my) => { if (mx < 0) { ctxShow = false; } else { ctxX = mx; ctxY = my; ctxShow = true; } }} />
  <VideoControls visible={player.controlsVisible} bind:settingsOpen />

  <ContextMenu
    show={ctxShow}
    x={ctxX}
    y={ctxY}
    onclose={() => ctxShow = false}
    onopen={handleOpenFile}
    onopenurl={handleOpenUrl}
    onpanel={openPanel}
  />

  <MediaInfoPanel bind:visible={infoPanel} />
  <VolumeOsd />
  <Toast />
</div>
