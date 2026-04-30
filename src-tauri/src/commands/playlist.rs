use tauri::State;

use crate::error::AppError;
use crate::services::playlist::{PlaylistItem, PlaylistService};
use crate::state::MpvState;

#[tauri::command]
pub async fn playlist_add(path: String, state: State<'_, MpvState>) -> Result<(), AppError> {
    if !std::path::Path::new(&path).exists() {
        return Err(AppError::FileNotFound(path));
    }
    PlaylistService::add(state.get()?, &path)?;
    Ok(())
}

#[tauri::command]
pub async fn playlist_remove(index: i64, state: State<'_, MpvState>) -> Result<(), AppError> {
    PlaylistService::remove(state.get()?, index)?;
    Ok(())
}

#[tauri::command]
pub async fn playlist_next(state: State<'_, MpvState>) -> Result<(), AppError> {
    PlaylistService::next(state.get()?)?;
    Ok(())
}

#[tauri::command]
pub async fn playlist_prev(state: State<'_, MpvState>) -> Result<(), AppError> {
    PlaylistService::prev(state.get()?)?;
    Ok(())
}

#[tauri::command]
pub async fn playlist_play_index(index: i64, state: State<'_, MpvState>) -> Result<(), AppError> {
    PlaylistService::play_index(state.get()?, index)?;
    Ok(())
}

#[tauri::command]
pub async fn playlist_clear(state: State<'_, MpvState>) -> Result<(), AppError> {
    PlaylistService::clear(state.get()?)?;
    Ok(())
}

#[tauri::command]
pub async fn get_playlist(state: State<'_, MpvState>) -> Result<Vec<PlaylistItem>, AppError> {
    Ok(PlaylistService::get_all(state.get()?))
}
