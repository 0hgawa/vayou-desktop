use tauri::State;

use crate::error::AppError;
use crate::services::audio_fx::AudioFxService;
use crate::state::MpvState;

#[tauri::command]
pub async fn set_audio_normalization(
    enabled: bool,
    state: State<'_, MpvState>,
) -> Result<(), AppError> {
    AudioFxService::set_normalization(state.get()?, enabled)?;
    Ok(())
}

#[tauri::command]
pub async fn set_audio_equalizer(
    bands: [f64; 5],
    state: State<'_, MpvState>,
) -> Result<(), AppError> {
    AudioFxService::set_equalizer(state.get()?, &bands)?;
    Ok(())
}

#[tauri::command]
pub async fn reset_audio_equalizer(state: State<'_, MpvState>) -> Result<(), AppError> {
    AudioFxService::reset_equalizer(state.get()?)?;
    Ok(())
}
