<script lang="ts">
  import { player } from "$lib/stores/player.svelte";
  import { t } from "$lib/i18n/index.svelte";
  import { langName } from "$lib/utils/lang-names";
  import {
    togglePause, stop, screenshot,
    getChapters, seekChapter,
    setSpeed, type Chapter,
  } from "$lib/bindings/playback";
  import { setAlwaysOnTop } from "$lib/bindings/window";
  import { getTracks, selectSubtitle, selectAudioTrack, type TrackInfo } from "$lib/bindings/tracks";
  import { setAspectRatio, getAspectRatio } from "$lib/bindings/video";
  import { sleepTimer } from "$lib/stores/sleepTimer.svelte";
  import { abLoop } from "$lib/stores/abLoop.svelte";
  import { ICONS } from "$lib/icons";

  let {
    show = false,
    x = 0,
    y = 0,
    onclose,
    onopen,
    onopenurl,
    onpanel,
  }: {
    show: boolean;
    x: number;
    y: number;
    onclose: () => void;
    onopen: () => void;
    onopenurl: () => void;
    onpanel: (name: string) => void;
  } = $props();

  let subTracks = $state<TrackInfo[]>([]);
  let audioTracks = $state<TrackInfo[]>([]);
  let chapters = $state<Chapter[]>([]);
  let page = $state("main");
  let alwaysOnTop = $state(false);
  let currentRatio = $state("-1");

  const abLoopLabel = $derived(
    !abLoop.enabled ? "" :
    abLoop.a === null ? "On" :
    abLoop.b === null ? "A" : "A • B"
  );

  const speeds = [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 3.0, 4.0];
  const ratios: [string, string][] = [
    ["-1", "Auto"], ["16:9", "16:9"], ["4:3", "4:3"], ["21:9", "21:9"],
    ["16:10", "16:10"], ["5:4", "5:4"], ["1:1", "1:1"], ["2.35:1", "2.35:1"], ["2.39:1", "2.39:1"],
  ];

  $effect(() => {
    if (show) {
      page = "main";
      getTracks().then((tracks) => {
        subTracks = tracks.filter((t) => t.track_type === "sub");
        audioTracks = tracks.filter((t) => t.track_type === "audio");
      }).catch(() => {});
      getChapters().then((c) => { chapters = c; }).catch(() => {});
      getAspectRatio().then((r) => { currentRatio = r; }).catch(() => {});
    }
  });

  function act(fn: () => void) { fn(); onclose(); }

  async function handleScreenshot() { await screenshot().catch(() => {}); onclose(); }
  function handleAbLoop() {
    if (abLoop.enabled) abLoop.clear();
    else abLoop.enable();
    onclose();
  }
  async function handleAlwaysOnTop() { alwaysOnTop = !alwaysOnTop; await setAlwaysOnTop(alwaysOnTop); onclose(); }

  function formatTime(s: number): string {
    const h = Math.floor(s / 3600);
    const m = Math.floor((s % 3600) / 60);
    const sec = Math.floor(s % 60);
    const pad = (n: number) => n.toString().padStart(2, "0");
    return h > 0 ? `${h}:${pad(m)}:${pad(sec)}` : `${m}:${pad(sec)}`;
  }

  let menuEl = $state<HTMLDivElement | null>(null);
  let posLeft = $state(0);
  let posTop = $state(0);

  $effect(() => {
    if (!show || !menuEl) return;
    // Also react to page changes so it repositions on navigate
    page;
    // Wait one frame for the DOM to render the new content
    requestAnimationFrame(() => {
      if (!menuEl) return;
      const rect = menuEl.getBoundingClientRect();
      const vw = window.innerWidth;
      const vh = window.innerHeight;
      const pad = 8;

      // Flip: if overflows right, place left of cursor; else right of cursor
      let left = x + rect.width > vw ? x - rect.width : x;
      // Flip: if overflows bottom, place above cursor; else below cursor
      let top = y + rect.height > vh ? y - rect.height : y;

      // Shift: clamp so it never goes off-screen
      left = Math.max(pad, Math.min(left, vw - rect.width - pad));
      top = Math.max(pad, Math.min(top, vh - rect.height - pad));

      posLeft = left;
      posTop = top;
    });
  });
</script>

