import { invoke } from "@tauri-apps/api/core";

export interface SubResult {
  name: string;
  lang: string;
  download_link: string;
  downloads: string;
  matched_by: string;
}

export const searchSubtitles = (query: string, lang: string) =>
  invoke<SubResult[]>("search_subtitles", { query, lang });

export const downloadSubtitle = (result: SubResult) =>
  invoke<string>("download_subtitle", { result });
