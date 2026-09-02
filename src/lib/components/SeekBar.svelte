<script lang="ts">
  import { player } from "$lib/stores/player.svelte";
  import { seekAbsolute, getChapters, type Chapter } from "$lib/bindings/playback";
  import { abLoop } from "$lib/stores/abLoop.svelte";
  import { formatTime } from "$lib/utils/format-time";
  import { t } from "$lib/i18n/index.svelte";

  let trackEl: HTMLDivElement;
  let seeking = $state(false);
  let chapters = $state<Chapter[]>([]);

  // Refresh chapters when duration changes (new file loaded)
  let lastDuration = 0;
  $effect(() => {
    if (player.duration > 0 && player.duration !== lastDuration) {
      lastDuration = player.duration;
      getChapters().then((c) => { chapters = c; }).catch(() => { chapters = []; });
    }
  });

  let pendingTime = 0;

  // Keyframe seek while dragging (fast), exact seek on release (precise).
  function handleSeek(e: MouseEvent, exact: boolean) {
    if (!trackEl || player.duration <= 0) return;
    const rect = trackEl.getBoundingClientRect();
    const fraction = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
    pendingTime = fraction * player.duration;
    player.currentTime = pendingTime;
    seekAbsolute(pendingTime, exact);
  }

  function onMouseDown(e: MouseEvent) {
    seeking = true;
    handleSeek(e, false);
  }
  function onMouseMove(e: MouseEvent) {
    if (seeking) handleSeek(e, false);
  }
  function onMouseUp() {
    if (!seeking) return;
    seeking = false;
    seekAbsolute(pendingTime, true);
  }

  /** Keyboard operation for the slider, matching the global seek shortcuts
   * (±5s, ±30s with Shift) plus Home/End for the ends of the file.
   *
   * Propagation is stopped because the same arrow keys are bound globally on
   * `<svelte:window>`; without it a focused seek bar would seek twice per
   * press. */
  function onKeyDown(e: KeyboardEvent) {
    if (player.duration <= 0) return;
    const step = e.shiftKey ? 30 : 5;
    let target: number;
    switch (e.key) {
      case "ArrowLeft": target = player.currentTime - step; break;
      case "ArrowRight": target = player.currentTime + step; break;
      case "Home": target = 0; break;
      case "End": target = player.duration; break;
      default: return;
    }
    e.preventDefault();
    e.stopPropagation();
    const clamped = Math.max(0, Math.min(player.duration, target));
    player.currentTime = clamped;
    seekAbsolute(clamped, true);
  }
</script>

<svelte:window onmouseup={onMouseUp} onmousemove={onMouseMove} />

<div
  class="w-full cursor-pointer py-2 slider-focus"
  bind:this={trackEl}
  role="slider"
  tabindex="0"
  aria-label={t().seekBar}
  aria-valuemin={0}
  aria-valuemax={Math.round(player.duration)}
  aria-valuenow={Math.round(player.currentTime)}
  aria-valuetext="{formatTime(player.currentTime)} / {formatTime(player.duration)}"
  onmousedown={onMouseDown}
  onkeydown={onKeyDown}
>
  <div class="seek-track">
    <div class="seek-progress" style="width: {player.progress}%"></div>
    {#if player.duration > 0 && abLoop.a !== null}
      {@const aPct = (abLoop.a / player.duration) * 100}
      {@const bPct = abLoop.b !== null ? (abLoop.b / player.duration) * 100 : 100}
      <div
        class="absolute top-0 h-full bg-white/25 pointer-events-none"
        style="left: {aPct}%; width: {bPct - aPct}%"
      ></div>
      <div
        class="absolute top-[-2px] h-[8px] w-[2px] bg-white pointer-events-none"
        style="left: {aPct}%"
      ></div>
      {#if abLoop.b !== null}
        <div
          class="absolute top-[-2px] h-[8px] w-[2px] bg-white pointer-events-none"
          style="left: {bPct}%"
        ></div>
      {/if}
    {/if}
    {#each chapters as ch}
      {#if player.duration > 0 && ch.time > 0}
        <div
          class="absolute top-0 h-full w-[2px] bg-white/40"
          style="left: {(ch.time / player.duration) * 100}%"
          title={ch.title}
        ></div>
      {/if}
    {/each}
    <div class="seek-thumb" style="left: {player.progress}%"></div>
  </div>
</div>
