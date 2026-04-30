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

  let { visible = true, settingsOpen = $bindable(false) }: { visible?: boolean; settingsOpen?: boolean } = $props();

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

      <span class="text-xs text-white/70 ml-2 tabular-nums">
        {player.formattedTime}
      </span>

      <div class="flex-1"></div>

      <!-- Subtitle -->
      <button onclick={() => togglePanel("sub")} class="ctrl-btn w-9 h-9 {subPanelVisible ? 'text-accent' : ''}" title="{t().subtitles} (V)">
        <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
          <path d="M20 4H4c-1.1 0-2 .9-2 2v12c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 14H4V6h16v12zM6 10h2v2H6zm0 4h8v2H6zm10 0h2v2h-2zm-6-4h8v2h-8z" />
        </svg>
      </button>

      <!-- Audio -->
      <button onclick={() => togglePanel("audio")} class="ctrl-btn w-9 h-9 {audioPanelVisible ? 'text-accent' : ''}" title="{t().audio} (A)">
        <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
          <path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55c-2.21 0-4 1.79-4 4s1.79 4 4 4s4-1.79 4-4V7h4V3h-6zm-2 16c-1.1 0-2-.9-2-2s.9-2 2-2s2 .9 2 2s-.9 2-2 2z" />
        </svg>
      </button>

      <!-- Speed -->
      <button
        bind:this={speedBtnEl}
        onclick={() => { closeAll(); speedDropOpen = !speedDropOpen; }}
        class="ctrl-btn h-9 px-2 text-sm font-medium {speedDropOpen ? 'text-accent' : ''}"
        title="{t().playbackSpeed}"
      >
        {player.speed}x
      </button>

      <!-- Playlist -->
      <button onclick={() => togglePanel("playlist")} class="ctrl-btn w-9 h-9 {playlistPanelVisible ? 'text-accent' : ''}" title="{t().playlist}">
        <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
          <path d="M3 10h11v2H3zm0-4h11v2H3zm0 8h7v2H3zm13-1v8l6-4z" />
        </svg>
      </button>

      <!-- Settings -->
      <button onclick={() => togglePanel("settings")} class="ctrl-btn w-9 h-9 {settingsPanelVisible ? 'text-accent' : ''}" title="{t().settings}">
        <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
          <path d="M19.43 12.98c.04-.32.07-.64.07-.98c0-.34-.03-.66-.07-.98l2.11-1.65c.19-.15.24-.42.12-.64l-2-3.46a.5.5 0 0 0-.61-.22l-2.49 1c-.52-.4-1.08-.73-1.69-.98l-.38-2.65A.488.488 0 0 0 14 2h-4c-.25 0-.46.18-.49.42l-.38 2.65c-.61.25-1.17.59-1.69.98l-2.49-1a.566.566 0 0 0-.18-.03c-.17 0-.34.09-.43.25l-2 3.46c-.13.22-.07.49.12.64l2.11 1.65c-.04.32-.07.65-.07.98c0 .33.03.66.07.98l-2.11 1.65c-.19.15-.24.42-.12.64l2 3.46a.5.5 0 0 0 .61.22l2.49-1c.52.4 1.08.73 1.69.98l.38 2.65c.03.24.24.42.49.42h4c.25 0 .46-.18.49-.42l.38-2.65c.61-.25 1.17-.59 1.69-.98l2.49 1c.06.02.12.03.18.03c.17 0 .34-.09.43-.25l2-3.46c.12-.22.07-.49-.12-.64l-2.11-1.65zm-1.98-1.71c.04.31.05.52.05.73c0 .21-.02.43-.05.73l-.14 1.13l.89.7l1.08.84l-.7 1.21l-1.27-.51l-1.04-.42l-.9.68c-.43.32-.84.56-1.25.73l-1.06.43l-.16 1.13l-.2 1.35h-1.4l-.19-1.35l-.16-1.13l-1.06-.43c-.43-.18-.83-.41-1.23-.71l-.91-.7l-1.06.43l-1.27.51l-.7-1.21l1.08-.84l.89-.7l-.14-1.13c-.03-.31-.05-.54-.05-.74s.02-.43.05-.73l.14-1.13l-.89-.7l-1.08-.84l.7-1.21l1.27.51l1.04.42l.9-.68c.43-.32.84-.56 1.25-.73l1.06-.43l.16-1.13l.2-1.35h1.39l.19 1.35l.16 1.13l1.06.43c.43.18.83.41 1.23.71l.91.7l1.06-.43l1.27-.51l.7 1.21l-1.07.85l-.89.7l.14 1.13zM12 8c-2.21 0-4 1.79-4 4s1.79 4 4 4s4-1.79 4-4s-1.79-4-4-4zm0 6c-1.1 0-2-.9-2-2s.9-2 2-2s2 .9 2 2s-.9 2-2 2z" />
        </svg>
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
    class="fixed z-[71] w-[120px] bg-[#1a1a1f]/98 backdrop-blur-md border border-white/10 rounded-lg shadow-2xl py-1"
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
