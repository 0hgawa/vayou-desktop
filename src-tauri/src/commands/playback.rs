use tauri::{Manager, State};

use crate::error::AppError;
use crate::mpv::events;
use crate::mpv::player::MpvPlayer;
use crate::mpv::types::*;
use crate::services::playback::{PlaybackService, PlaybackState};
use crate::services::playlist::PlaylistService;
use crate::state::{set_pending_resume, AppState, MpvState};

#[tauri::command]
pub async fn init_player(
    app: tauri::AppHandle,
    mpv_state: State<'_, MpvState>,
) -> Result<(), AppError> {
    // Skip if already initialized (e.g. HMR re-render)
    if mpv_state.is_initialized() {
        return Ok(());
    }

    let window = app
        .get_webview_window("main")
        .ok_or_else(|| AppError::FileNotFound("main window not found".into()))?;

    #[cfg(target_os = "windows")]
    let hwnd_val = {
        let hwnd = window
            .hwnd()
            .map_err(|e| AppError::Config(format!("HWND: {e}")))?;
        hwnd.0 as i64
    };

    #[cfg(not(target_os = "windows"))]
    let hwnd_val: i64 = {
        let _ = window;
        return Err(AppError::Config("Only Windows is supported".into()));
    };

    let mpv = MpvPlayer::new(hwnd_val)?;

    // Observe properties for event-driven frontend updates
    mpv.observe_property("time-pos", 1, MPV_FORMAT_DOUBLE)?;
    mpv.observe_property("duration", 2, MPV_FORMAT_DOUBLE)?;
    mpv.observe_property("pause", 3, MPV_FORMAT_FLAG)?;
    mpv.observe_property("volume", 4, MPV_FORMAT_DOUBLE)?;
    mpv.observe_property("media-title", 5, MPV_FORMAT_STRING)?;

    mpv_state.init(mpv)?;

    // Start event loop with Arc from state
    let mpv_arc = mpv_state.get()?.clone();
    events::start_event_loop(mpv_arc, app.clone());

    Ok(())
}

#[tauri::command]
pub async fn open_file(
    path: String,
    mpv_state: State<'_, MpvState>,
    app_state: State<'_, AppState>,
) -> Result<(), AppError> {
    let is_url = path.starts_with("http://") || path.starts_with("https://");
    if !is_url && !std::path::Path::new(&path).exists() {
        return Err(AppError::FileNotFound(path));
    }
    let mpv = mpv_state.get()?;

    // Save position of current file before switching
    app_state.with(|settings, current_file| {
        if let Some(prev) = current_file.clone() {
            let pos = mpv.get::<f64>("time-pos").unwrap_or(0.0);
            let title = mpv.get_property_string("media-title").unwrap_or_default();
            settings.touch_recent(&prev, &title, pos);
        }
    })?;

    // Check for resume position
    let resume = app_state.with(|settings, _| {
        if settings.remember_position {
            settings.get_saved_position(&path)
        } else {
            None
        }
    })?;

    // Queue resume position (consumed by event loop on FILE_LOADED)
    if let Some(pos) = resume {
        set_pending_resume(pos);
    }

    // Load file (+ populate playlist with sibling media files for local files)
    if is_url {
        mpv.command(&["loadfile", &path, "replace"])?;
    } else {
        PlaylistService::open_with_siblings(mpv, &path)?;
    }

    // Update state + recent files
    let title = std::path::Path::new(&path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    app_state.with(|settings, current_file| {
        *current_file = Some(path.clone());
        settings.touch_recent(&path, &title, 0.0);
        settings.save().ok();
    })?;

    Ok(())
}

#[tauri::command]
pub async fn toggle_pause(state: State<'_, MpvState>) -> Result<(), AppError> {
    PlaybackService::toggle_pause(state.get()?)?;
    Ok(())
}

#[tauri::command]
pub async fn play(state: State<'_, MpvState>) -> Result<(), AppError> {
    PlaybackService::play(state.get()?)?;
    Ok(())
}

#[tauri::command]
pub async fn pause(state: State<'_, MpvState>) -> Result<(), AppError> {
    PlaybackService::pause(state.get()?)?;
    Ok(())
}

#[tauri::command]
pub async fn stop(state: State<'_, MpvState>) -> Result<(), AppError> {
    PlaybackService::stop(state.get()?)?;
    Ok(())
}

#[tauri::command]
pub async fn seek_relative(seconds: f64, state: State<'_, MpvState>) -> Result<(), AppError> {
    PlaybackService::seek_relative(state.get()?, seconds)?;
    Ok(())
}

#[tauri::command]
pub async fn seek_absolute(seconds: f64, state: State<'_, MpvState>) -> Result<(), AppError> {
    PlaybackService::seek_absolute(state.get()?, seconds)?;
    Ok(())
}



#[tauri::command]
pub async fn set_volume(volume: f64, state: State<'_, MpvState>) -> Result<(), AppError> {
    PlaybackService::set_volume(state.get()?, volume)?;
    Ok(())
}

#[tauri::command]
pub async fn set_speed(speed: f64, state: State<'_, MpvState>) -> Result<(), AppError> {
    PlaybackService::set_speed(state.get()?, speed)?;
    Ok(())
}

#[tauri::command]
pub async fn get_playback_state(state: State<'_, MpvState>) -> Result<PlaybackState, AppError> {
    let mpv = state.get()?;
    Ok(PlaybackService::get_state(mpv))
}

#[tauri::command]
pub async fn screenshot(state: State<'_, MpvState>) -> Result<String, AppError> {
    let dir = dirs::picture_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_default())
        .join("Vayou");
    std::fs::create_dir_all(&dir)?;
    let name = chrono::Local::now().format("vayou_%Y%m%d_%H%M%S.png");
    let path = dir.join(name.to_string());
    let path_str = path.to_string_lossy().to_string();
    PlaybackService::screenshot(state.get()?, &path_str)?;
    Ok(path_str)
}

#[tauri::command]
pub async fn frame_step(state: State<'_, MpvState>) -> Result<(), AppError> {
    PlaybackService::frame_step(state.get()?)?;
    Ok(())
}

#[tauri::command]
pub async fn frame_back_step(state: State<'_, MpvState>) -> Result<(), AppError> {
    PlaybackService::frame_back_step(state.get()?)?;
    Ok(())
}

#[tauri::command]
pub async fn toggle_ab_loop(
    state: State<'_, MpvState>,
) -> Result<crate::services::playback::AbLoopState, AppError> {
    Ok(PlaybackService::set_ab_loop(state.get()?, "toggle")?)
}

#[tauri::command]
pub async fn get_chapters(
    state: State<'_, MpvState>,
) -> Result<Vec<crate::services::playback::Chapter>, AppError> {
    Ok(PlaybackService::get_chapters(state.get()?))
}

#[tauri::command]
pub async fn seek_chapter(index: i64, state: State<'_, MpvState>) -> Result<(), AppError> {
    PlaybackService::seek_chapter(state.get()?, index)?;
    Ok(())
}

#[tauri::command]
pub async fn open_url(url: String, state: State<'_, MpvState>) -> Result<(), AppError> {
    PlaybackService::open_url(state.get()?, &url)?;
    Ok(())
}

#[tauri::command]
pub async fn set_mpv_property(name: String, value: String, state: State<'_, MpvState>) -> Result<(), AppError> {
    state.get()?.set::<&str>(&name, &value)?;
    Ok(())
}

#[tauri::command]
pub async fn mpv_command(args: Vec<String>, state: State<'_, MpvState>) -> Result<(), AppError> {
    let strs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    state.get()?.command(&strs)?;
    Ok(())
}
