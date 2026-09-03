<script lang="ts">
  import { player } from "$lib/stores/player.svelte";
  import { setVolume } from "$lib/bindings/playback";
  import { settings } from "$lib/stores/settings.svelte";
  import { ICONS } from "$lib/icons";
  import { t } from "$lib/i18n/index.svelte";

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
    player.pulseVolumeOsd();
  }

  function toggleMute() {
    player.muted = !player.muted;
    setVolume(player.muted ? 0 : player.volume || 100);
    player.pulseVolumeOsd();
  }

  const fillPct = $derived(player.muted ? 0 : (player.volume / maxVol) * 100);
  const shownVol = $derived(player.muted ? 0 : Math.round(player.volume));

  /** Nothing is coming out, whether the mute flag is set or the slider is at
   * zero. Both the icon and the button's label follow this rather than
   * `player.muted` alone, so neither can describe a state the other denies. */
  const silent = $derived(player.muted || player.volume === 0);

  /** Keyboard operation, matching the global volume shortcuts (±5). Stops
   * propagation because the same arrows are bound on `<svelte:window>`, which
   * would otherwise apply the step a second time. */
  function onKeyDown(e: KeyboardEvent) {
    let target: number;
    switch (e.key) {
      case "ArrowLeft":
      case "ArrowDown": target = shownVol - 5; break;
      case "ArrowRight":
      case "ArrowUp": target = shownVol + 5; break;
      case "Home": target = 0; break;
      case "End": target = maxVol; break;
      default: return;
    }
    e.preventDefault();
    e.stopPropagation();
    const vol = Math.max(0, Math.min(maxVol, target));
    setVolume(vol);
    player.volume = vol;
    player.muted = vol === 0;
    player.pulseVolumeOsd();
  }
</script>

<svelte:window onmouseup={() => (dragging = false)} onmousemove={(e) => dragging && handleVolume(e)} />

<!-- Icon + slider as one unit: the hover pill wraps both (YouTube-style),
     so the highlight and expansion cover the whole control, not just the icon. -->
<div class="flex items-center group h-9 rounded-full transition-colors hover:bg-white/[0.12]">
  <!-- The label names what the click will do, not what the state is: the button
       said "Mute" while already muted, which is the one moment it is wrong. -->
  <button
    onclick={toggleMute}
    class="vol-btn w-9 h-9 shrink-0 flex items-center justify-center text-white/85 hover:text-white transition-colors"
    aria-label={silent ? t().unmute : t().mute}
    aria-pressed={player.muted}
    title={silent ? t().unmute : t().mute}
  >
    <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
      {@html silent ? ICONS.volumeOff : ICONS.volumeUp}
    </svg>
  </button>

  <!-- Expanding region: full row height so hover/click tolerate vertical drift.
       It also expands on `focus-within`, otherwise tabbing to the slider would
       land on a zero-width control the user cannot see. -->
  <div
    class="overflow-hidden h-9 flex items-center w-0 group-hover:w-28 group-focus-within:w-28 transition-[width] duration-200 ease-out"
    class:!w-28={dragging}
  >
    <div
      class="relative flex-1 h-9 ml-2 mr-5 flex items-center cursor-pointer slider-focus"
      bind:this={trackEl}
      role="slider"
      tabindex="0"
      aria-label={t().volume}
      aria-valuemin={0}
      aria-valuemax={maxVol}
      aria-valuenow={shownVol}
      aria-valuetext="{shownVol}%"
      onmousedown={(e) => { dragging = true; handleVolume(e); }}
      onkeydown={onKeyDown}
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
