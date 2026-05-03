use serde::Serialize;
use tracing::info;

use crate::error::MpvError;
use crate::mpv::player::MpvPlayer;
use crate::state;

/// Pure playback logic — no Tauri dependency, fully testable.
pub struct PlaybackService;

impl PlaybackService {
    pub fn toggle_pause(mpv: &MpvPlayer) -> Result<(), MpvError> {
        mpv.command(&["cycle", "pause"])
    }

    pub fn play(mpv: &MpvPlayer) -> Result<(), MpvError> {
        mpv.set("pause", false)
    }

    pub fn pause(mpv: &MpvPlayer) -> Result<(), MpvError> {
        mpv.set("pause", true)
    }

    pub fn stop(mpv: &MpvPlayer) -> Result<(), MpvError> {
        mpv.command(&["stop"])
    }

    pub fn seek_relative(mpv: &MpvPlayer, seconds: f64) -> Result<(), MpvError> {
        mpv.command(&["seek", &seconds.to_string(), "relative"])
    }

    pub fn seek_absolute(mpv: &MpvPlayer, seconds: f64) -> Result<(), MpvError> {
        mpv.command(&["seek", &seconds.to_string(), "absolute"])
    }

    pub fn set_volume(mpv: &MpvPlayer, volume: f64) -> Result<(), MpvError> {
        mpv.set("volume", volume)
    }

    pub fn set_speed(mpv: &MpvPlayer, speed: f64) -> Result<(), MpvError> {
        mpv.set("speed", speed)
    }

    // --- Fase 3: Playback enhancements ---

    pub fn screenshot(mpv: &MpvPlayer, path: &str) -> Result<(), MpvError> {
        mpv.command(&["screenshot-to-file", path, "subtitles"])
    }

    pub fn frame_step(mpv: &MpvPlayer) -> Result<(), MpvError> {
        mpv.command(&["frame-step"])
    }

    pub fn frame_back_step(mpv: &MpvPlayer) -> Result<(), MpvError> {
        mpv.command(&["frame-back-step"])
    }

    /// Cycle A → B → clear. Snapshots time-pos from mpv directly (no frontend
    /// latency). Loop enforcement happens in the event loop — see
    /// `mpv::events::enforce_ab_loop`.
    pub fn cycle_ab_loop(mpv: &MpvPlayer) -> Result<AbLoopState, MpvError> {
        let pos = mpv.get::<f64>("time-pos").unwrap_or(0.0);
        let (a, b) = (state::ab_loop::get_a(), state::ab_loop::get_b());

        let new_state = match (a, b) {
            (None, _) => {
                state::ab_loop::set_a(Some(pos));
                AbLoopState { a: Some(pos), b: None }
            }
            (Some(a_val), None) if pos > a_val => {
                state::ab_loop::set_b(Some(pos));
                AbLoopState { a, b: Some(pos) }
            }
            (Some(_), None) => {
                // pos <= A: replace A rather than creating an invalid B<A range.
                state::ab_loop::set_a(Some(pos));
                AbLoopState { a: Some(pos), b: None }
            }
            (Some(_), Some(_)) => {
                state::ab_loop::clear();
                AbLoopState { a: None, b: None }
            }
        };

        info!(a = ?new_state.a, b = ?new_state.b, "ab-loop cycled");
        Ok(new_state)
    }

    pub fn set_ab_loop_a(time: Option<f64>) {
        state::ab_loop::set_a(time);
    }

    pub fn set_ab_loop_b(time: Option<f64>) {
        state::ab_loop::set_b(time);
    }

    pub fn clear_ab_loop() {
        state::ab_loop::clear();
    }

    pub fn get_chapters(mpv: &MpvPlayer) -> Vec<Chapter> {
        let count: i64 = mpv
            .get_property_string("chapter-list/count")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        let current: i64 = mpv
            .get_property_string("chapter")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(-1);

        (0..count)
            .filter_map(|i| {
                let title = mpv
                    .get_property_string(&format!("chapter-list/{i}/title"))
                    .unwrap_or_else(|_| format!("Chapter {}", i + 1));
                let time: f64 = mpv
                    .get_property_string(&format!("chapter-list/{i}/time"))
                    .ok()?
                    .parse()
                    .ok()?;
                Some(Chapter {
                    index: i,
                    title,
                    time,
                    current: i == current,
                })
            })
            .collect()
    }

    pub fn seek_chapter(mpv: &MpvPlayer, index: i64) -> Result<(), MpvError> {
        mpv.set::<&str>("chapter", &index.to_string())
    }

    pub fn open_url(mpv: &MpvPlayer, url: &str) -> Result<(), MpvError> {
        mpv.command(&["loadfile", url])
    }

    pub fn get_state(mpv: &MpvPlayer) -> PlaybackState {
        PlaybackState {
            time_pos: mpv.get::<f64>("time-pos").unwrap_or(0.0),
            duration: mpv.get::<f64>("duration").unwrap_or(0.0),
            paused: mpv.get::<bool>("pause").unwrap_or(true),
            title: {
                let filename = mpv.get_property_string("filename").unwrap_or_default();
                if filename.is_empty() {
                    mpv.get_property_string("media-title").unwrap_or_default()
                } else {
                    filename
                }
            },
            volume: mpv.get::<f64>("volume").unwrap_or(100.0),
        }
    }
}

#[derive(Serialize)]
pub struct PlaybackState {
    pub time_pos: f64,
    pub duration: f64,
    pub paused: bool,
    pub title: String,
    pub volume: f64,
}

#[derive(Serialize)]
pub struct AbLoopState {
    pub a: Option<f64>,
    pub b: Option<f64>,
}

#[derive(Serialize)]
pub struct Chapter {
    pub index: i64,
    pub title: String,
    pub time: f64,
    pub current: bool,
}
