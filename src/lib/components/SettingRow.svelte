<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    icon,
    title,
    value = "",
    onclick,
    trailing,
    below,
  }: {
    icon: string;
    title: string;
    value?: string;
    onclick?: () => void;
    trailing?: Snippet;
    below?: Snippet;
  } = $props();
</script>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div
  class="flex items-center gap-3 pl-3 pr-4 py-1.5 {onclick ? 'cursor-pointer hover:bg-white/[0.04]' : ''} transition-colors"
  role={onclick ? "button" : undefined}
  tabindex={onclick ? 0 : undefined}
  {onclick}
  onkeydown={onclick ? (e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); onclick?.(); } } : undefined}
>
  <svg class="w-5 h-5 text-white/55 shrink-0" fill="currentColor" viewBox="0 0 24 24">{@html icon}</svg>
  <div class="flex-1 min-w-0">
    {#if below}
      <div class="flex items-baseline justify-between gap-3">
        <div class="text-[13px] text-white/90 truncate leading-tight">{title}</div>
        {#if value}<div class="text-[11px] text-white/55 tabular-nums shrink-0">{value}</div>{/if}
      </div>
      <div class="mt-1.5">{@render below()}</div>
    {:else}
      <div class="text-[13px] text-white/90 truncate leading-tight">{title}</div>
      {#if value}<div class="text-[11px] text-white/45 truncate mt-0.5 leading-tight">{value}</div>{/if}
    {/if}
  </div>
  {#if trailing}<div class="shrink-0">{@render trailing()}</div>{/if}
</div>
