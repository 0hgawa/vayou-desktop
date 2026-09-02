use tauri::State;

use crate::error::AppError;
use crate::services::settings::{PlayerSettings, RecentFile};
use crate::state::AppState;

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<PlayerSettings, AppError> {
    state.with(|settings, _| settings.clone())
}

#[tauri::command]
pub fn save_settings(new_settings: PlayerSettings, state: State<'_, AppState>) -> Result<(), AppError> {
    state.with(|settings, _| {
        *settings = new_settings;
    })?;
    state.with(|settings, _| settings.save())??;
    Ok(())
}

#[tauri::command]
pub fn get_recent_files(state: State<'_, AppState>) -> Result<Vec<RecentFile>, AppError> {
    state.with(|settings, _| settings.recent_files.clone())
}

#[tauri::command]
pub fn clear_recent_files(state: State<'_, AppState>) -> Result<(), AppError> {
    state.with(|settings, _| {
        settings.recent_files.clear();
    })?;
    state.with(|settings, _| settings.save())??;
    Ok(())
}
