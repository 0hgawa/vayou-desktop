import { invoke } from "@tauri-apps/api/core";

export interface TrackInfo {
  id: number;
  track_type: string;
  title: string;
  lang: string;
  selected: boolean;
}

export const getTracks = () => invoke<TrackInfo[]>("get_tracks");
export const selectSubtitle = (id: number) => invoke<void>("select_subtitle", { id });
export const selectAudioTrack = (id: number) => invoke<void>("select_audio_track", { id });
export const loadSubtitle = (path: string) => invoke<void>("load_subtitle", { path });
export const toggleSubtitles = () => invoke<void>("toggle_subtitles");
export const setSubtitleDelay = (seconds: number) => invoke<void>("set_subtitle_delay", { seconds });
export const setAudioDelay = (seconds: number) => invoke<void>("set_audio_delay", { seconds });

export interface SubStyle {
  font: string;
  size: number;
  color: string;
  border_color: string;
  border_size: number;
  position: number;
}
export const setSubStyle = (style: SubStyle) => invoke<void>("set_sub_style", { style });
