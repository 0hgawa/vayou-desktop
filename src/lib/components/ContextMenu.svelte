<script lang="ts">
  import { player } from "$lib/stores/player.svelte";
  import { t } from "$lib/i18n/index.svelte";
  import { langName } from "$lib/utils/lang-names";
  import {
    togglePause, stop, screenshot,
    toggleAbLoop, getChapters, seekChapter,
    setSpeed, type Chapter,
  } from "$lib/bindings/playback";
  import { setAlwaysOnTop } from "$lib/bindings/window";
  import { getTracks, selectSubtitle, selectAudioTrack, type TrackInfo } from "$lib/bindings/tracks";
  import { setAspectRatio, getAspectRatio } from "$lib/bindings/video";

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
  let abLoopLabel = $state("Set A");
  let currentRatio = $state("-1");

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
  async function handleAbLoop() {
    const state = await toggleAbLoop().catch(() => null);
    if (state) {
      if (state.a < 0) abLoopLabel = "Set A";
      else if (state.b < 0) abLoopLabel = "Set B";
      else abLoopLabel = "Set A";
    }
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

<svelte:window onclick={() => show && onclose()} />

{#if show}
  <div
    bind:this={menuEl}
    data-panel
    class="fixed z-[100] min-w-[200px] py-1 bg-[#1a1a1f]/98 backdrop-blur-md border border-white/10 rounded-lg shadow-2xl text-[13px] text-white/90 select-none"
    style="left:{posLeft}px;top:{posTop}px;"
    role="menu"
    tabindex="-1"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.key === "Escape" && (page === "main" ? onclose() : page = "main")}
  >
    {#if page === "main"}
      <button class="ctx-item" onclick={() => act(onopen)}>{t().openFile}<span class="ctx-key">Ctrl+O</span></button>
      <button class="ctx-item" onclick={() => act(onopenurl)}>{t().openUrl}<span class="ctx-key">Ctrl+U</span></button>
      <div class="ctx-sep"></div>
      <button class="ctx-item" onclick={() => act(() => togglePause())}>
        {player.playing ? t().pause : t().play}<span class="ctx-key">Space</span>
      </button>
      <button class="ctx-item" onclick={() => act(() => stop())}>{t().stop}</button>
      <div class="ctx-sep"></div>
      {#if subTracks.length > 0}
        <button class="ctx-item" onclick={() => page = "sub"}>{t().subtitles}<span class="ctx-arrow">▸</span></button>
      {/if}
      {#if audioTracks.length > 0}
        <button class="ctx-item" onclick={() => page = "audio"}>{t().audio}<span class="ctx-arrow">▸</span></button>
      {/if}
      <button class="ctx-item" onclick={() => page = "speed"}>{t().speed} ({player.speed}x)<span class="ctx-arrow">▸</span></button>
      {#if chapters.length > 0}
        <button class="ctx-item" onclick={() => page = "chapters"}>{t().chapters}<span class="ctx-arrow">▸</span></button>
      {/if}
      <button class="ctx-item" onclick={() => page = "aspect"}>{t().aspectRatio}<span class="ctx-arrow">▸</span></button>
      <div class="ctx-sep"></div>
      <button class="ctx-item" onclick={handleAbLoop}>{t().abLoop} ({abLoopLabel})<span class="ctx-key">L</span></button>
      <button class="ctx-item" onclick={handleScreenshot}>{t().screenshot}<span class="ctx-key">S</span></button>
      <div class="ctx-sep"></div>
      <button class="ctx-item" onclick={() => { onclose(); onpanel("info"); }}>{t().mediaInfo}<span class="ctx-key">I</span></button>
      <button class="ctx-item" onclick={() => { onclose(); onpanel("settings"); }}>{t().settings}</button>
      <div class="ctx-sep"></div>
      <button class="ctx-item" onclick={handleAlwaysOnTop}>
        {alwaysOnTop ? "✓ " : "\u00A0 "}{t().alwaysOnTop}
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
    {/if}
  </div>
{/if}

<style>
  .ctx-item {
    display: flex;
    align-items: center;
    width: 100%;
    padding: 5px 12px;
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
