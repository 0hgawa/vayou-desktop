<script lang="ts">
  import { setBrightness, setContrast, setSaturation, setVideoZoom, resetVideoZoomPan, toggleDeinterlace } from "$lib/bindings/video";
  import { setAudioNormalization, setAudioEqualizer, resetAudioEqualizer } from "$lib/bindings/audio-fx";
  import { settings, subFonts } from "$lib/stores/settings.svelte";
  import { keybindings, KeybindingsStore } from "$lib/stores/keybindings.svelte";
  import { t, setLocale } from "$lib/i18n/index.svelte";
  import { ICONS } from "$lib/icons";
  import SettingRow from "./SettingRow.svelte";
  import OptionsDialog from "./OptionsDialog.svelte";

  type Tab = "general" | "video" | "audio" | "subtitles" | "shortcuts";
  type Dialog = null | "language" | "speed" | "font";

  let { visible = $bindable(false) }: { visible: boolean } = $props();
  let tab = $state<Tab>("general");
  let dialog = $state<Dialog>(null);
  let rebinding = $state<string | null>(null);

  // Video state (mirrors mpv, not persisted)
  let brightness = $state(0);
  let contrast = $state(0);
  let saturation = $state(0);
  let zoom = $state(0);
  let deinterlace = $state(false);

  // Audio state (mirrors mpv)
  let normEnabled = $state(false);
  let eqBands = $state([0, 0, 0, 0, 0]);
  const eqLabels = ["60Hz", "230Hz", "910Hz", "3.6kHz", "14kHz"];
  const eqPresets: Record<string, number[]> = {
    Flat: [0, 0, 0, 0, 0],
    Bass: [8, 5, 0, 0, 0],
    Treble: [0, 0, 0, 4, 8],
    Vocal: [-2, 0, 4, 4, 0],
    Rock: [4, 2, -1, 2, 4],
  };

  const speeds = [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0, 3.0, 4.0];
  const languages: Record<string, string> = {
    en: "English", pt: "Português", es: "Español", fr: "Français",
    de: "Deutsch", it: "Italiano", ja: "日本語", ko: "한국어",
    zh: "中文", ru: "Русский", ar: "العربية", hi: "हिन्दी",
  };

  const categories: { id: Tab; get label(): string; icon: string }[] = [
    { id: "general", get label() { return t().general; }, icon: ICONS.tune },
    { id: "video", get label() { return t().video; }, icon: ICONS.movie },
    { id: "audio", get label() { return t().audio; }, icon: ICONS.graphicEq },
    { id: "subtitles", get label() { return t().subtitles; }, icon: ICONS.subtitles },
    { id: "shortcuts", get label() { return t().shortcuts; }, icon: ICONS.keyboard },
  ];

  const languageOptions = $derived(
    Object.entries(languages).map(([value, label]) => ({ value, label }))
  );
  const speedOptions = $derived(
    speeds.map((s) => ({ value: s, label: `${s}x` }))
  );
  const fontOptions = $derived(
    subFonts.map((f) => ({ value: f, label: f, style: `font-family:'${f}'` }))
  );

  function setLang(code: string) {
    settings.language = code;
    setLocale(code);
    settings.save();
  }
  function resetVideo() {
    brightness = 0; contrast = 0; saturation = 0; zoom = 0;
    setBrightness(0); setContrast(0); setSaturation(0); resetVideoZoomPan();
  }
  function applyEq() { setAudioEqualizer(eqBands).catch(() => {}); }
  function resetEq() { eqBands = [0, 0, 0, 0, 0]; resetAudioEqualizer(); }
  function setPreset(name: string) { eqBands = [...eqPresets[name]]; applyEq(); }

  function resetAll() {
    settings.resetAll();
    resetVideo();
    normEnabled = false;
    resetEq();
    setAudioNormalization(false);
  }

  function handleRebind(e: KeyboardEvent) {
    if (!rebinding) return;
    e.preventDefault(); e.stopPropagation();
    if (e.key === "Escape") { rebinding = null; return; }
    const parts: string[] = [];
    if (e.ctrlKey) parts.push("Ctrl");
    if (e.shiftKey) parts.push("Shift");
    if (e.altKey) parts.push("Alt");
    if (!["Control", "Shift", "Alt"].includes(e.key)) parts.push(e.key === " " ? "Space" : e.key);
    if (parts.length === 0) return;
    keybindings.setKey(rebinding, parts.join("+"));
    rebinding = null;
    settings.save();
  }

  function close() { visible = false; }

  const shortcutCategories = $derived([...new Set(keybindings.actions.map((a) => a.category))]);

  // Bipolar slider fill: returns CSS vars for the colored segment from the
  // neutral point (default 0) to the current value.
  function biStyle(v: number, min: number, max: number, neutral = 0): string {
    const pct = (n: number) => ((n - min) / (max - min)) * 100;
    const c = pct(neutral);
    const p = pct(v);
    return `--lo: ${Math.min(c, p)}%; --hi: ${Math.max(c, p)}%`;
  }
