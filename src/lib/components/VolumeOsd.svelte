<script lang="ts">
  import { player } from "$lib/stores/player.svelte";
  import { ICONS } from "$lib/icons";

  let visible = $state(false);
  let timer: ReturnType<typeof setTimeout> | null = null;
  // Plain (non-reactive) so updating it inside the effect doesn't re-trigger it.
  let last = { v: player.volume, m: player.muted };

  // Briefly show the level whenever the volume or mute state changes (e.g. via
  // the keyboard/scroll), so tweaks are visible even with the controls hidden.
  // Each change resets the countdown; suppressed before a file plays.
  $effect(() => {
    const v = player.volume;
    const m = player.muted;
    const d = player.duration;
    if (v === last.v && m === last.m) return;
    last = { v, m };
    if (d <= 0) return;
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
