<script lang="ts">
  import { player } from "$lib/stores/player.svelte";
  import { seekAbsolute, getChapters, type Chapter } from "$lib/bindings/playback";
  import { abLoop } from "$lib/stores/abLoop.svelte";

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

  function handleSeek(e: MouseEvent) {
    if (!trackEl || player.duration <= 0) return;
    const rect = trackEl.getBoundingClientRect();
    const fraction = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
    const time = fraction * player.duration;
    player.currentTime = time;
    seekAbsolute(time);
  }

  function onMouseDown(e: MouseEvent) {
    seeking = true;
    handleSeek(e);
  }
  function onMouseMove(e: MouseEvent) {
    if (seeking) handleSeek(e);
  }
  function onMouseUp() {
    seeking = false;
  }
</script>

<svelte:window onmouseup={onMouseUp} onmousemove={onMouseMove} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="w-full cursor-pointer py-2" bind:this={trackEl} onmousedown={onMouseDown}>
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