{#snippet ctxIcon(svg: string, active: boolean = false)}
  <svg class="w-4 h-4 mr-2.5 shrink-0 {active ? 'text-accent' : 'text-white/55'}" fill="currentColor" viewBox="0 0 24 24">{@html svg}</svg>
{/snippet}

<svelte:window onclick={() => show && onclose()} />

{#if show}
  <div
    bind:this={menuEl}
    data-panel
    class="fixed z-[100] min-w-[200px] max-w-[360px] py-2 bg-surface-container-high/98 backdrop-blur-md border border-white/10 rounded-md shadow-2xl text-[13px] text-white/90 select-none overflow-y-auto overflow-x-hidden"
    style="left:{posLeft}px;top:{posTop}px;max-height:calc(100vh - 16px);"
    role="menu"
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.key === "Escape" && (page === "main" ? onclose() : page = "main")}
  >
    {#if page === "main"}
      <button class="ctx-item" onclick={() => act(onopen)}>{@render ctxIcon(ICONS.folderOpen)}{t().openFile}<span class="ctx-key">Ctrl+O</span></button>
      <button class="ctx-item" onclick={() => act(onopenurl)}>{@render ctxIcon(ICONS.link)}{t().openUrl}<span class="ctx-key">Ctrl+U</span></button>
      <div class="ctx-sep"></div>
      <button class="ctx-item" onclick={() => act(() => togglePause())}>
        {@render ctxIcon(player.playing ? ICONS.pause : ICONS.play)}{player.playing ? t().pause : t().play}<span class="ctx-key">Space</span>
      </button>
      <button class="ctx-item" onclick={() => act(() => stop())}>{@render ctxIcon(ICONS.stop)}{t().stop}</button>
      <div class="ctx-sep"></div>
      {#if subTracks.length > 0}
        <button class="ctx-item" onclick={() => page = "sub"}>{@render ctxIcon(ICONS.subtitles)}{t().subtitles}<span class="ctx-arrow">▸</span></button>
      {/if}
      {#if audioTracks.length > 0}
        <button class="ctx-item" onclick={() => page = "audio"}>{@render ctxIcon(ICONS.volumeUp)}{t().audio}<span class="ctx-arrow">▸</span></button>
      {/if}
      <button class="ctx-item" onclick={() => page = "speed"}>{@render ctxIcon(ICONS.speed)}{t().speed} ({player.speed}x)<span class="ctx-arrow">▸</span></button>
      {#if chapters.length > 0}
        <button class="ctx-item" onclick={() => page = "chapters"}>{@render ctxIcon(ICONS.segment)}{t().chapters}<span class="ctx-arrow">▸</span></button>
      {/if}
      <button class="ctx-item" onclick={() => page = "aspect"}>{@render ctxIcon(ICONS.aspectRatio)}{t().aspectRatio}<span class="ctx-arrow">▸</span></button>
      <div class="ctx-sep"></div>
      <button class="ctx-item" onclick={handleAbLoop}>
        {@render ctxIcon(ICONS.repeat, abLoop.enabled)}{t().abLoop}{#if abLoopLabel}<span class="text-accent text-[11px] ml-2 tabular-nums">{abLoopLabel}</span>{/if}
        <span class="ctx-key">L</span>
      </button>
      <button class="ctx-item" onclick={handleScreenshot}>{@render ctxIcon(ICONS.camera)}{t().screenshot}<span class="ctx-key">S</span></button>
      <div class="ctx-sep"></div>
      <button class="ctx-item" onclick={() => { onclose(); onpanel("info"); }}>{@render ctxIcon(ICONS.info)}{t().mediaInfo}<span class="ctx-key">I</span></button>
      <button class="ctx-item" onclick={() => { onclose(); onpanel("settings"); }}>{@render ctxIcon(ICONS.settings)}{t().settings}</button>
      <div class="ctx-sep"></div>
      <button class="ctx-item" onclick={() => page = "sleep"}>
        {@render ctxIcon(ICONS.timer, sleepTimer.formatted !== null)}{t().sleepTimer}
        {#if sleepTimer.formatted}
          <span class="ctx-key text-accent tabular-nums">{sleepTimer.formatted}</span>
        {:else}
          <span class="ctx-arrow">▸</span>
        {/if}
      </button>
      <button class="ctx-item" onclick={handleAlwaysOnTop}>
        {@render ctxIcon(ICONS.pushPin, alwaysOnTop)}{t().alwaysOnTop}
      </button>

    {:else if page === "sub"}
      <button class="ctx-back" onclick={() => page = "main"}>← {t().subtitles}</button>
      <div class="ctx-sep"></div>
      <button class="ctx-item" onclick={() => { selectSubtitle(0); onclose(); }}>
        {subTracks.every((st) => !st.selected) ? "✓ " : "\u00A0 "}{t().off}
      </button>
      {#each subTracks as t}
        <button class="ctx-item" onclick={() => { selectSubtitle(t.id); onclose(); }}>
          {t.selected ? "✓ " : "\u00A0 "}{t.title || langName(t.lang) || `Track ${t.id}`}
        </button>
      {/each}

    {:else if page === "audio"}
      <button class="ctx-back" onclick={() => page = "main"}>← {t().audio}</button>
      <div class="ctx-sep"></div>
      {#each audioTracks as t}
        <button class="ctx-item" onclick={() => { selectAudioTrack(t.id); onclose(); }}>
          {t.selected ? "✓ " : "\u00A0 "}{t.title || langName(t.lang) || `Track ${t.id}`}
        </button>
      {/each}

    {:else if page === "speed"}
      <button class="ctx-back" onclick={() => page = "main"}>← {t().speed}</button>
      <div class="ctx-sep"></div>
      {#each speeds as s}
        <button class="ctx-item" onclick={() => { player.speed = s; setSpeed(s); onclose(); }}>
          {player.speed === s ? "✓ " : "\u00A0 "}{s}x
        </button>
      {/each}

    {:else if page === "chapters"}
      <button class="ctx-back" onclick={() => page = "main"}>← {t().chapters}</button>
      <div class="ctx-sep"></div>
      <div class="max-h-[300px] overflow-y-auto">
        {#each chapters as ch}
          <button class="ctx-item" onclick={() => { seekChapter(ch.index); onclose(); }}>
            {ch.current ? "▶ " : "\u00A0 "}{ch.title}
            <span class="ctx-key">{formatTime(ch.time)}</span>
          </button>
        {/each}
      </div>

    {:else if page === "aspect"}
      <button class="ctx-back" onclick={() => page = "main"}>← {t().aspectRatio}</button>
      <div class="ctx-sep"></div>
      {#each ratios as [value, label]}
        <button class="ctx-item" onclick={() => { setAspectRatio(value); currentRatio = value; onclose(); }}>
          {currentRatio === value ? "✓ " : "\u00A0 "}{label}
        </button>
      {/each}

    {:else if page === "sleep"}
      <button class="ctx-back" onclick={() => page = "main"}>← {t().sleepTimer}</button>
      <div class="ctx-sep"></div>
      {#if sleepTimer.formatted}
        <div class="px-3 py-2 text-center">
          <div class="text-accent text-lg font-semibold tabular-nums">{sleepTimer.formatted}</div>
        </div>
        <button class="ctx-item" onclick={() => { sleepTimer.cancel(); onclose(); }}>
          ✕ {t().cancel}
        </button>
        <div class="ctx-sep"></div>
      {/if}
      {#each [5, 10, 15, 20, 30, 45, 60, 90] as min}
        <button class="ctx-item" onclick={() => { sleepTimer.setTimer(min); onclose(); }}>
          {sleepTimer.activeMinutes === min ? "✓ " : "\u00A0 "}{min} min
        </button>
      {/each}
    {/if}
  </div>
{/if}

<style>
  .ctx-item {
    display: flex;
    align-items: center;
    width: 100%;
    padding: 7px 14px;
    text-align: left;
    background: none;
    border: none;
    color: rgba(255, 255, 255, 0.9);
    font-size: 13px;
    white-space: nowrap;
  }
  .ctx-item:hover { background: rgba(255, 255, 255, 0.1); }
  .ctx-back {
    display: flex;
    align-items: center;
    width: 100%;
    padding: 5px 12px;
    text-align: left;
    background: none;
    border: none;
    color: rgba(255, 255, 255, 0.5);
    font-size: 13px;
    white-space: nowrap;
  }
  .ctx-back:hover { background: rgba(255, 255, 255, 0.1); color: rgba(255, 255, 255, 0.9); }
  .ctx-key {
    margin-left: auto;
    padding-left: 20px;
    color: rgba(255, 255, 255, 0.35);
    font-size: 12px;
  }
  .ctx-arrow {
    margin-left: auto;
    padding-left: 12px;
    color: rgba(255, 255, 255, 0.35);
  }
  .ctx-sep {
    height: 1px;
    margin: 4px 8px;
    background: rgba(255, 255, 255, 0.08);
  }
</style>
