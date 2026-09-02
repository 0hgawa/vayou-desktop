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
        v.filter(|t| *t >= 0.0).map_or(-1, |t| (t * 1000.0) as i64)
    }
    fn from_ms(v: i64) -> Option<f64> {
        if v >= 0 { Some(v as f64 / 1000.0) } else { None }
    }
    fn now_ms() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64)
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

    /// The pure loop predicate: is `pos` far enough into a valid `[a, b]`
    /// range to warrant seeking back to A? Split out of `check` so the range
    /// and look-ahead rules are testable without the atomics or the clock.
    fn should_loop(pos: f64, a: f64, b: f64) -> bool {
        b - a >= MIN_RANGE_S && pos >= b - LOOK_AHEAD_S
    }

    /// Returns Some(target) if the event loop should seek to that position now.
    /// Hot path — called on every time-pos tick. Lock-free.
    pub fn check(pos: f64) -> Option<f64> {
        let a = get_a()?;
        let b = get_b()?;
        if !should_loop(pos, a, b) { return None; }

        let now = now_ms();
        let last = LAST_SEEK_MS.load(Ordering::Relaxed);
        if now.saturating_sub(last) < SEEK_COOLDOWN_MS { return None; }
        LAST_SEEK_MS.store(now, Ordering::Relaxed);
        Some(a)
    }

    #[cfg(test)]
    mod tests {
        use super::{from_ms, should_loop, to_ms};

        #[test]
        #[allow(clippy::float_cmp)] // exact binary values, no arithmetic
        fn to_from_ms_round_trip() {
            assert_eq!(to_ms(Some(1.5)), 1500);
            assert_eq!(from_ms(1500), Some(1.5));
            assert_eq!(to_ms(None), -1);
            assert_eq!(from_ms(-1), None);
            // A negative time means "unset", not "before zero".
            assert_eq!(to_ms(Some(-2.0)), -1);
        }

        #[test]
        fn loops_once_past_b_within_look_ahead() {
            // B = 5.0 and LOOK_AHEAD_S = 0.05, so the trigger point is 4.95 —
            // we fire slightly early so the seek lands before overshooting B.
            assert!(should_loop(4.96, 0.0, 5.0));
            assert!(should_loop(5.0, 0.0, 5.0));
            assert!(should_loop(6.0, 0.0, 5.0));
            // Comfortably before the window — playback continues.
            assert!(!should_loop(4.9, 0.0, 5.0));
            assert!(!should_loop(0.0, 0.0, 5.0));
        }

        #[test]
        fn rejects_ranges_shorter_than_minimum() {
            // MIN_RANGE_S = 0.1. A tighter loop would seek on every tick and
            // freeze the picture, so it is refused outright.
            assert!(!should_loop(10.0, 1.0, 1.05));
            assert!(should_loop(10.0, 1.0, 1.2));
        }
    }
}

/// Lock-free access to the mpv player instance.
pub struct MpvState(OnceLock<Arc<MpvPlayer>>);

impl MpvState {
    pub const fn new() -> Self {
        Self(OnceLock::new())
    }

    pub fn init(&self, player: MpvPlayer) {
        let _ = self.0.set(Arc::new(player));
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
