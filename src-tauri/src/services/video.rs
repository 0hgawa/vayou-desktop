use serde::Serialize;

use crate::error::MpvError;
use crate::mpv::player::MpvPlayer;

pub struct VideoService;

impl VideoService {
    pub fn set_brightness(mpv: &MpvPlayer, value: i64) -> Result<(), MpvError> {
        mpv.set::<&str>("brightness", &value.to_string())
    }

    pub fn set_contrast(mpv: &MpvPlayer, value: i64) -> Result<(), MpvError> {
        mpv.set::<&str>("contrast", &value.to_string())
    }

    pub fn set_saturation(mpv: &MpvPlayer, value: i64) -> Result<(), MpvError> {
        mpv.set::<&str>("saturation", &value.to_string())
    }

    pub fn toggle_deinterlace(mpv: &MpvPlayer) -> Result<(), MpvError> {
        mpv.command(&["cycle", "deinterlace"])
    }

    pub fn set_zoom(mpv: &MpvPlayer, value: f64) -> Result<(), MpvError> {
        mpv.set("video-zoom", value)
    }

    pub fn set_pan(mpv: &MpvPlayer, x: f64, y: f64) -> Result<(), MpvError> {
        mpv.set("video-pan-x", x)?;
        mpv.set("video-pan-y", y)
    }

    pub fn get_zoom_pan(mpv: &MpvPlayer) -> ZoomPanState {
        ZoomPanState {
            zoom: mpv.get::<f64>("video-zoom").unwrap_or(0.0),
            pan_x: mpv.get::<f64>("video-pan-x").unwrap_or(0.0),
            pan_y: mpv.get::<f64>("video-pan-y").unwrap_or(0.0),
        }
    }

    pub fn reset_zoom_pan(mpv: &MpvPlayer) -> Result<(), MpvError> {
        mpv.set("video-zoom", 0.0_f64)?;
        mpv.set("video-pan-x", 0.0_f64)?;
        mpv.set("video-pan-y", 0.0_f64)
    }

    pub fn set_aspect_ratio(mpv: &MpvPlayer, ratio: &str) -> Result<(), MpvError> {
        mpv.set::<&str>("video-aspect-override", ratio)
    }

    pub fn get_aspect_ratio(mpv: &MpvPlayer) -> String {
        mpv.get_property_string("video-aspect-override").unwrap_or_else(|_| "-1".into())
    }
}

#[derive(Serialize)]
pub struct ZoomPanState {
    pub zoom: f64,
    pub pan_x: f64,
    pub pan_y: f64,
}
