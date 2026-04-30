<script lang="ts">
  import {
    getPlaylist, playlistAdd, playlistRemove, playlistPlayIndex,
    playlistClear, type PlaylistItem,
  } from "$lib/bindings/playlist";
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { t } from "$lib/i18n/index.svelte";

  let { visible = $bindable(false) }: { visible: boolean } = $props();

  let items = $state<PlaylistItem[]>([]);
  let repeatMode = $state<"off" | "all" | "one">("off");
  let shuffle = $state(false);

  function cycleRepeat() {
    repeatMode = repeatMode === "off" ? "all" : repeatMode === "all" ? "one" : "off";
    invoke("set_mpv_property", { name: "loop-playlist", value: repeatMode === "all" ? "inf" : "no" }).catch(() => {});
    invoke("set_mpv_property", { name: "loop-file", value: repeatMode === "one" ? "inf" : "no" }).catch(() => {});
  }

  async function toggleShuffle() {
    shuffle = !shuffle;
    try {
      await invoke("mpv_command", { args: shuffle ? ["playlist-shuffle"] : ["playlist-unshuffle"] });
    } catch {}
    await refresh();
  }

  async function refresh() {
    try { items = await getPlaylist(); } catch {}
  }

  $effect(() => { if (visible) refresh(); });

  async function handlePlay(index: number) {
    await playlistPlayIndex(index);
    await refresh();
  }

  async function handleRemove(index: number) {
    await playlistRemove(index);
    await refresh();
  }

  async function handleAddFiles() {
    const selected = await open({
      multiple: true,
      filters: [
        {
          name: "Media Files",
          extensions: [
            "mp4", "mkv", "avi", "mov", "wmv", "flv", "webm",
            "mpg", "mpeg", "m4v", "3gp", "ts", "vob",
            "mp3", "flac", "wav", "ogg", "m4a", "aac", "opus", "wma",
          ],
        },
        { name: "All Files", extensions: ["*"] },
      ],
    });
    if (selected && Array.isArray(selected)) {
      for (const path of selected) await playlistAdd(path);
      await refresh();
    } else if (selected) {
      await playlistAdd(selected as string);
      await refresh();
    }
  }

  async function handleAddFolder() {
    const selected = await open({ directory: true, multiple: false });
    if (selected) {
      // Scan folder for media files via Rust, or add all files manually
      // For now, we rely on mpv's ability to handle directories
      await playlistAdd(selected as string);
      await refresh();
    }
  }

  async function handleClear() {
    await playlistClear();
    await refresh();
  }
</script>

{#if visible}
  <button aria-label="Close" class="fixed inset-0 z-80 w-full h-full bg-transparent border-none cursor-default" onclick={() => visible = false}></button>

  <div data-panel class="fixed right-4 bottom-16 z-81 w-80 max-h-[80vh] bg-[#18181c]/95 backdrop-blur-md border border-white/10 rounded-xl shadow-2xl text-[13px] text-white/90 flex flex-col select-none">
    <!-- Header -->
    <div class="flex items-center px-3 py-2">
      <span class="font-medium text-xs">{t().playlist}</span>
      <span class="text-white/30 text-[11px] ml-2">{items.length} items</span>
      <div class="flex-1"></div>
      <button class="ctrl-btn w-6 h-6 text-xs" onclick={() => visible = false}>✕</button>
    </div>

    <!-- Items -->
    <div class="flex-1 overflow-y-auto max-h-[400px]">
      {#each items as item, i}
        <div
          class="group w-full flex items-center px-3 py-2 hover:bg-white/8 cursor-default {item.current ? 'bg-white/5' : ''}"
          role="option"
          aria-selected={item.current}
          tabindex="-1"
          ondblclick={() => handlePlay(item.index)}
          title={item.filename}
        >
          <span class="w-5 text-[11px] shrink-0 {item.current ? 'text-accent' : 'text-white/25'}">
            {item.current ? "▶" : i + 1}
          </span>

          <span class="flex-1 truncate mx-2 {item.current ? 'text-accent' : 'text-white/80'}">
            {item.title}
          </span>

          <button
            class="ctrl-btn w-5 h-5 opacity-0 group-hover:opacity-100 hover:text-red-400 transition-opacity"
            title={t().remove}
            onclick={(e) => { e.stopPropagation(); handleRemove(item.index); }}
          >✕</button>
        </div>
      {/each}

      {#if items.length === 0}
        <div class="px-3 py-8 text-center text-white/30 text-xs">{t().emptyPlaylist}</div>
      {/if}
    </div>

    <!-- Controls -->
    <div class="flex items-center px-2 py-1.5 gap-1">
      <button
        class="flex items-center justify-center w-7 h-7 rounded-md border-none transition-all {shuffle ? 'bg-accent/20 text-accent' : 'bg-transparent text-white/50'}"
        title={t().shuffle} onclick={toggleShuffle}
      >
        <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24"><path d="M10.59 9.17L5.41 4L4 5.41l5.17 5.17l1.42-1.41zM14.5 4l2.04 2.04L4 18.59L5.41 20L17.96 7.46L20 9.5V4h-5.5zm.33 9.41l-1.41 1.41l3.13 3.13L14.5 20H20v-5.5l-2.04 2.04l-3.13-3.13z"/></svg>
      </button>
      <button
        class="flex items-center justify-center w-7 h-7 rounded-md border-none transition-all relative {repeatMode !== 'off' ? 'bg-accent/20 text-accent' : 'bg-transparent text-white/50'}"
        title="{t().repeat}: {repeatMode}" onclick={cycleRepeat}
      >
        <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24"><path d="M7 7h10v3l4-4l-4-4v3H5v6h2V7zm10 10H7v-3l-4 4l4 4v-3h12v-6h-2v4z"/></svg>
        {#if repeatMode === "one"}<span class="absolute -bottom-0.5 -right-0.5 text-[7px] font-bold bg-accent text-black rounded-full w-3 h-3 flex items-center justify-center">1</span>{/if}
      </button>
      <div class="flex-1"></div>
      <button class="ctrl-btn w-7 h-7 rounded-md hover:bg-white/10 text-white/50 hover:text-white/90" title={t().addFiles} onclick={handleAddFiles}>
        <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24"><path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/></svg>
      </button>
      <button class="ctrl-btn w-7 h-7 rounded-md hover:bg-white/10 text-white/50 hover:text-white/90" title={t().addFolder} onclick={handleAddFolder}>
        <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24"><path d="M9.17 6l2 2H20v10H4V6h5.17M10 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"/></svg>
      </button>
      <button class="ctrl-btn w-7 h-7 rounded-md hover:bg-white/10 text-white/50 hover:text-red-400" title={t().clear} onclick={handleClear}>
        <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24"><path d="M16 9v10H8V9h8m-1.5-6h-5l-1 1H5v2h14V4h-3.5l-1-1zM18 7H6v12c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7z"/></svg>
      </button>
    </div>
  </div>
{/if}
