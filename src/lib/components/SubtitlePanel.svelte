<script lang="ts">
  import { getTracks, selectSubtitle, loadSubtitle, setSubtitleDelay, type TrackInfo } from "$lib/bindings/tracks";
  import { searchSubtitles, downloadSubtitle, type SubResult } from "$lib/bindings/opensubtitles";
  import { open } from "@tauri-apps/plugin-dialog";
  import Select from "./Select.svelte";
  import { t } from "$lib/i18n/index.svelte";
  import { settings, subFonts } from "$lib/stores/settings.svelte";
  import { player } from "$lib/stores/player.svelte";
  import { translate } from "$lib/stores/translate.svelte";
  import { langName } from "$lib/utils/lang-names";
  import { ICONS } from "$lib/icons";

  let { visible = $bindable(false) }: { visible: boolean } = $props();

  // Mirrors mobile LANGUAGES list (OnlineSubtitleSearchView.kt)
  const LANGUAGES: Array<[string, string]> = [
    ["", "All"],
    ["por", "Português"],
    ["pob", "Português (BR)"],
    ["eng", "English"],
    ["spa", "Español"],
    ["fre", "Français"],
    ["ger", "Deutsch"],
    ["ita", "Italiano"],
    ["dut", "Nederlands"],
    ["rus", "Русский"],
    ["jpn", "日本語"],
    ["chi", "中文"],
    ["kor", "한국어"],
    ["ara", "العربية"],
    ["tur", "Türkçe"],
    ["pol", "Polski"],
    ["rum", "Română"],
    ["hrv", "Hrvatski"],
    ["scc", "Srpski"],
    ["hun", "Magyar"],
    ["cze", "Čeština"],
    ["hin", "हिन्दी"],
  ];

  let searchQuery = $state("");
  let searchLang = $state("");
  let searchResults = $state<SubResult[]>([]);
  let searching = $state(false);
  let downloadingIndex = $state<number | null>(null);
  let searchError = $state("");
  let hasSearched = $state(false);
  let queryDirty = $state(false);

  // Auto-fill the query from the current playing title until the user types.
  // Reactive on player.title — updates for free when the playlist advances.
  $effect(() => {
    if (!queryDirty) {
      searchQuery = (player.title || "").replace(/\.[^.]+$/, "");
    }
  });

  function formatDownloads(count: string): string {
    const n = Number(count);
    if (!Number.isFinite(n)) return count;
    if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M dl`;
    if (n >= 1_000) return `${(n / 1_000).toFixed(1)}k dl`;
    return `${n} dl`;
  }

  function openSearch() {
    page = "search";
    searchError = "";
    if (!hasSearched && searchQuery) handleSearch();
  }

  async function handleSearch() {
    searching = true;
    searchError = "";
    searchResults = [];
    hasSearched = true;
    try {
      searchResults = await searchSubtitles(searchQuery.trim(), searchLang);
    } catch (e: any) {
      searchError = String(e);
    }
    searching = false;
  }

  async function handleDownload(result: SubResult, index: number) {
    if (downloadingIndex !== null) return;
    downloadingIndex = index;
    searchError = "";
    try {
      await downloadSubtitle(result);
      page = "main";
      refresh();
    } catch (e: any) {
      searchError = t().downloadFailed;
    }
    downloadingIndex = null;
  }

  const translateLangs = [
    { code: "pt", name: "Português" }, { code: "en", name: "English" }, { code: "es", name: "Español" },
    { code: "fr", name: "Français" }, { code: "de", name: "Deutsch" }, { code: "it", name: "Italiano" },
    { code: "ja", name: "日本語" }, { code: "ko", name: "한국어" }, { code: "zh", name: "中文" },
    { code: "ru", name: "Русский" }, { code: "ar", name: "العربية" }, { code: "hi", name: "हिन्दी" },
  ];

  // Refresh the track list whenever the store flips `translating` off — the
  // new translation track only appears in mpv after sub-add completes.
  $effect(() => {
    if (!translate.translating) refresh();
  });

  async function handleOff() {
    await translate.clear();
    await refresh();
    page = "main";
  }

  let tracks = $state<TrackInfo[]>([]);
  let delay = $state(0);
  let page = $state<"main" | "style" | "search" | "searchLang" | "translateLang">("main");

  const noSubSelected = $derived(tracks.length === 0 || tracks.every((t) => !t.selected));
  const translateOff = $derived(settings.translateLang === "off");
  const translateActive = $derived(!translateOff && translate.translatedForPath !== null && translate.translatedForPath === player.title);
  const translateLangLabel = $derived(
    translateOff ? "Off" : (translateLangs.find((l) => l.code === settings.translateLang)?.name ?? settings.translateLang),
  );
  const searchLangLabel = $derived(LANGUAGES.find(([id]) => id === searchLang)?.[1] ?? "All");

  async function refresh() {
    try {
      const all = await getTracks();
      tracks = all.filter((t) => t.track_type === "sub");
    } catch {}
  }

  $effect(() => { if (visible) { refresh(); page = "main"; } });

  async function handleSelect(id: number) {
    await selectSubtitle(id);
    await refresh();
    const selectedNow = tracks.find((t) => t.track_type === "sub" && t.selected);
    // Avoid re-translating if user just clicked our own AUTO track.
    const clickedAuto = !!translate.translationTrackPath
      && selectedNow?.external
      && selectedNow.external_filename === translate.translationTrackPath;
    if (clickedAuto) return;
    // Re-translate so switching the source language preserves the translation.
    if (settings.translateLang !== "off" && id !== -1 && !translate.translating) {
      translate.translate();
    }
  }

  async function handleLoadExternal() {
    const selected = await open({
      multiple: false,
      filters: [
        { name: "Subtitles", extensions: ["srt", "ass", "ssa", "sub", "vtt", "idx", "sup"] },
        { name: "All Files", extensions: ["*"] },
      ],
    });
    if (selected) {
      await loadSubtitle(selected as string);
      await refresh();
    }
  }

  function handleDelayChange(delta: number) {
    delay = +(delay + delta).toFixed(1);
    setSubtitleDelay(delay);
  }
</script>

{#if visible}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <button aria-label="Close" class="fixed inset-0 z-80 w-full h-full bg-transparent border-none cursor-default" onclick={() => visible = false}></button>

  <div data-panel class="fixed right-4 bottom-16 z-81 w-70 max-h-[80vh] bg-[#18181c]/95 backdrop-blur-md border border-white/10 rounded-xl shadow-2xl text-[13px] text-white/90 flex flex-col select-none overflow-hidden">

    {#if page === "main"}
      <!-- Header -->
      <div class="flex items-center px-3 py-2">
        <span class="font-medium text-xs">{t().subtitles}</span>
        <div class="flex-1"></div>
        <button class="ctrl-btn w-6 h-6 text-xs" onclick={() => visible = false}>✕</button>
      </div>

      <div class="flex-1 overflow-y-auto">
      <!-- Track list (Disable as virtual entry on top) -->
      <div class="max-h-[200px] overflow-y-auto">
        <button
          class="w-full flex items-center px-3 py-2 hover:bg-white/8 text-left {noSubSelected ? 'text-accent' : 'text-white/70'}"
          onclick={() => handleSelect(-1)}
        >
          <svg class="w-4 h-4 shrink-0 mr-2 {noSubSelected ? 'opacity-100' : 'opacity-0'}" fill="currentColor" viewBox="0 0 24 24">{@html ICONS.check}</svg>
          <span class="flex-1 truncate">{t().disableSubs}</span>
        </button>
        {#each tracks as track}
          {@const isAuto = !!translate.translationTrackPath && track.external && track.external_filename === translate.translationTrackPath}
          <button
            class="w-full flex items-center px-3 py-2 hover:bg-white/8 text-left {track.selected ? 'text-accent' : 'text-white/70'}"
            onclick={() => handleSelect(track.id)}
          >
            <svg class="w-4 h-4 shrink-0 mr-2 {track.selected ? 'opacity-100' : 'opacity-0'}" fill="currentColor" viewBox="0 0 24 24">{@html ICONS.check}</svg>
            <span class="flex-1 truncate">
              {track.title || langName(track.lang) || `Track ${track.id}`}
              {#if track.lang && track.title && !isAuto}
                <span class="text-white/30 ml-1">[{langName(track.lang)}]</span>
              {/if}
            </span>
            {#if isAuto}
              <span class="ml-2 px-1.5 py-0.5 rounded bg-accent/15 text-accent text-[9px] font-semibold tracking-wider uppercase">Auto</span>
            {/if}
          </button>
        {/each}
      </div>

      <div class="border-t border-white/[0.06] my-1.5 mx-3"></div>

      <!-- Open local subtitle -->
      <button
        class="w-full flex items-center gap-3 px-3 py-2 hover:bg-white/8 text-white/85"
        onclick={handleLoadExternal}
      >
        <svg class="w-4 h-4 text-white/55" fill="currentColor" viewBox="0 0 24 24"><path d="M14 2H6c-1.1 0-1.99.9-1.99 2L4 20c0 1.1.89 2 1.99 2H18c1.1 0 2-.9 2-2V8l-6-6zM6 20V4h7v5h5v11H6z"/></svg>
        <span class="flex-1 text-left">{t().loadExternalFile}</span>
      </button>

      <!-- Search online -->
      <button
        class="w-full flex items-center gap-3 px-3 py-2 hover:bg-white/8 text-white/85"
        onclick={openSearch}
      >
        <svg class="w-4 h-4 text-white/55" fill="currentColor" viewBox="0 0 24 24"><path d="M15.5 14h-.79l-.28-.27A6.471 6.471 0 0 0 16 9.5A6.5 6.5 0 1 0 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5S14 7.01 14 9.5S11.99 14 9.5 14z"/></svg>
        <span class="flex-1 text-left">{t().searchOnline}</span>
        <svg class="w-4 h-4 text-white/30" fill="currentColor" viewBox="0 0 24 24"><path d="M10 6L8.59 7.41L13.17 12l-4.58 4.59L10 18l6-6l-6-6z"/></svg>
      </button>

      <!-- Translate row + action button on right -->
      <!-- Translate: single row that opens a dedicated sub-page where each
           language is itself the action (one click = pick + translate). -->
      <button
        class="w-full flex items-center gap-3 px-3 py-2 hover:bg-white/8 text-white/85"
        onclick={() => page = "translateLang"}
        disabled={translate.translating}
      >
        <svg class="w-4 h-4 text-white/55 shrink-0" fill="currentColor" viewBox="0 0 24 24"><path d="M12.87 15.07l-2.54-2.51l.03-.03A17.52 17.52 0 0 0 14.07 6H17V4h-7V2H8v2H1v1.99h11.17C11.5 7.92 10.44 9.75 9 11.35C8.07 10.32 7.3 9.19 6.69 8h-2c.73 1.63 1.73 3.17 2.98 4.56l-5.09 5.02L4 19l5-5l3.11 3.11l.76-2.04zM18.5 10h-2L12 22h2l1.12-3h4.75L21 22h2l-4.5-12zm-2.62 7l1.62-4.33L19.12 17h-3.24z"/></svg>
        <span class="flex-1 text-left">{t().translate}</span>
        {#if translate.translating}
          <span class="text-[11px] text-white/55 tabular-nums">{translate.total > 0 ? Math.round((translate.progress / translate.total) * 100) : 0}%</span>
          <span class="w-3.5 h-3.5 border-2 border-white/20 border-t-accent rounded-full animate-spin shrink-0"></span>
        {:else}
          <span class="text-[12px] {translateActive ? 'text-accent' : 'text-white/40'}">{translateLangLabel}</span>
          <svg class="w-4 h-4 text-white/30 shrink-0" fill="currentColor" viewBox="0 0 24 24"><path d="M10 6L8.59 7.41L13.17 12l-4.58 4.59L10 18l6-6l-6-6z"/></svg>
        {/if}
      </button>

      {#if translate.error}
        <div class="px-3 pb-1 text-red-400 text-[11px] truncate" title={translate.error}>{translate.error}</div>
      {/if}

      <!-- Customization (style) -->
      <button
        class="w-full flex items-center gap-3 px-3 py-2 hover:bg-white/8 text-white/85"
        onclick={() => page = "style"}
      >
        <svg class="w-4 h-4 text-white/55" fill="currentColor" viewBox="0 0 24 24"><path d="M12 3a9 9 0 0 0 0 18c.83 0 1.5-.67 1.5-1.5c0-.39-.15-.74-.39-1.01c-.23-.26-.38-.61-.38-.99c0-.83.67-1.5 1.5-1.5H16c2.76 0 5-2.24 5-5c0-4.42-4.03-8-9-8zm-5.5 9a1.5 1.5 0 1 1 0-3a1.5 1.5 0 0 1 0 3zm3-4a1.5 1.5 0 1 1 0-3a1.5 1.5 0 0 1 0 3zm5 0a1.5 1.5 0 1 1 0-3a1.5 1.5 0 0 1 0 3zm3 4a1.5 1.5 0 1 1 0-3a1.5 1.5 0 0 1 0 3z"/></svg>
        <span class="flex-1 text-left">{t().style}</span>
        <svg class="w-4 h-4 text-white/30" fill="currentColor" viewBox="0 0 24 24"><path d="M10 6L8.59 7.41L13.17 12l-4.58 4.59L10 18l6-6l-6-6z"/></svg>
      </button>

      <div class="border-t border-white/[0.06] my-1.5 mx-3"></div>

      <!-- Delay -->
      <div class="w-full flex items-center gap-3 px-3 py-2 text-white/85">
        <svg class="w-4 h-4 text-white/55" fill="currentColor" viewBox="0 0 24 24"><path d="M15 1H9v2h6V1zm-4 13h2V8h-2v6zm8.03-6.61l1.42-1.42c-.43-.51-.9-.99-1.41-1.41l-1.42 1.42A8.962 8.962 0 0 0 12 4c-4.97 0-9 4.03-9 9s4.02 9 9 9s9-4.03 9-9c0-2.12-.74-4.07-1.97-5.61zM12 20c-3.87 0-7-3.13-7-7s3.13-7 7-7s7 3.13 7 7s-3.13 7-7 7z"/></svg>
        <span class="flex-1">{t().delay}</span>
        <button class="ctrl-btn w-6 h-6 text-xs" onclick={() => handleDelayChange(-0.1)}>−</button>
        <span class="w-12 text-center tabular-nums text-xs">{delay.toFixed(1)}s</span>
        <button class="ctrl-btn w-6 h-6 text-xs" onclick={() => handleDelayChange(0.1)}>+</button>
      </div>
      </div>

    {:else if page === "style"}
      <!-- Style header -->
      <div class="flex items-center px-3 py-2">
        <button class="ctrl-btn w-6 h-6 text-xs mr-2 hover:bg-white/10 rounded-md" onclick={() => page = "main"}>←</button>
        <span class="font-medium text-xs">{t().style}</span>
        <div class="flex-1"></div>
        <button class="text-xs text-white/40 hover:text-white/70" onclick={() => settings.resetSubStyle()}>{t().reset}</button>
      </div>

      <!-- Style controls -->
      <div class="flex-1 overflow-y-auto p-3 space-y-3">
        <div>
          <span class="text-white/50 text-xs block mb-1">{t().font}</span>
          <div class="flex items-center gap-1.5">
            <div class="flex-1"><Select items={subFonts} value={settings.subFont} itemStyle={(f) => `font-family:'${f}'`} onchange={(f) => { settings.subFont = f; settings.applySubStyle(); }} /></div>
            <button
              class="w-8 h-8 rounded-md text-sm font-bold transition-all {settings.subBold ? 'bg-accent/20 text-accent' : 'bg-white/8 text-white/60 hover:bg-white/12'}"
              onclick={() => { settings.subBold = !settings.subBold; settings.applySubStyle(); }}
            >B</button>
          </div>
        </div>

        <div>
          <div class="flex items-center justify-between mb-1">
            <span class="text-white/50 text-xs">{t().size}</span>
            <span class="text-white/50 text-xs tabular-nums">{settings.subSize}</span>
          </div>
          <input type="range" min="20" max="100" bind:value={settings.subSize} oninput={() => settings.applySubStyle()} class="s-range w-full" style="--val: {((settings.subSize - 20) / 80) * 100}%" />
        </div>

        <div class="flex items-center gap-4">
          <div class="flex items-center gap-2">
            <span class="text-white/50 text-xs">{t().text}</span>
            <input type="color" bind:value={settings.subColor} oninput={() => settings.applySubStyle()} class="s-color" />
          </div>
          <div class="flex items-center gap-2">
            <span class="text-white/50 text-xs">{t().border}</span>
            <input type="color" bind:value={settings.subBorderColor} oninput={() => settings.applySubStyle()} class="s-color" />
          </div>
        </div>

        <div>
          <div class="flex items-center justify-between mb-1">
            <span class="text-white/50 text-xs">{t().borderSize}</span>
            <span class="text-white/50 text-xs tabular-nums">{settings.subBorderSize}</span>
          </div>
          <input type="range" min="0" max="10" bind:value={settings.subBorderSize} oninput={() => settings.applySubStyle()} class="s-range w-full" style="--val: {(settings.subBorderSize / 10) * 100}%" />
        </div>

        <div>
          <div class="flex items-center justify-between mb-1">
            <span class="text-white/50 text-xs">{t().position}</span>
            <span class="text-white/50 text-xs tabular-nums">{settings.subPosition}%</span>
          </div>
          <input type="range" min="0" max="100" bind:value={settings.subPosition} oninput={() => settings.applySubStyle()} class="s-range w-full" style="--val: {settings.subPosition}%" />
        </div>
      </div>
    {:else if page === "translateLang"}
      <!-- Each language IS the action: click translates immediately to that
           language. Removes the need for a separate "Translate" button. -->
      <div class="flex items-center px-3 py-2">
        <button class="ctrl-btn w-6 h-6 text-xs mr-2 hover:bg-white/10 rounded-md" onclick={() => page = "main"}>←</button>
        <span class="font-medium text-xs">{t().translate}</span>
        {#if translate.translating}
          <div class="flex-1"></div>
          <span class="text-[11px] text-white/55 tabular-nums">{translate.total > 0 ? Math.round((translate.progress / translate.total) * 100) : 0}%</span>
          <span class="w-3.5 h-3.5 ml-2 border-2 border-white/20 border-t-accent rounded-full animate-spin"></span>
        {/if}
      </div>
      {#if translate.error}
        <div class="px-3 pb-1 text-red-400 text-[11px] truncate" title={translate.error}>{translate.error}</div>
      {/if}
      <div class="flex-1 overflow-y-auto">
        <button
          class="w-full flex items-center gap-3 px-3 py-2 hover:bg-white/8 disabled:opacity-50 text-left {translateOff ? 'text-accent' : 'text-white/60'}"
          onclick={handleOff}
          disabled={translate.translating}
        >
          <svg class="w-4 h-4 shrink-0 {translateOff ? 'opacity-100 text-accent' : 'opacity-0'}" fill="currentColor" viewBox="0 0 24 24">{@html ICONS.check}</svg>
          <span class="flex-1">{t().off}</span>
        </button>
        <div class="border-t border-white/[0.06] my-1.5 mx-3"></div>
        {#each translateLangs as l}
          <button
            class="w-full flex items-center gap-3 px-3 py-2 hover:bg-white/8 disabled:opacity-50 text-left {settings.translateLang === l.code ? 'text-accent' : 'text-white/85'}"
            onclick={() => { settings.translateLang = l.code; settings.save(); translate.translate(); page = "main"; }}
            disabled={translate.translating}
          >
            <svg class="w-4 h-4 shrink-0 {settings.translateLang === l.code ? 'opacity-100 text-accent' : 'opacity-0'}" fill="currentColor" viewBox="0 0 24 24">{@html ICONS.check}</svg>
            <span class="flex-1">{l.name}</span>
          </button>
        {/each}
      </div>
    {:else if page === "searchLang"}
      <!-- Search language picker -->
      <div class="flex items-center px-3 py-2">
        <button class="ctrl-btn w-6 h-6 text-xs mr-2 hover:bg-white/10 rounded-md" onclick={() => page = "search"}>←</button>
        <span class="font-medium text-xs">{t().searchOnlineTitle}</span>
      </div>
      <div class="flex-1 overflow-y-auto">
        {#each LANGUAGES as [code, label]}
          <button
            class="w-full flex items-center gap-3 px-3 py-2 hover:bg-white/8 text-left {searchLang === code ? 'text-accent' : 'text-white/80'}"
            onclick={() => { searchLang = code; page = "search"; handleSearch(); }}
          >
            <svg class="w-4 h-4 shrink-0 {searchLang === code ? 'opacity-100' : 'opacity-0'}" fill="currentColor" viewBox="0 0 24 24">{@html ICONS.check}</svg>
            <span class="flex-1">{label}</span>
          </button>
        {/each}
      </div>
    {:else if page === "search"}
      <!-- Search online -->
      <div class="flex items-center gap-2 px-3 py-2">
        <button class="ctrl-btn w-6 h-6 text-xs hover:bg-white/10 rounded-md" onclick={() => page = "main"}>←</button>
        <span class="font-medium text-xs">{t().searchOnlineTitle}</span>
      </div>

      <div class="px-3 py-2.5">
        <div class="relative">
          <input
            type="text"
            class="s-input pr-7"
            placeholder={t().searchSubtitle}
            bind:value={searchQuery}
            oninput={() => { queryDirty = true; }}
            onkeydown={(e) => { if (e.key === "Enter") handleSearch(); }}
          />
          {#if searchQuery}
            <button
              class="absolute right-1.5 top-1/2 -translate-y-1/2 ctrl-btn w-5 h-5 text-xs"
              aria-label="Clear"
              onclick={() => { searchQuery = ""; queryDirty = true; }}
            >✕</button>
          {:else}
            <button
              class="absolute right-1.5 top-1/2 -translate-y-1/2 ctrl-btn w-5 h-5"
              aria-label="Search"
              disabled={searching}
              onclick={handleSearch}
            >
              <svg class="w-3.5 h-3.5" fill="currentColor" viewBox="0 0 24 24"><path d="M15.5 14h-.79l-.28-.27A6.471 6.471 0 0 0 16 9.5A6.5 6.5 0 1 0 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5S14 7.01 14 9.5S11.99 14 9.5 14z"/></svg>
            </button>
          {/if}
        </div>
      </div>

      <button
        class="w-full flex items-center gap-3 px-3 py-2 hover:bg-white/8 text-white/85"
        onclick={() => page = "searchLang"}
      >
        <svg class="w-4 h-4 text-white/55" fill="currentColor" viewBox="0 0 24 24"><path d="M11.99 2C6.47 2 2 6.48 2 12s4.47 10 9.99 10C17.52 22 22 17.52 22 12S17.52 2 11.99 2zm6.93 6h-2.95a15.65 15.65 0 0 0-1.38-3.56A8.03 8.03 0 0 1 18.92 8zM12 4.04c.83 1.2 1.48 2.53 1.91 3.96h-3.82c.43-1.43 1.08-2.76 1.91-3.96zM4.26 14C4.1 13.36 4 12.69 4 12s.1-1.36.26-2h3.38c-.08.66-.14 1.32-.14 2c0 .68.06 1.34.14 2H4.26zm.82 2h2.95c.32 1.25.78 2.45 1.38 3.56A7.987 7.987 0 0 1 5.08 16zm2.95-8H5.08a7.987 7.987 0 0 1 4.33-3.56A15.65 15.65 0 0 0 8.03 8zM12 19.96c-.83-1.2-1.48-2.53-1.91-3.96h3.82c-.43 1.43-1.08 2.76-1.91 3.96zM14.34 14H9.66c-.09-.66-.16-1.32-.16-2c0-.68.07-1.35.16-2h4.68c.09.65.16 1.32.16 2c0 .68-.07 1.34-.16 2zm.25 5.56c.6-1.11 1.06-2.31 1.38-3.56h2.95a8.03 8.03 0 0 1-4.33 3.56zM16.36 14c.08-.66.14-1.32.14-2c0-.68-.06-1.34-.14-2h3.38c.16.64.26 1.31.26 2s-.1 1.36-.26 2h-3.38z"/></svg>
        <span class="flex-1 text-left">{searchLangLabel}</span>
        <svg class="w-4 h-4 text-white/30" fill="currentColor" viewBox="0 0 24 24"><path d="M10 6L8.59 7.41L13.17 12l-4.58 4.59L10 18l6-6l-6-6z"/></svg>
      </button>

      <div class="flex-1 overflow-y-auto">
        {#if searching}
          <div class="flex items-center justify-center py-8 text-white/40 text-xs">{t().searching}</div>
        {:else if searchResults.length > 0}
          {#each searchResults as sub, i}
            <button
              class="w-full text-left px-3 py-2.5 hover:bg-white/8 disabled:opacity-30 flex items-center gap-3"
              disabled={downloadingIndex !== null}
              onclick={() => handleDownload(sub, i)}
            >
              <span class="text-accent text-[11px] font-medium uppercase tabular-nums w-9 shrink-0">{sub.lang}</span>
              <span class="flex-1 min-w-0">
                <span class="block text-white/85 text-xs truncate">{sub.name}</span>
                <span class="flex items-center gap-2 mt-0.5 text-[11px] text-white/40">
                  <span>{formatDownloads(sub.downloads)}</span>
                  {#if sub.matched_by}
                    <span>{sub.matched_by === "moviehash" ? "hash" : sub.matched_by}</span>
                  {/if}
                </span>
              </span>
              {#if downloadingIndex === i}
                <span class="w-3.5 h-3.5 border-2 border-white/20 border-t-accent rounded-full animate-spin shrink-0"></span>
              {/if}
            </button>
          {/each}
        {:else if hasSearched}
          <div class="flex items-center justify-center py-8 text-white/30 text-xs">{searchError || t().noResults}</div>
        {/if}
      </div>
    {/if}
  </div>
{/if}
