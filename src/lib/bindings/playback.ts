import { invoke } from "@tauri-apps/api/core";

export const initPlayer = () => invoke<void>("init_player");
export const openFile = (path: string) => invoke<void>("open_file", { path });
export const togglePause = () => invoke<void>("toggle_pause");
export const play = () => invoke<void>("play");
export const pause = () => invoke<void>("pause");
export const stop = () => invoke<void>("stop");
export const seekRelative = (seconds: number) => invoke<void>("seek_relative", { seconds });
export const seekAbsolute = (seconds: number) => invoke<void>("seek_absolute", { seconds });
export const setVolume = (volume: number) => invoke<void>("set_volume", { volume });
export const setSpeed = (speed: number) => invoke<void>("set_speed", { speed });

export interface PlaybackState {
  time_pos: number;
  duration: number;
  paused: boolean;
  title: string;
  volume: number;
}

export const getPlaybackState = () => invoke<PlaybackState>("get_playback_state");

export const screenshot = () => invoke<string>("screenshot");
export const frameStep = () => invoke<void>("frame_step");
export const frameBackStep = () => invoke<void>("frame_back_step");
export const openUrl = (url: string) => invoke<void>("open_url", { url });

export interface AbLoopState {
  a: number;
  b: number;
  active: boolean;
}
export const toggleAbLoop = () => invoke<AbLoopState>("toggle_ab_loop");

export interface Chapter {
  index: number;
  title: string;
  time: number;
  current: boolean;
}
export const getChapters = () => invoke<Chapter[]>("get_chapters");
export const seekChapter = (index: number) => invoke<void>("seek_chapter", { index });
