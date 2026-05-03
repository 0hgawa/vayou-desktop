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

/// AB-loop state & enforcement.
///
/// The mpv built-in `ab-loop` is bypassed because of two upstream bugs:
///   • #7596 — `mpv_set_property(ab-loop-a, MPV_FORMAT_DOUBLE)` clamps positive
///     values to 0 due to a missing M_OPT_MIN flag.
///   • #10640 — the native `ab-loop` command pauses on certain MP4 containers
///     instead of looping cleanly.
///
/// We keep endpoints in lock-free atomics and the event loop seeks back to A
/// when `time-pos` crosses B (with a small look-ahead and cooldown).
pub mod ab_loop {
    use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    /// Trigger seek slightly before B so we don't visually overshoot
    /// (time-pos ticks arrive every ~40ms on 25fps content).
    const LOOK_AHEAD_S: f64 = 0.05;
    /// Reject loops shorter than this — protects against pathological tight
    /// loops that would just freeze the player.
    const MIN_RANGE_S: f64 = 0.1;
    /// After firing a seek, ignore further triggers for this long. Absorbs
    /// stale time-pos ticks that arrive between the seek call and mpv's
    /// internal position update.
    const SEEK_COOLDOWN_MS: u64 = 250;

    static A_MS: AtomicI64 = AtomicI64::new(-1);
    static B_MS: AtomicI64 = AtomicI64::new(-1);
    static LAST_SEEK_MS: AtomicU64 = AtomicU64::new(0);

    fn to_ms(v: Option<f64>) -> i64 {
        v.filter(|t| *t >= 0.0).map(|t| (t * 1000.0) as i64).unwrap_or(-1)
    }
    fn from_ms(v: i64) -> Option<f64> {
        if v >= 0 { Some(v as f64 / 1000.0) } else { None }
    }
    fn now_ms() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
    }

    pub fn set_a(time: Option<f64>) {
        A_MS.store(to_ms(time), Ordering::Relaxed);
        LAST_SEEK_MS.store(0, Ordering::Relaxed);
    }
    pub fn set_b(time: Option<f64>) {
        B_MS.store(to_ms(time), Ordering::Relaxed);
        LAST_SEEK_MS.store(0, Ordering::Relaxed);
    }
    pub fn get_a() -> Option<f64> { from_ms(A_MS.load(Ordering::Relaxed)) }
    pub fn get_b() -> Option<f64> { from_ms(B_MS.load(Ordering::Relaxed)) }
    pub fn clear() {
        A_MS.store(-1, Ordering::Relaxed);
        B_MS.store(-1, Ordering::Relaxed);
        LAST_SEEK_MS.store(0, Ordering::Relaxed);
    }
    /// True only when both endpoints are set. Cheap (2 atomic loads) — used to
    /// gate the time-pos read in the event loop hot path.
    pub fn is_armed() -> bool {
        A_MS.load(Ordering::Relaxed) >= 0 && B_MS.load(Ordering::Relaxed) >= 0
    }

    /// Returns Some(target) if the event loop should seek to that position now.
    /// Hot path — called on every time-pos tick. Lock-free.
    pub fn check(pos: f64) -> Option<f64> {
        let a = get_a()?;
        let b = get_b()?;
        if b - a < MIN_RANGE_S { return None; }
        if pos < b - LOOK_AHEAD_S { return None; }

        let now = now_ms();
        let last = LAST_SEEK_MS.load(Ordering::Relaxed);
        if now.saturating_sub(last) < SEEK_COOLDOWN_MS { return None; }
        LAST_SEEK_MS.store(now, Ordering::Relaxed);
        Some(a)
    }
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
