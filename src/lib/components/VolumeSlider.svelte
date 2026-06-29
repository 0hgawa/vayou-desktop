<script lang="ts">
  import { player } from "$lib/stores/player.svelte";
  import { setVolume } from "$lib/bindings/playback";
  import { settings } from "$lib/stores/settings.svelte";
  import { ICONS } from "$lib/icons";

  let trackEl: HTMLDivElement;
  let dragging = $state(false);

  const maxVol = $derived(settings.volumeBoost ? 200 : 100);

  function handleVolume(e: MouseEvent) {
    if (!trackEl) return;
    const rect = trackEl.getBoundingClientRect();
    if (rect.width === 0) return;
    const vol = Math.round(Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width)) * maxVol);
    setVolume(vol);
    player.volume = vol;
    player.muted = vol === 0;
  }

  function toggleMute() {
    player.muted = !player.muted;
    setVolume(player.muted ? 0 : player.volume || 100);
  }

  const fillPct = $derived(player.muted ? 0 : (player.volume / maxVol) * 100);
</script>

<svelte:window onmouseup={() => (dragging = false)} onmousemove={(e) => dragging && handleVolume(e)} />

<!-- Icon + slider as one unit: the hover pill wraps both (YouTube-style),
     so the highlight and expansion cover the whole control, not just the icon. -->
<div class="flex items-center group h-9 rounded-full transition-colors hover:bg-white/[0.12]">
  <button onclick={toggleMute} class="vol-btn w-9 h-9 shrink-0 flex items-center justify-center text-white/85 hover:text-white transition-colors" title={player.muted ? "Unmute" : "Mute"}>
    <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
      {@html player.muted || player.volume === 0 ? ICONS.volumeOff : ICONS.volumeUp}
    </svg>
  </button>

  <!-- Expanding region: full row height so hover/click tolerate vertical drift -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="overflow-hidden h-9 flex items-center w-0 group-hover:w-28 transition-[width] duration-200 ease-out"
    class:!w-28={dragging}
  >
    <div
      class="relative flex-1 h-9 ml-2 mr-5 flex items-center cursor-pointer"
      bind:this={trackEl}
      onmousedown={(e) => { dragging = true; handleVolume(e); }}
    >
      <!-- Visible thin track -->
      <div class="w-full h-1 rounded" style="background: var(--color-surface-container-highest);">
        <div class="h-full bg-white rounded pointer-events-none" style="width: {fillPct}%"></div>
      </div>
      <!-- Thumb -->
      <div
        class="absolute w-3 h-3 bg-white rounded-full pointer-events-none top-1/2 -translate-x-1/2 -translate-y-1/2"
        style="left: {fillPct}%"
      ></div>
    </div>
  </div>
</div>
