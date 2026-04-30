import { invoke } from "@tauri-apps/api/core";

export interface MediaInfo {
  filename: string;
  path: string;
  duration: number;
  width: number;
  height: number;
  video_codec: string;
  audio_codec: string;
  video_bitrate: number;
  audio_bitrate: number;
  fps: number;
  file_size: number;
}

export const getMediaInfo = () => invoke<MediaInfo>("get_media_info");