</script>

{#if visible}
  <button aria-label="Close" class="fixed inset-0 z-90 w-full h-full bg-black/40 border-none cursor-default" onclick={close}></button>

  <div
    data-panel
    role="dialog"
    tabindex="-1"
    class="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 z-91 w-[620px] h-[min(85vh,460px)] bg-[#18181c]/95 backdrop-blur-md border border-white/10 rounded-xl shadow-2xl text-[13px] text-white/90 flex flex-col select-none overflow-hidden"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.key === "Escape" && close()}
  >
    <!-- Header -->
    <div class="flex items-center justify-between px-4 h-12 shrink-0">
      <span class="font-medium text-sm">{t().settings}</span>
      <button onclick={close} class="ctrl-btn w-7 h-7 text-xs">✕</button>
    </div>

    <div class="flex flex-1 min-h-0">
      <!-- Rail -->
      <nav class="w-[200px] py-2 px-2 shrink-0 overflow-y-auto space-y-0.5">
        {#each categories as cat}
          <button
            onclick={() => (tab = cat.id)}
            class="w-full flex items-center gap-3 px-3 py-2.5 text-[13px] rounded-lg transition-colors hover:bg-white/8 {tab === cat.id ? 'bg-white/5 text-accent' : 'text-white/70 hover:text-white/95'}"
          >
            <svg class="w-5 h-5 shrink-0" fill="currentColor" viewBox="0 0 24 24">{@html cat.icon}</svg>
            <span class="truncate">{cat.label}</span>
          </button>
        {/each}
      </nav>

      <!-- Detail -->
      <div class="flex-1 min-w-0 overflow-y-auto">
        {#if tab === "general"}
          <SettingRow
            icon={ICONS.language}
            title={t().language}
            value={languages[settings.language]}
            onclick={() => (dialog = "language")}
          >
            {#snippet trailing()}
              <svg class="w-4 h-4 text-white/30" fill="currentColor" viewBox="0 0 24 24">{@html ICONS.chevronRight}</svg>
            {/snippet}
          </SettingRow>

          <SettingRow icon={ICONS.volumeUp} title={t().defaultVolume} value="{settings.volume}%">
            {#snippet below()}
              <input
                type="range" min="0" max="100"
                bind:value={settings.volume}
                oninput={() => settings.save()}
                class="s-range w-full"
                style="--val: {settings.volume}%"
              />
            {/snippet}
          </SettingRow>

          <SettingRow
            icon={ICONS.speed}
            title={t().defaultSpeed}
            value="{settings.speed}x"
            onclick={() => (dialog = "speed")}
          >
            {#snippet trailing()}
              <svg class="w-4 h-4 text-white/30" fill="currentColor" viewBox="0 0 24 24">{@html ICONS.chevronRight}</svg>
            {/snippet}
          </SettingRow>

          <SettingRow
            icon={ICONS.history}
            title={t().rememberPosition}
            onclick={() => { settings.rememberPosition = !settings.rememberPosition; settings.save(); }}
          >
            {#snippet trailing()}
              <span class="vayou-switch" class:on={settings.rememberPosition}></span>
            {/snippet}
          </SettingRow>

          <SettingRow
            icon={ICONS.playCircle}
            title={t().autoPlay}
            onclick={() => { settings.autoPlay = !settings.autoPlay; settings.save(); }}
          >
            {#snippet trailing()}
              <span class="vayou-switch" class:on={settings.autoPlay}></span>
            {/snippet}
          </SettingRow>

        {:else if tab === "video"}
          <div class="px-4 pt-4 pb-1 flex items-center justify-between">
            <span class="text-[11px] uppercase tracking-wider text-accent font-medium">{t().color}</span>
            <button class="flex items-center gap-1 text-[11px] text-white/40 hover:text-white/80 transition-colors" onclick={resetVideo} title={t().reset}>
              <svg class="w-3.5 h-3.5" fill="currentColor" viewBox="0 0 24 24">{@html ICONS.restartAlt}</svg>
              {t().reset}
            </button>
          </div>

          {#each [
            { icon: ICONS.brightness, get label() { return t().brightness; }, get: () => brightness, set: (v: number) => { brightness = v; setBrightness(v); } },
            { icon: ICONS.contrast, get label() { return t().contrast; }, get: () => contrast, set: (v: number) => { contrast = v; setContrast(v); } },
            { icon: ICONS.palette, get label() { return t().saturation; }, get: () => saturation, set: (v: number) => { saturation = v; setSaturation(v); } },
          ] as ctrl}
            <SettingRow icon={ctrl.icon} title={ctrl.label} value={String(ctrl.get())}>
              {#snippet below()}
                <input
                  type="range" min="-100" max="100"
                  value={ctrl.get()}
                  oninput={(e) => ctrl.set(Number((e.target as HTMLInputElement).value))}
                  class="s-range-bi w-full"
                  style={biStyle(ctrl.get(), -100, 100)}
                />
              {/snippet}
            </SettingRow>
          {/each}

          <SettingRow icon={ICONS.zoomIn} title={t().zoom} value={zoom.toFixed(1)}>
            {#snippet below()}
              <input
                type="range" min="-1" max="3" step="0.1"
                bind:value={zoom}
                oninput={() => setVideoZoom(zoom)}
                class="s-range-bi w-full"
                style={biStyle(zoom, -1, 3)}
              />
            {/snippet}
          </SettingRow>

          <SettingRow
            icon={ICONS.tune}
            title={t().deinterlace}
            onclick={() => { deinterlace = !deinterlace; toggleDeinterlace(); }}
          >
            {#snippet trailing()}
              <span class="vayou-switch" class:on={deinterlace}></span>
            {/snippet}
          </SettingRow>

        {:else if tab === "audio"}
          <SettingRow
            icon={ICONS.graphicEq}
            title={t().normalization}
            onclick={() => { normEnabled = !normEnabled; setAudioNormalization(normEnabled); }}
          >
            {#snippet trailing()}
              <span class="vayou-switch" class:on={normEnabled}></span>
            {/snippet}
          </SettingRow>

          <div class="px-4 pt-4 pb-2 flex items-center justify-between">
            <span class="text-[11px] uppercase tracking-wider text-accent font-medium">{t().equalizer}</span>
            <button class="flex items-center gap-1 text-[11px] text-white/40 hover:text-white/80 transition-colors" onclick={resetEq} title={t().reset}>
              <svg class="w-3.5 h-3.5" fill="currentColor" viewBox="0 0 24 24">{@html ICONS.restartAlt}</svg>
              {t().reset}
            </button>
          </div>

          <div class="px-4 pb-2 flex gap-1.5 flex-wrap">
            {#each Object.keys(eqPresets) as name}
              <button class="px-2.5 py-1 rounded-md text-xs bg-white/[0.06] hover:bg-white/[0.12] text-white/70 transition-colors" onclick={() => setPreset(name)}>{name}</button>
            {/each}
          </div>

          <div class="px-4 pb-3 space-y-2.5">
            {#each eqLabels as label, i}
              <div class="flex items-center gap-3">
                <span class="w-12 text-xs text-white/55 text-right shrink-0">{label}</span>
                <input
                  type="range" min="-12" max="12" step="1"
                  bind:value={eqBands[i]}
                  oninput={applyEq}
                  class="s-range-bi flex-1"
                  style={biStyle(eqBands[i], -12, 12)}
                />
                <span class="w-7 text-xs text-white/55 text-right tabular-nums shrink-0">{eqBands[i] > 0 ? "+" : ""}{eqBands[i]}</span>
              </div>
            {/each}
          </div>

        {:else if tab === "subtitles"}
          <div class="px-4 pt-4 pb-1 flex items-center justify-between">
            <span class="text-[11px] uppercase tracking-wider text-accent font-medium">{t().style}</span>
            <button class="flex items-center gap-1 text-[11px] text-white/40 hover:text-white/80 transition-colors" onclick={() => settings.resetSubStyle()} title={t().reset}>
              <svg class="w-3.5 h-3.5" fill="currentColor" viewBox="0 0 24 24">{@html ICONS.restartAlt}</svg>
              {t().reset}
            </button>
          </div>

          <SettingRow
            icon={ICONS.textFields}
            title={t().font}
            value={settings.subFont}
            onclick={() => (dialog = "font")}
          >
            {#snippet trailing()}
              <button
                class="w-7 h-7 rounded-md text-sm font-bold transition-all {settings.subBold ? 'bg-accent/20 text-accent' : 'bg-white/[0.06] text-white/60 hover:bg-white/[0.12]'}"
                onclick={(e) => { e.stopPropagation(); settings.subBold = !settings.subBold; settings.applySubStyle(); }}
                title="Bold"
              >B</button>
            {/snippet}
          </SettingRow>

          <SettingRow icon={ICONS.formatSize} title={t().size} value={String(settings.subSize)}>
            {#snippet below()}
              <input
                type="range" min="10" max="100"
                bind:value={settings.subSize}
                oninput={() => settings.applySubStyle()}
                class="s-range w-full"
                style="--val: {((settings.subSize - 10) / 90) * 100}%"
              />
            {/snippet}
          </SettingRow>

          <SettingRow icon={ICONS.formatColorText} title={t().textColor} value={settings.subColor}>
            {#snippet trailing()}
              <input
                type="color"
                bind:value={settings.subColor}
                oninput={() => settings.applySubStyle()}
                class="s-color"
              />
            {/snippet}
          </SettingRow>

          <SettingRow icon={ICONS.borderColor} title={t().borderColor} value={settings.subBorderColor}>
            {#snippet trailing()}
              <input
                type="color"
                bind:value={settings.subBorderColor}
                oninput={() => settings.applySubStyle()}
                class="s-color"
              />
            {/snippet}
          </SettingRow>

          <SettingRow icon={ICONS.lineWeight} title={t().borderSize} value={String(settings.subBorderSize)}>
            {#snippet below()}
              <input
                type="range" min="0" max="5" step="0.5"
                bind:value={settings.subBorderSize}
                oninput={() => settings.applySubStyle()}
                class="s-range w-full"
                style="--val: {(settings.subBorderSize / 5) * 100}%"
              />
            {/snippet}
          </SettingRow>

          <SettingRow icon={ICONS.alignBottom} title={t().position} value="{settings.subPosition}%">
            {#snippet below()}
              <input
                type="range" min="0" max="100"
                bind:value={settings.subPosition}
                oninput={() => settings.applySubStyle()}
                class="s-range w-full"
                style="--val: {settings.subPosition}%"
              />
            {/snippet}
          </SettingRow>

        {:else if tab === "shortcuts"}
          {#each shortcutCategories as category}
            <div class="px-4 pt-4 pb-1">
              <span class="text-[11px] uppercase tracking-wider text-accent font-medium">{(t() as Record<string, string>)[category] ?? category}</span>
            </div>
            {#each keybindings.actions.filter((a) => a.category === category) as action}
              <div class="flex items-center px-4 py-2.5 hover:bg-white/[0.02]">
                <span class="flex-1 text-[13px] text-white/85">{(t() as Record<string, string>)[action.i18nKey] ?? action.i18nKey}</span>
                <button
                  class="min-w-25 px-3 py-1 text-xs rounded-md transition-all text-center {rebinding === action.id ? 'bg-accent/25 text-accent ring-1 ring-accent/40' : 'bg-white/[0.06] text-white/80 hover:bg-white/[0.10]'}"
                  onclick={() => (rebinding = action.id)}
                  onkeydown={rebinding === action.id ? handleRebind : undefined}
                >
                  {rebinding === action.id ? t().pressKey : KeybindingsStore.keyLabel(keybindings.getKey(action.id))}
                </button>
              </div>
            {/each}
          {/each}
          <div class="px-4 py-3">
            <button
              class="w-full py-2 text-xs text-white/55 hover:text-white/85 bg-white/[0.04] rounded-md hover:bg-white/[0.08] transition-colors"
              onclick={() => { keybindings.resetAll(); settings.save(); }}
            >
              {t().resetShortcuts}
            </button>
          </div>
        {/if}
      </div>
    </div>

    <!-- Footer -->
    <div class="flex items-center px-4 h-9 shrink-0">
      <button class="text-[11px] text-white/35 hover:text-white/70 transition-colors" onclick={resetAll}>{t().restoreDefaults}</button>
      <div class="flex-1"></div>
      <span class="text-[11px] text-white/25">Vayou v0.1.0</span>
    </div>
  </div>

  {#if dialog === "language"}
    <OptionsDialog
      title={t().language}
      options={languageOptions}
      selected={settings.language}
      onselect={setLang}
      onclose={() => (dialog = null)}
    />
  {:else if dialog === "speed"}
    <OptionsDialog
      title={t().defaultSpeed}
      options={speedOptions}
      selected={settings.speed}
      onselect={(s) => { settings.speed = s; settings.save(); }}
      onclose={() => (dialog = null)}
    />
  {:else if dialog === "font"}
    <OptionsDialog
      title={t().font}
      options={fontOptions}
      selected={settings.subFont}
      onselect={(f) => { settings.subFont = f; settings.applySubStyle(); }}
      onclose={() => (dialog = null)}
    />
  {/if}
{/if}
