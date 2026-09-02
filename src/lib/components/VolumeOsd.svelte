<script lang="ts">
  import { player } from "$lib/stores/player.svelte";
  import { ICONS } from "$lib/icons";

  let visible = $state(false);
  let timer: ReturnType<typeof setTimeout> | null = null;
  // Plain (non-reactive) so updating it inside the effect doesn't re-trigger it.
  let lastTick = player.volumeOsdTick;

  // Briefly show the level when the USER changes volume/mute (each such action
  // bumps `volumeOsdTick`), so tweaks are visible even with the controls hidden.
  // Driven by the tick — not `volume` — so mpv's own volume echoes on load
  // don't pop the OSD. Each change resets the countdown; hidden before a file.
  $effect(() => {
    const tick = player.volumeOsdTick;
    if (tick === lastTick) return;
    lastTick = tick;
    if (player.duration <= 0) return;
    visible = true;
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => (visible = false), 1200);
  });

  const muted = $derived(player.muted || player.volume === 0);
  const pct = $derived(player.muted ? 0 : Math.round(player.volume));
</script>

{#if visible}
  <div
    class="absolute left-1/2 top-11 -translate-x-1/2 z-[120] flex items-center gap-2 h-[38px] pl-4 pr-[17px] rounded-full bg-black/70 border border-white/10 pointer-events-none select-none"
  >
    <svg class="w-[19px] h-[19px] text-white/90" fill="currentColor" viewBox="0 0 24 24">{@html muted ? ICONS.volumeOff : ICONS.volumeUp}</svg>
    <span class="text-[15px] font-semibold text-white/90 tabular-nums w-[42px] text-center">{pct}%</span>
  </div>
{/if}
