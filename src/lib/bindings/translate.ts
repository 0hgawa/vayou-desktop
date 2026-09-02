import { invoke } from "@tauri-apps/api/core";

export const translateSubtitles = (targetLang: string) =>
  invoke<string>("translate_subtitles", { targetLang });

export const clearTranslation = () => invoke<void>("clear_translation");
export const cancelTranslation = () => invoke<void>("cancel_translation");
