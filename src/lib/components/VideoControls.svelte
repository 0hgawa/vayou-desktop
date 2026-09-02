<script lang="ts">
  import { player } from "$lib/stores/player.svelte";
  import { t } from "$lib/i18n/index.svelte";
  import { togglePause, setSpeed } from "$lib/bindings/playback";
  import { playlistNext, playlistPrev } from "$lib/bindings/playlist";
  import { toggleFullscreen } from "$lib/bindings/window";
  import SeekBar from "./SeekBar.svelte";
  import VolumeSlider from "./VolumeSlider.svelte";
  import SubtitlePanel from "./SubtitlePanel.svelte";
  import AudioPanel from "./AudioPanel.svelte";
  import PlaylistPanel from "./PlaylistPanel.svelte";
  import SettingsPanel from "./SettingsPanel.svelte";
  import { ICONS } from "$lib/icons";
  import { abLoop } from "$lib/stores/abLoop.svelte";
  import { formatTime } from "$lib/utils/format-time";

  let { visible = true, settingsOpen = $bindable(false) }: { visible?: boolean; settingsOpen?: boolean } = $props();
  let showRemaining = $state(false);

  const timeDisplay = $derived.by(() => {
    const cur = formatTime(player.currentTime);
    const dur = formatTime(player.duration);
    if (showRemaining && player.duration > 0) {
      const rem = Math.max(0, player.duration - player.currentTime);
      return `-${formatTime(rem)} / ${dur}`;
    }
    return `${cur} / ${dur}`;
  });

  let subPanelVisible = $state(false);
  let audioPanelVisible = $state(false);
  let playlistPanelVisible = $state(false);
  let settingsPanelVisible = $state(false);
  $effect(() => { if (settingsOpen) { closeAll(); settingsPanelVisible = true; settingsOpen = false; } });
  let speedDropOpen = $state(false);
  let speedBtnEl = $state<HTMLButtonElement | null>(null);

  const speeds = [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 3.0, 4.0];

  function closeAll() {
    subPanelVisible = false;
    audioPanelVisible = false;
    playlistPanelVisible = false;
    settingsPanelVisible = false;
    speedDropOpen = false;
  }

  function togglePanel(panel: "sub" | "audio" | "playlist" | "settings") {
    const map = { sub: () => subPanelVisible, audio: () => audioPanelVisible, playlist: () => playlistPanelVisible, settings: () => settingsPanelVisible };
    const was = map[panel]();
    closeAll();
    if (panel === "sub") subPanelVisible = !was;
    else if (panel === "audio") audioPanelVisible = !was;
    else if (panel === "playlist") playlistPanelVisible = !was;
    else settingsPanelVisible = !was;
  }

  function pickSpeed(s: number) {
    player.speed = s;
    setSpeed(s);
    speedDropOpen = false;
  }
</script>

