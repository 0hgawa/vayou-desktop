<script lang="ts">
  import { toast } from "$lib/stores/toast.svelte";
</script>

{#if toast.message}
  <div
    role="status"
    aria-live="polite"
    class="toast fixed top-14 left-1/2 z-[200] px-4 py-2 bg-surface-container-high/95 backdrop-blur-md border border-white/10 rounded-full shadow-2xl text-[13px] text-white/90 select-none pointer-events-none whitespace-nowrap"
  >
    {toast.message}
  </div>
{/if}

<style>
  /* All transform handled here so the Tailwind `transform` utilities
   * don't fight the CSS animation (the conflict was making the toast
   * snap horizontally after the fade-in finished). */
  @keyframes toast-in {
    from { opacity: 0; transform: translate(-50%, -8px); }
    to   { opacity: 1; transform: translate(-50%, 0); }
  }
  .toast {
    transform: translate(-50%, 0);
    animation: toast-in 180ms ease-out;
  }
</style>
