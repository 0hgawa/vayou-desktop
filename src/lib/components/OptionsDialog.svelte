<script lang="ts" generics="T">
  import { ICONS } from "$lib/icons";

  interface Option {
    value: T;
    label: string;
    style?: string;
  }

  let {
    title,
    options,
    selected,
    onselect,
    onclose,
  }: {
    title: string;
    options: Option[];
    selected: T;
    onselect: (value: T) => void;
    onclose: () => void;
  } = $props();

  function pick(v: T) { onselect(v); onclose(); }
</script>

<button
  aria-label="Close"
  class="fixed inset-0 z-[95] w-full h-full bg-black/40 border-none cursor-default"
  onclick={onclose}
></button>

<div
  data-panel
  role="dialog"
  tabindex="-1"
  class="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 z-[96] w-72 max-h-[70vh] bg-[#1a1a1f]/98 backdrop-blur-md border border-white/10 rounded-xl shadow-2xl text-[13px] text-white/90 flex flex-col select-none overflow-hidden"
  onclick={(e) => e.stopPropagation()}
  onkeydown={(e) => { if (e.key === "Escape") { e.stopPropagation(); onclose(); } }}
>
  <div class="px-4 py-3 font-medium text-sm">{title}</div>

  <div class="flex-1 overflow-y-auto py-1">
    {#each options as opt}
      <button
        class="w-full flex items-center gap-3 px-4 py-2.5 hover:bg-white/[0.06] text-left transition-colors {opt.value === selected ? 'text-accent' : 'text-white/85'}"
        style={opt.style ?? ""}
        onclick={() => pick(opt.value)}
      >
        <svg class="w-4 h-4 shrink-0 {opt.value === selected ? 'opacity-100' : 'opacity-0'}" fill="currentColor" viewBox="0 0 24 24">{@html ICONS.check}</svg>
        <span class="flex-1 truncate">{opt.label}</span>
      </button>
    {/each}
  </div>
</div>
