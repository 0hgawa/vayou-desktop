use tauri::State;

use crate::error::AppError;
use crate::services::video::{VideoService, ZoomPanState};
use crate::state::MpvState;

#[tauri::command]
pub async fn set_brightness(value: i64, state: State<'_, MpvState>) -> Result<(), AppError> {
    VideoService::set_brightness(state.get()?, value)?;
    Ok(())
}

#[tauri::command]
pub async fn set_contrast(value: i64, state: State<'_, MpvState>) -> Result<(), AppError> {
    VideoService::set_contrast(state.get()?, value)?;
    Ok(())
}

#[tauri::command]
pub async fn set_saturation(value: i64, state: State<'_, MpvState>) -> Result<(), AppError> {
    VideoService::set_saturation(state.get()?, value)?;
    Ok(())
}

#[tauri::command]
pub async fn toggle_deinterlace(state: State<'_, MpvState>) -> Result<(), AppError> {
    VideoService::toggle_deinterlace(state.get()?)?;
    Ok(())
}

#[tauri::command]
pub async fn set_video_zoom(value: f64, state: State<'_, MpvState>) -> Result<(), AppError> {
    VideoService::set_zoom(state.get()?, value)?;
    Ok(())
}

#[tauri::command]
pub async fn set_video_pan(x: f64, y: f64, state: State<'_, MpvState>) -> Result<(), AppError> {
    VideoService::set_pan(state.get()?, x, y)?;
    Ok(())
}

#[tauri::command]
pub async fn get_video_zoom_pan(state: State<'_, MpvState>) -> Result<ZoomPanState, AppError> {
    Ok(VideoService::get_zoom_pan(state.get()?))
}

#[tauri::command]
pub async fn reset_video_zoom_pan(state: State<'_, MpvState>) -> Result<(), AppError> {
    VideoService::reset_zoom_pan(state.get()?)?;
    Ok(())
}

#[tauri::command]
pub async fn set_aspect_ratio(ratio: String, state: State<'_, MpvState>) -> Result<(), AppError> {
    VideoService::set_aspect_ratio(state.get()?, &ratio)?;
    Ok(())
}

#[tauri::command]
pub async fn get_aspect_ratio(state: State<'_, MpvState>) -> Result<String, AppError> {
    Ok(VideoService::get_aspect_ratio(state.get()?))
}
