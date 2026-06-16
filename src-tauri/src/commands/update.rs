use crate::services::update::{self, UpdateInfo};

/// Check the release feed. `Some` carries the verified update the frontend then
/// hands back to `install_update`; `None` means we're already current.
#[tauri::command]
pub async fn check_update() -> Result<Option<UpdateInfo>, String> {
    update::check().await
}

/// Download the new executable, verify its signature, and swap it in place.
#[tauri::command]
pub async fn install_update(info: UpdateInfo) -> Result<(), String> {
    update::download_and_apply(&info).await
}

/// Relaunch into the just-installed binary.
#[tauri::command]
pub fn relaunch_app(app: tauri::AppHandle) {
    app.restart();
}

/// Open the releases page - fallback when the in-app update can't be applied.
#[tauri::command]
pub fn open_release_page() {
    update::open_release_page();
}
