import { listen } from "@tauri-apps/api/event";
import { translateSubtitles, clearTranslation } from "$lib/bindings/translate";
import { player } from "./player.svelte";
import { settings } from "./settings.svelte";

/**
 * Global translation state. Lives outside `SubtitlePanel` so the auto-
 * translate effect (registered in +page.svelte) keeps working even when
 * the panel is closed, and the panel's open/close cycle no longer wipes
 * `translatedForPath` / progress state.
 */
class TranslateStore {
  translating = $state(false);
  progress = $state(0);
  total = $state(0);
  error = $state("");
  /** Player.title of the file currently translated. Used to gate the auto-
   * translate effect ("don't re-run for the same file"). */
  translatedForPath = $state<string | null>(null);
  /** External filename of the loaded translation track — exposed so the
   * subtitle list can hide it from the source-track dropdown. */
  translationTrackPath = $state<string | null>(null);

  async translate(): Promise<void> {
    if (settings.translateLang === "off") return;
    if (this.translating) return;
    this.translating = true;
    this.progress = 0;
    this.total = 0;
    this.error = "";
    const unlisten = await listen<{ current: number; total: number; done: boolean }>(
      "translate:progress",
      (e) => {
        this.progress = e.payload.current;
        this.total = e.payload.total;
        if (e.payload.done) this.translating = false;
      },
    );
    try {
      const path = await translateSubtitles(settings.translateLang);
      this.translatedForPath = player.title || "";
      this.translationTrackPath = path;
    } catch (e: any) {
      this.error = String(e);
      this.translating = false;
    }
    unlisten();
  }

  async clear(): Promise<void> {
    settings.translateLang = "off";
    settings.save();
    await clearTranslation().catch(() => {});
    this.translatedForPath = null;
    this.translationTrackPath = null;
  }
}

export const translate = new TranslateStore();
