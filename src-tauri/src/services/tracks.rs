use serde::{Deserialize, Serialize};

use crate::error::MpvError;
use crate::mpv::player::MpvPlayer;

#[derive(Serialize)]
pub struct TrackInfo {
    pub id: i64,
    pub track_type: String,
    pub title: String,
    pub lang: String,
    pub selected: bool,
    pub external: bool,
    pub external_filename: String,
    pub codec: String,
}

pub struct TracksService;

impl TracksService {
    pub fn get_all(mpv: &MpvPlayer) -> Vec<TrackInfo> {
        let count: i64 = mpv
            .get_property_string("track-list/count")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        (0..count)
            .filter_map(|i| {
                let p = format!("track-list/{i}");
                Some(TrackInfo {
                    id: mpv.get_property_string(&format!("{p}/id")).ok()?.parse().ok()?,
                    track_type: mpv.get_property_string(&format!("{p}/type")).ok()?,
                    title: mpv.get_property_string(&format!("{p}/title")).unwrap_or_default(),
                    lang: mpv.get_property_string(&format!("{p}/lang")).unwrap_or_default(),
                    selected: mpv.get_property_string(&format!("{p}/selected")).ok().as_deref() == Some("yes"),
                    external: mpv.get_property_string(&format!("{p}/external")).ok().as_deref() == Some("yes"),
                    external_filename: mpv.get_property_string(&format!("{p}/external-filename")).unwrap_or_default(),
                    codec: mpv.get_property_string(&format!("{p}/codec")).unwrap_or_default(),
                })
            })
            .collect()
    }

    pub fn select_subtitle(mpv: &MpvPlayer, id: i64) -> Result<(), MpvError> {
        if id < 0 {
            mpv.set::<&str>("sid", "no")
        } else {
            mpv.set::<&str>("sid", &id.to_string())
        }
    }

    pub fn select_audio(mpv: &MpvPlayer, id: i64) -> Result<(), MpvError> {
        mpv.set::<&str>("aid", &id.to_string())
    }

    pub fn load_subtitle(mpv: &MpvPlayer, path: &str) -> Result<(), MpvError> {
        mpv.command(&["sub-add", path])
    }

    pub fn toggle_subtitles(mpv: &MpvPlayer) -> Result<(), MpvError> {
        mpv.command(&["cycle", "sub-visibility"])
    }

    pub fn set_subtitle_delay(mpv: &MpvPlayer, seconds: f64) -> Result<(), MpvError> {
        mpv.set("sub-delay", seconds)
    }

    pub fn set_audio_delay(mpv: &MpvPlayer, seconds: f64) -> Result<(), MpvError> {
        mpv.set("audio-delay", seconds)
    }

    pub fn set_sub_style(mpv: &MpvPlayer, style: &SubStyle) -> Result<(), MpvError> {
        mpv.set::<&str>("sub-font", &style.font)?;
        mpv.set("sub-font-size", style.size as f64)?;
        mpv.set::<&str>("sub-color", &style.color)?;
        mpv.set::<&str>("sub-border-color", &style.border_color)?;
        mpv.set("sub-border-size", style.border_size as f64)?;
        mpv.set("sub-pos", style.position as f64)?;
        mpv.set::<&str>("sub-bold", if style.bold { "yes" } else { "no" })?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SubStyle {
    pub font: String,
    pub size: u32,
    pub color: String,
    pub border_color: String,
    pub border_size: u32,
    pub position: u32,
    #[serde(default)]
    pub bold: bool,
}
