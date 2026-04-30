import { invoke } from "@tauri-apps/api/core";

export const translateSubtitles = (targetLang: string) =>
  invoke<string>("translate_subtitles", { targetLang });
