import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { player } from "$lib/stores/player.svelte";

export async function toggleFullscreen(): Promise<void> {
  player.controlsVisible = false;
  await new Promise((r) => requestAnimationFrame(r));
  await invoke("toggle_fullscreen");
  player.fullscreen = await getCurrentWebviewWindow().isFullscreen();
}

export const setAlwaysOnTop = (enabled: boolean) => invoke<void>("set_always_on_top", { enabled });
export const minimizeWindow = () => invoke<void>("minimize_window");
export const maximizeWindow = () => invoke<void>("maximize_window");
export const closeWindow = () => invoke<void>("close_window");
