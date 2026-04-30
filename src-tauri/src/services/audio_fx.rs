use crate::error::MpvError;
use crate::mpv::player::MpvPlayer;

pub struct AudioFxService;

impl AudioFxService {
    pub fn set_normalization(mpv: &MpvPlayer, enabled: bool) -> Result<(), MpvError> {
        if enabled {
            mpv.command(&["af", "add", "@norm:dynaudnorm"])
        } else {
            mpv.command(&["af", "remove", "@norm"])
        }
    }

    /// Set 5-band equalizer. Values are in dB (-12 to +12).
    /// Bands: [60Hz, 230Hz, 910Hz, 3.6kHz, 14kHz]
    pub fn set_equalizer(mpv: &MpvPlayer, bands: &[f64; 5]) -> Result<(), MpvError> {
        // Map 5 bands to superequalizer's 10 bands (duplicating neighbors)
        let eq = format!(
            "@eq:superequalizer={b0}:{b0}:{b1}:{b1}:{b2}:{b2}:{b3}:{b3}:{b4}:{b4}",
            b0 = bands[0],
            b1 = bands[1],
            b2 = bands[2],
            b3 = bands[3],
            b4 = bands[4],
        );
        // Remove old EQ first, then add new
        let _ = mpv.command(&["af", "remove", "@eq"]);
        if bands.iter().any(|&b| b.abs() > 0.1) {
            mpv.command(&["af", "add", &eq])
        } else {
            Ok(()) // All flat — just remove
        }
    }

    pub fn reset_equalizer(mpv: &MpvPlayer) -> Result<(), MpvError> {
        let _ = mpv.command(&["af", "remove", "@eq"]);
        Ok(())
    }
}
