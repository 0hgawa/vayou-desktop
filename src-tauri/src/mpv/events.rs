use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::Arc;
use std::time::Instant;

use tauri::{Emitter, Manager};
use tracing::{debug, info, warn};

use super::player::MpvPlayer;
use super::types::*;
use crate::state::{self, take_pending_resume, AppState};

/// Spawn a named background thread that polls mpv events and emits them
/// as typed Tauri events to the frontend.
pub fn start_event_loop(mpv: Arc<MpvPlayer>, app: tauri::AppHandle) {
    std::thread::Builder::new()
        .name("mpv-events".into())
        .spawn(move || {
            info!("Event loop started");
            run_loop(&mpv, &app);
            info!("Event loop ended");
        })
        .expect("Failed to spawn mpv event loop thread");
}

fn run_loop(mpv: &MpvPlayer, app: &tauri::AppHandle) {
    let mut last_save = Instant::now();

    loop {
        let evt = mpv.wait_event(0.05);

        match evt.event_id {
            MPV_EVENT_PROPERTY_CHANGE => {
                if evt.data.is_null() {
                    continue;
                }
                if let Err(e) = handle_property_change(evt, app) {
                    warn!(error = %e, "Failed to handle property change");
                }
            }

            MPV_EVENT_FILE_LOADED => {
                debug!("File loaded");
                state::ab_loop::clear();
                if let Some(pos) = take_pending_resume() {
                    let _ = mpv.command(&["seek", &pos.to_string(), "absolute"]);
                }
                restore_saved_tracks(mpv, app);
                let _ = app.emit("mpv:file-loaded", ());
            }

            MPV_EVENT_END_FILE => {
                debug!("End of file");
                save_position(mpv, app);
                let _ = app.emit("mpv:end-file", ());
            }

            MPV_EVENT_SHUTDOWN => {
                info!("mpv shutdown event");
                save_position(mpv, app);
                break;
            }

            _ => {}
        }

        // AB loop enforcement runs on every loop iteration (~20Hz), not on
        // time-pos events — those can be coalesced/throttled by mpv to as low
        // as 1Hz on some containers, which would let the playhead overshoot B
        // by seconds before triggering.
        enforce_ab_loop(mpv);

        // Save position every 30 seconds
        if last_save.elapsed().as_secs() >= 30 {
            save_position(mpv, app);
            last_save = Instant::now();
        }
    }
}

/// Manual A-B loop enforcement. Cheap when not armed (2 atomic loads).
/// When armed, polls time-pos directly from mpv and seeks if past B.
fn enforce_ab_loop(mpv: &MpvPlayer) {
    if !state::ab_loop::is_armed() { return; }
    let Ok(pos) = mpv.get::<f64>("time-pos") else { return };
    if let Some(target) = state::ab_loop::check(pos) {
        let _ = mpv.command(&["seek", &target.to_string(), "absolute+exact"]);
        debug!(target, "ab-loop: seek back to A");
    }
}

fn restore_saved_tracks(mpv: &MpvPlayer, app: &tauri::AppHandle) {
    let Some(state) = app.try_state::<AppState>() else { return };
    let _ = state.with(|settings, current_file| {
        if !settings.remember_selections { return; }
        let Some(path) = current_file.as_ref() else { return };
        let (audio, sub) = settings.get_saved_tracks(path);
        if let Some(id) = audio {
            let _ = mpv.set::<&str>("aid", &id.to_string());
        }
        if let Some(id) = sub {
            let _ = mpv.set::<&str>("sid", &id.to_string());
        }
    });
}

fn save_position(mpv: &MpvPlayer, app: &tauri::AppHandle) {
    let pos = mpv.get::<f64>("time-pos").unwrap_or(0.0);
    if pos <= 1.0 { return; }

    if let Some(state) = app.try_state::<AppState>() {
        let _ = state.with(|settings: &mut crate::services::settings::PlayerSettings, current_file: &mut Option<String>| {
            if let Some(path) = current_file.as_ref() {
                let title = mpv.get_property_string("filename").unwrap_or_default();
                settings.touch_recent(path, &title, pos);
                settings.save().ok();
            }
        });
    }
}

fn handle_property_change(evt: &MpvEvent, app: &tauri::AppHandle) -> Result<(), String> {
    let prop = unsafe { &*(evt.data as *const MpvEventProperty) };

    if prop.name.is_null() || prop.data.is_null() {
        return Ok(()); // Property value unavailable (e.g. during init)
    }

    let name = unsafe {
        CStr::from_ptr(prop.name)
            .to_str()
            .map_err(|e| e.to_string())?
    };

    unsafe {
        match (name, prop.format) {
            ("time-pos", MPV_FORMAT_DOUBLE) => {
                let val = *(prop.data as *const f64);
                let _ = app.emit("mpv:time-pos", val);
            }
            ("duration", MPV_FORMAT_DOUBLE) => {
                let val = *(prop.data as *const f64);
                let _ = app.emit("mpv:duration", val);
            }
            ("pause", MPV_FORMAT_FLAG) => {
                let val = *(prop.data as *const i32) != 0;
                let _ = app.emit("mpv:pause", val);
            }
            ("volume", MPV_FORMAT_DOUBLE) => {
                let val = *(prop.data as *const f64);
                let _ = app.emit("mpv:volume", val);
            }
            ("media-title", MPV_FORMAT_STRING) => {
                let ptr = *(prop.data as *const *const c_char);
                if !ptr.is_null() {
                    if let Ok(s) = CStr::from_ptr(ptr).to_str() {
                        let _ = app.emit("mpv:media-title", s.to_owned());
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}
