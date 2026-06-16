import { invoke } from "@tauri-apps/api/core";

// A newer release the backend verified: round-trips back to install_update.
export interface UpdateInfo {
  version: string;
  url: string;
  signature: string;
}

// `null` means we're already on the latest version.
export const checkUpdate = () => invoke<UpdateInfo | null>("check_update");
export const installUpdate = (info: UpdateInfo) => invoke<void>("install_update", { info });
export const relaunchApp = () => invoke<void>("relaunch_app");
export const openReleasePage = () => invoke<void>("open_release_page");
