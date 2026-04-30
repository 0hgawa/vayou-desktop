use std::path::PathBuf;

use tauri::State;

use crate::error::AppError;
use crate::services::opensubtitles::{self, SubResult};
use crate::state::{AppState, MpvState};

#[tauri::command]
pub async fn search_subtitles(
    query: String,
    lang: String,
    app_state: State<'_, AppState>,
) -> Result<Vec<SubResult>, AppError> {
    let video_path = app_state.with(|_, f| f.clone())?;

    // Hashing reads two 64KB chunks — keep it off the async reactor.
    let file_hash = match video_path {
        Some(path) => tokio::task::spawn_blocking(move || opensubtitles::compute_hash(&path).ok())
            .await
            .ok()
            .flatten(),
        None => None,
    };

    opensubtitles::search(file_hash, &query, &lang)
        .await
        .map_err(AppError::Config)
}

#[tauri::command]
pub async fn download_subtitle(
    result: SubResult,
    mpv_state: State<'_, MpvState>,
) -> Result<String, AppError> {
    if result.download_link.is_empty() {
        return Err(AppError::Config("Missing download link".into()));
    }

    let dir = subtitle_cache_dir();
    let saved = opensubtitles::download(&result.download_link, &dir, &result.name)
        .await
        .map_err(AppError::Config)?;
    let saved_str = saved.to_string_lossy().to_string();

    mpv_state.get()?.command(&["sub-add", &saved_str, "select"])?;
    Ok(saved_str)
}

fn subtitle_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .or_else(dirs::data_local_dir)
        .unwrap_or_else(std::env::temp_dir)
        .join("Vayou")
        .join("subtitles")
}
