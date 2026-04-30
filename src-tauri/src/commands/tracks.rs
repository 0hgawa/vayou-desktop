use tauri::State;

use crate::error::AppError;
use crate::services::tracks::{SubStyle, TrackInfo, TracksService};
use crate::state::MpvState;

#[tauri::command]
pub async fn get_tracks(state: State<'_, MpvState>) -> Result<Vec<TrackInfo>, AppError> {
    let mpv = state.get()?;
    Ok(TracksService::get_all(mpv))
}

#[tauri::command]
pub async fn select_subtitle(id: i64, state: State<'_, MpvState>) -> Result<(), AppError> {
    TracksService::select_subtitle(state.get()?, id)?;
    Ok(())
}

#[tauri::command]
pub async fn select_audio_track(id: i64, state: State<'_, MpvState>) -> Result<(), AppError> {
    TracksService::select_audio(state.get()?, id)?;
    Ok(())
}

#[tauri::command]
pub async fn load_subtitle(path: String, state: State<'_, MpvState>) -> Result<(), AppError> {
    TracksService::load_subtitle(state.get()?, &path)?;
    Ok(())
}

#[tauri::command]
pub async fn toggle_subtitles(state: State<'_, MpvState>) -> Result<(), AppError> {
    TracksService::toggle_subtitles(state.get()?)?;
    Ok(())
}

#[tauri::command]
pub async fn set_subtitle_delay(seconds: f64, state: State<'_, MpvState>) -> Result<(), AppError> {
    TracksService::set_subtitle_delay(state.get()?, seconds)?;
    Ok(())
}

#[tauri::command]
pub async fn set_audio_delay(seconds: f64, state: State<'_, MpvState>) -> Result<(), AppError> {
    TracksService::set_audio_delay(state.get()?, seconds)?;
    Ok(())
}

#[tauri::command]
pub async fn set_sub_style(style: SubStyle, state: State<'_, MpvState>) -> Result<(), AppError> {
    TracksService::set_sub_style(state.get()?, &style)?;
    Ok(())
}
