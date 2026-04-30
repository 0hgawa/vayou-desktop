use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use crate::error::{AppError, MpvError};
use crate::mpv::player::MpvPlayer;
use crate::services::settings::PlayerSettings;

/// Pending resume position (stored as millseconds × 1000 to avoid f64 atomics).
/// Set by open_file, consumed by event loop on FILE_LOADED.
static PENDING_RESUME: AtomicI64 = AtomicI64::new(-1);

pub fn set_pending_resume(pos: f64) {
    PENDING_RESUME.store((pos * 1000.0) as i64, Ordering::Relaxed);
}

pub fn take_pending_resume() -> Option<f64> {
    let v = PENDING_RESUME.swap(-1, Ordering::Relaxed);
    if v >= 0 { Some(v as f64 / 1000.0) } else { None }
}

/// Lock-free access to the mpv player instance.
pub struct MpvState(OnceLock<Arc<MpvPlayer>>);

impl MpvState {
    pub fn new() -> Self {
        Self(OnceLock::new())
    }

    pub fn init(&self, player: MpvPlayer) -> Result<(), MpvError> {
        let _ = self.0.set(Arc::new(player));
        Ok(())
    }

    pub fn is_initialized(&self) -> bool {
        self.0.get().is_some()
    }

    pub fn get(&self) -> Result<&Arc<MpvPlayer>, MpvError> {
        self.0.get().ok_or(MpvError::NotInitialized)
    }
}

/// Mutable app state (settings, current file). Only for non-hot-path data.
pub struct AppState {
    inner: Mutex<AppStateInner>,
}

struct AppStateInner {
    pub settings: PlayerSettings,
    pub current_file: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(AppStateInner {
                settings: PlayerSettings::load(),
                current_file: None,
            }),
        }
    }

    pub fn with<F, R>(&self, f: F) -> Result<R, AppError>
    where
        F: FnOnce(&mut PlayerSettings, &mut Option<String>) -> R,
    {
        let mut guard = self.inner.lock().map_err(|e| AppError::Config(e.to_string()))?;
        let inner = &mut *guard;
        Ok(f(&mut inner.settings, &mut inner.current_file))
    }
}
