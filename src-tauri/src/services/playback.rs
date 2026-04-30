use serde::Serialize;

use crate::error::MpvError;
use crate::mpv::player::MpvPlayer;

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

    pub fn set_ab_loop(mpv: &MpvPlayer, point: &str) -> Result<AbLoopState, MpvError> {
        let a = mpv.get_property_string("ab-loop-a").unwrap_or_default();
        let b = mpv.get_property_string("ab-loop-b").unwrap_or_default();

        match point {
            "toggle" => {
                if a == "no" || a.is_empty() {
                    // Set A point
                    let pos = mpv.get::<f64>("time-pos").unwrap_or(0.0);
                    mpv.set::<&str>("ab-loop-a", &pos.to_string())?;
                    Ok(AbLoopState { a: pos, b: -1.0, active: false })
                } else if b == "no" || b.is_empty() {
                    // Set B point
                    let pos = mpv.get::<f64>("time-pos").unwrap_or(0.0);
                    mpv.set::<&str>("ab-loop-b", &pos.to_string())?;
                    let a_val: f64 = a.parse().unwrap_or(0.0);
                    Ok(AbLoopState { a: a_val, b: pos, active: true })
                } else {
                    // Clear both
                    mpv.set::<&str>("ab-loop-a", "no")?;
                    mpv.set::<&str>("ab-loop-b", "no")?;
                    Ok(AbLoopState { a: -1.0, b: -1.0, active: false })
                }
            }
            "clear" => {
                mpv.set::<&str>("ab-loop-a", "no")?;
                mpv.set::<&str>("ab-loop-b", "no")?;
                Ok(AbLoopState { a: -1.0, b: -1.0, active: false })
            }
            _ => Ok(AbLoopState { a: -1.0, b: -1.0, active: false }),
        }
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
    pub a: f64,
    pub b: f64,
    pub active: bool,
}

#[derive(Serialize)]
pub struct Chapter {
    pub index: i64,
    pub title: String,
    pub time: f64,
    pub current: bool,
}
