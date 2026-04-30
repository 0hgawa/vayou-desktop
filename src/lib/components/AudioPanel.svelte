<script lang="ts">
  import { getTracks, selectAudioTrack, setAudioDelay, type TrackInfo } from "$lib/bindings/tracks";
  import { t } from "$lib/i18n/index.svelte";
  import { langName } from "$lib/utils/lang-names";
  import { ICONS } from "$lib/icons";

  let { visible = $bindable(false) }: { visible: boolean } = $props();

  let tracks = $state<TrackInfo[]>([]);
  let delay = $state(0);

  async function refresh() {
    try {
      const all = await getTracks();
      tracks = all.filter((t) => t.track_type === "audio");
    } catch {}
  }

  $effect(() => { if (visible) refresh(); });

  async function handleSelect(id: number) {
    await selectAudioTrack(id);
    await refresh();
  }

  function handleDelayChange(delta: number) {
    delay = +(delay + delta).toFixed(1);
    setAudioDelay(delay);
  }
</script>

{#if visible}
  <button aria-label="Close" class="fixed inset-0 z-80 w-full h-full bg-transparent border-none cursor-default" onclick={() => visible = false}></button>

  <div data-panel class="fixed right-4 bottom-16 z-81 w-70 max-h-[60vh] bg-[#18181c]/95 backdrop-blur-md border border-white/10 rounded-xl shadow-2xl text-[13px] text-white/90 flex flex-col select-none">
    <!-- Header -->
    <div class="flex items-center px-3 py-2">
      <span class="font-medium text-xs">{t().audio}</span>
      <div class="flex-1"></div>
      <button class="ctrl-btn w-6 h-6 text-xs" onclick={() => visible = false}>✕</button>
    </div>

    <!-- Track list -->
    <div class="flex-1 overflow-y-auto max-h-62.5">
      {#each tracks as track}
        <button
          class="w-full flex items-center px-3 py-2 hover:bg-white/8 text-left {track.selected ? 'text-accent' : 'text-white/70'}"
          onclick={() => handleSelect(track.id)}
        >
          <svg class="w-4 h-4 shrink-0 mr-2 {track.selected ? 'opacity-100' : 'opacity-0'}" fill="currentColor" viewBox="0 0 24 24">{@html ICONS.check}</svg>
          <span class="flex-1 truncate">
            {track.title || langName(track.lang) || `Track ${track.id}`}
            {#if track.lang && track.title}
              <span class="text-white/30 ml-1">[{langName(track.lang)}]</span>
            {/if}
          </span>
        </button>
      {/each}

      {#if tracks.length === 0}
        <div class="px-3 py-4 text-center text-white/30 text-xs">{t().noAudioTracks}</div>
      {/if}
    </div>

    <!-- Delay control -->
    <div class="px-3 py-2 mt-1">
      <div class="flex items-center justify-between">
        <span class="text-white/50">{t().delay}</span>
        <div class="flex items-center gap-1">
          <button class="ctrl-btn w-6 h-6 text-xs" onclick={() => handleDelayChange(-0.1)}>-</button>
          <span class="w-14 text-center tabular-nums text-xs">{delay.toFixed(1)}s</span>
          <button class="ctrl-btn w-6 h-6 text-xs" onclick={() => handleDelayChange(0.1)}>+</button>
        </div>
      </div>
    </div>
  </div>
{/if}
