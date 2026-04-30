<script lang="ts">
  import { player } from "$lib/stores/player.svelte";
  import { setVolume } from "$lib/bindings/playback";

  let trackEl: HTMLDivElement;
  let dragging = $state(false);

  function handleVolume(e: MouseEvent) {
    if (!trackEl) return;
    const rect = trackEl.getBoundingClientRect();
    if (rect.width === 0) return;
    const vol = Math.round(Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width)) * 100);
    setVolume(vol);
    player.volume = vol;
    player.muted = vol === 0;
  }

  function toggleMute() {
    player.muted = !player.muted;
    setVolume(player.muted ? 0 : player.volume || 100);
  }

  const fillPct = $derived(player.muted ? 0 : player.volume);
</script>

<svelte:window onmouseup={() => (dragging = false)} onmousemove={(e) => dragging && handleVolume(e)} />

<div class="flex items-center group h-9">
  <button onclick={toggleMute} class="ctrl-btn w-9 h-9 shrink-0" title={player.muted ? "Unmute" : "Mute"}>
    <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
      {#if player.muted || player.volume === 0}
        <path d="M4.34 2.93L2.93 4.34L7.29 8.7L7 9H3v6h4l5 5v-6.59l4.18 4.18c-.65.49-1.38.88-2.18 1.11v2.06a8.94 8.94 0 0 0 3.61-1.75l2.05 2.05l1.41-1.41L4.34 2.93zM10 15.17L7.83 13H5v-2h2.83l.88-.88L10 11.41v3.76zM19 12c0 .82-.15 1.61-.41 2.34l1.53 1.53c.56-1.17.88-2.48.88-3.87c0-4.28-2.99-7.86-7-8.77v2.06c2.89.86 5 3.54 5 6.71zm-7-8l-1.88 1.88L12 7.76zm4.5 8A4.5 4.5 0 0 0 14 7.97v1.79l2.48 2.48c.01-.08.02-.16.02-.24z" />
      {:else}
        <path d="M3 9v6h4l5 5V4L7 9H3zm7-.17v6.34L7.83 13H5v-2h2.83L10 8.83zM16.5 12A4.5 4.5 0 0 0 14 7.97v8.05c1.48-.73 2.5-2.25 2.5-4.02zM14 3.23v2.06c2.89.86 5 3.54 5 6.71s-2.11 5.85-5 6.71v2.06c4.01-.91 7-4.49 7-8.77c0-4.28-2.99-7.86-7-8.77z" />
      {/if}
    </svg>
  </button>

  <!-- Expanding region: full row height so hover/click tolerate vertical drift -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="overflow-hidden h-9 flex items-center w-0 group-hover:w-24 transition-[width] duration-200 ease-out"
    class:!w-24={dragging}
  >
    <div
      class="relative w-20 h-9 ml-1 mr-2 flex items-center cursor-pointer"
      bind:this={trackEl}
      onmousedown={(e) => { dragging = true; handleVolume(e); }}
    >
      <!-- Visible thin track -->
      <div class="w-full h-1 rounded" style="background: #2e2e2e;">
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
