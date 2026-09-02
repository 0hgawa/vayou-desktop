import { invoke } from "@tauri-apps/api/core";

export interface PlaylistItem {
  index: number;
  filename: string;
  current: boolean;
  title: string;
}

export const playlistAdd = (path: string) => invoke<void>("playlist_add", { path });
export const playlistRemove = (index: number) => invoke<void>("playlist_remove", { index });
export const playlistNext = () => invoke<void>("playlist_next");
export const playlistPrev = () => invoke<void>("playlist_prev");
export const playlistPlayIndex = (index: number) => invoke<void>("playlist_play_index", { index });
export const playlistClear = () => invoke<void>("playlist_clear");
export const getPlaylist = () => invoke<PlaylistItem[]>("get_playlist");