<div class="fixed bottom-0 left-0 right-0 z-50 controls-overlay bg-gradient-to-t from-black/80 to-transparent pt-6 pb-2 px-4" class:controls-hidden={!visible}>
    {#if abLoop.enabled}
      <div class="flex items-center gap-1.5 mb-2">
        <button
          class="px-3 h-9 rounded-md text-sm font-semibold tabular-nums transition-colors {abLoop.a !== null ? 'bg-accent text-black' : 'bg-white/15 text-white hover:bg-white/25'}"
          onclick={() => abLoop.setA(player.currentTime)}
          title="Set A"
        >{abLoop.a !== null ? `A ${formatTime(abLoop.a)}` : 'A'}</button>
        <button
          class="px-3 h-9 rounded-md text-sm font-semibold tabular-nums transition-colors disabled:opacity-30 disabled:cursor-not-allowed {abLoop.b !== null ? 'bg-accent text-black' : 'bg-white/15 text-white hover:bg-white/25'}"
          onclick={() => abLoop.setB(player.currentTime)}
          disabled={abLoop.a === null}
          title="Set B"
        >{abLoop.b !== null ? `B ${formatTime(abLoop.b)}` : 'B'}</button>
        <button
          class="ctrl-btn w-9 h-9 bg-white/15 hover:bg-white/25"
          onclick={() => abLoop.clear()}
          title="Clear AB Loop"
        >
          <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24"><path d="M19 6.41L17.59 5L12 10.59L6.41 5L5 6.41L10.59 12L5 17.59L6.41 19L12 13.41L17.59 19L19 17.59L13.41 12z"/></svg>
        </button>
      </div>
    {/if}
    <div class="mb-1"><SeekBar /></div>

    <div class="flex items-center gap-1">
      <button onclick={() => { if (player.duration > 0) player.playing = !player.playing; togglePause(); }} class="ctrl-btn w-9 h-9" title={player.playing ? `${t().pause} (Space)` : `${t().play} (Space)`}>
        <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
          {#if player.playing}
            <path d="M8,19c1.1,0 2,-0.9 2,-2L10,7c0,-1.1 -0.9,-2 -2,-2s-2,0.9 -2,2v10c0,1.1 0.9,2 2,2zM14,7v10c0,1.1 0.9,2 2,2s2,-0.9 2,-2L18,7c0,-1.1 -0.9,-2 -2,-2s-2,0.9 -2,2z" />
          {:else}
            <path d="M8,6.82v10.36c0,0.79 0.87,1.27 1.54,0.84l8.14,-5.18c0.62,-0.39 0.62,-1.29 0,-1.69L9.54,5.98C8.87,5.55 8,6.03 8,6.82z" />
          {/if}
        </svg>
      </button>

      <!-- Previous -->
      <button onclick={() => playlistPrev().catch(() => {})} class="ctrl-btn w-9 h-9" title="{t().previous} (P)">
        <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
          <path d="M7,6c0.55,0 1,0.45 1,1v10c0,0.55 -0.45,1 -1,1s-1,-0.45 -1,-1L6,7c0,-0.55 0.45,-1 1,-1zM10.66,12.82l5.77,4.07c0.66,0.47 1.58,-0.01 1.58,-0.82L18.01,7.93c0,-0.81 -0.91,-1.28 -1.58,-0.82l-5.77,4.07c-0.57,0.4 -0.57,1.24 0,1.64z" />
        </svg>
      </button>

      <!-- Next -->
      <button onclick={() => playlistNext().catch(() => {})} class="ctrl-btn w-9 h-9" title="{t().next} (N)">
        <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
          <path d="M7.58,16.89l5.77,-4.07c0.56,-0.4 0.56,-1.24 0,-1.63L7.58,7.11C6.91,6.65 6,7.12 6,7.93v8.14c0,0.81 0.91,1.28 1.58,0.82zM16,7v10c0,0.55 0.45,1 1,1s1,-0.45 1,-1V7c0,-0.55 -0.45,-1 -1,-1s-1,0.45 -1,1z" />
        </svg>
      </button>

      <VolumeSlider />

      <button
        class="text-xs text-white/70 ml-2 tabular-nums hover:text-white transition-colors"
        onclick={() => (showRemaining = !showRemaining)}
        title={showRemaining ? "Show elapsed" : "Show remaining"}
      >
        {timeDisplay}
      </button>

      <div class="flex-1"></div>

      <!-- Subtitle -->
      <button onclick={() => togglePanel("sub")} class="ctrl-btn w-9 h-9" class:is-accent={subPanelVisible} title="{t().subtitles} (V)">
        <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
          <path d="M20 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 14H4V6h16v12zM6 10h2v2H6zm0 4h8v2H6zm10 0h2v2h-2zm-6-4h8v2h-8z" />
        </svg>
      </button>

      <!-- Audio -->
      <button onclick={() => togglePanel("audio")} class="ctrl-btn w-9 h-9" class:is-accent={audioPanelVisible} title="{t().audio} (A)">
        <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
          <path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55c-2.21 0-4 1.79-4 4s1.79 4 4 4s4-1.79 4-4V7h4V3h-6zm-2 16c-1.1 0-2-.9-2-2s.9-2 2-2s2 .9 2 2s-.9 2-2 2z" />
        </svg>
      </button>

      <!-- Speed -->
      <button
        bind:this={speedBtnEl}
        onclick={() => { closeAll(); speedDropOpen = !speedDropOpen; }}
        class="ctrl-btn h-9 px-2 text-sm font-medium" class:is-accent={speedDropOpen}
        title="{t().playbackSpeed}"
      >
        {player.speed}x
      </button>

      <!-- Playlist -->
      <button onclick={() => togglePanel("playlist")} class="ctrl-btn w-9 h-9" class:is-accent={playlistPanelVisible} title="{t().playlist}">
        <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
          <path d="M3 10h11v2H3zm0-4h11v2H3zm0 8h7v2H3zm13-1v8l6-4z" />
        </svg>
      </button>

      <!-- Settings -->
      <button onclick={() => togglePanel("settings")} class="ctrl-btn w-9 h-9" class:is-accent={settingsPanelVisible} title="{t().settings}">
        <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">{@html ICONS.settings}</svg>
      </button>

      <!-- Fullscreen -->
      <button onclick={() => toggleFullscreen()} class="ctrl-btn w-9 h-9" title="{t().fullscreen} (F)">
        <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
          <path d="M7 14H5v5h5v-2H7v-3zm-2-4h2V7h3V5H5v5zm12 7h-3v2h5v-5h-2v3zM14 5v2h3v3h2V5h-5z" />
        </svg>
      </button>
    </div>
  </div>

<!-- Speed dropdown -->
{#if speedDropOpen}
  <button aria-label="Close" class="fixed inset-0 z-70 w-full h-full bg-transparent border-none cursor-default" onclick={() => speedDropOpen = false}></button>
  <div
    data-panel
    class="fixed z-[71] w-[120px] bg-surface-container-high/98 backdrop-blur-md border border-white/10 rounded-md shadow-2xl py-2"
    style="bottom: {speedBtnEl ? window.innerHeight - speedBtnEl.getBoundingClientRect().top + 4 : 60}px; left: {speedBtnEl ? speedBtnEl.getBoundingClientRect().left : 0}px;"
  >
    {#each speeds as s}
      <button
        class="w-full flex items-center gap-2 px-3 py-1.5 text-[13px] hover:bg-white/10 {player.speed === s ? 'text-accent' : 'text-white/80'}"
        onclick={() => pickSpeed(s)}
      >
        <svg class="w-3.5 h-3.5 shrink-0 {player.speed === s ? 'opacity-100' : 'opacity-0'}" fill="currentColor" viewBox="0 0 24 24">{@html ICONS.check}</svg>
        <span>{s}x</span>
      </button>
    {/each}
  </div>
{/if}

<!-- Panels -->
<SubtitlePanel bind:visible={subPanelVisible} />
<AudioPanel bind:visible={audioPanelVisible} />
<PlaylistPanel bind:visible={playlistPanelVisible} />
<SettingsPanel bind:visible={settingsPanelVisible} />
