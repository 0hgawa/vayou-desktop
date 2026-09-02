import { invoke } from "@tauri-apps/api/core";

export const setAudioNormalization = (enabled: boolean) => invoke<void>("set_audio_normalization", { enabled });
export const setAudioEqualizer = (bands: number[]) => invoke<void>("set_audio_equalizer", { bands });
export const resetAudioEqualizer = () => invoke<void>("reset_audio_equalizer");
