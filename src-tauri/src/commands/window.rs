use tauri::Manager;

use crate::error::AppError;

#[tauri::command]
pub async fn toggle_fullscreen(app: tauri::AppHandle) -> Result<(), AppError> {
    if let Some(window) = app.get_webview_window("main") {
        let is_fs = window
            .is_fullscreen()
            .map_err(|e| AppError::Config(e.to_string()))?;
        if !is_fs && window.is_maximized().unwrap_or(false) {
            let _ = window.unmaximize();
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
        window
            .set_fullscreen(!is_fs)
            .map_err(|e| AppError::Config(e.to_string()))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn set_always_on_top(app: tauri::AppHandle, enabled: bool) -> Result<(), AppError> {
    if let Some(window) = app.get_webview_window("main") {
        window
            .set_always_on_top(enabled)
            .map_err(|e| AppError::Config(e.to_string()))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn minimize_window(app: tauri::AppHandle) -> Result<(), AppError> {
    if let Some(window) = app.get_webview_window("main") {
        window
            .minimize()
            .map_err(|e| AppError::Config(e.to_string()))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn maximize_window(app: tauri::AppHandle) -> Result<(), AppError> {
    if let Some(window) = app.get_webview_window("main") {
        let is_max = window
            .is_maximized()
            .map_err(|e| AppError::Config(e.to_string()))?;
        if is_max {
            window.unmaximize()
        } else {
            window.maximize()
        }
        .map_err(|e| AppError::Config(e.to_string()))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn close_window(app: tauri::AppHandle) -> Result<(), AppError> {
    app.exit(0);
    Ok(())
}
