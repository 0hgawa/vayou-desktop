use tauri::State;

use crate::error::AppError;
use crate::services::media_info::{MediaInfo, MediaInfoService};
use crate::state::MpvState;

#[tauri::command]
pub async fn get_media_info(state: State<'_, MpvState>) -> Result<MediaInfo, AppError> {
    Ok(MediaInfoService::get(state.get()?))
}
