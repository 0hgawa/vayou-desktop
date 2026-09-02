import { listen } from "@tauri-apps/api/event";
import { translateSubtitles, clearTranslation, cancelTranslation } from "$lib/bindings/translate";
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
  /** Which half of the job is running. Extraction reports no progress and is
   * the long one on a large file, so without this the panel showed "0%" for
   * minutes and read as frozen. */
  phase = $state<"extracting" | "translating">("extracting");
  error = $state("");
  /** Player.title of the file currently translated. Used to gate the auto-
   * translate effect ("don't re-run for the same file"). */
  translatedForPath = $state<string | null>(null);
  /** External filename of the loaded translation track — exposed so the
   * subtitle list can hide it from the source-track dropdown. */
  translationTrackPath = $state<string | null>(null);

  async translate(): Promise<void> {
    if (settings.translateLang === "off") return;
    // Picking another language mid-run used to be silently dropped, which
    // meant a two-minute extraction had to finish before the choice took
    // effect. Stand the old run down and start the new one instead.
    if (this.translating) await this.cancel();
    this.translating = true;
    this.progress = 0;
    this.total = 0;
    this.phase = "extracting";
    this.error = "";
    const unlisten = await listen<{ current: number; total: number; done: boolean; phase: "extracting" | "translating" }>(
      "translate:progress",
      (e) => {
        this.progress = e.payload.current;
        this.total = e.payload.total;
        this.phase = e.payload.phase;
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

  /** Stand down whatever is running. The backend keeps reading until ffmpeg
   * finishes — the result still reaches the cache — but nothing is applied,
   * and the panel is usable again straight away. */
  async cancel(): Promise<void> {
    await cancelTranslation().catch(() => {});
    this.translating = false;
  }

  async clear(): Promise<void> {
    await this.cancel();
    settings.translateLang = "off";
    settings.save();
    await clearTranslation().catch(() => {});
    this.translatedForPath = null;
    this.translationTrackPath = null;
  }
}

export const translate = new TranslateStore();
