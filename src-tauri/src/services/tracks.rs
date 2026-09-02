use serde::{Deserialize, Serialize};

use crate::error::MpvError;
use crate::mpv::player::MpvPlayer;

/// One entry of mpv's `track-list`.
///
/// Deserialised straight from mpv's JSON, so the renames below are
/// deserialise-only — the names the frontend receives stay as they were.
/// `title`, `lang` and `external-filename` are absent rather than empty when
/// a track does not carry them, hence `default` on each; `selected` and
/// `external` arrive as real JSON booleans. Every other key mpv emits
/// (`src-id`, `demux-*`, `metadata`, …) is ignored.
#[derive(Serialize, Deserialize)]
pub struct TrackInfo {
    pub id: i64,
    #[serde(rename(deserialize = "type"))]
    pub track_type: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub lang: String,
    #[serde(default)]
    pub selected: bool,
    #[serde(default)]
    pub external: bool,
    #[serde(default, rename(deserialize = "external-filename"))]
    pub external_filename: String,
    #[serde(default)]
    pub codec: String,
}

/// Parse mpv's `track-list` JSON. Split out of `get_all` so the shape can be
/// tested against real mpv output without an mpv instance.
fn parse_track_list(json: &str) -> Vec<TrackInfo> {
    match serde_json::from_str(json) {
        Ok(tracks) => tracks,
        Err(e) => {
            // An empty panel with nothing in the log is the worst outcome: the
            // shape mpv sends is what this parser is pinned to, so if it ever
            // changes the reason has to be visible.
            tracing::warn!(error = %e, "track-list: could not be parsed");
            Vec::new()
        }
    }
}

pub struct TracksService;

impl TracksService {
    /// mpv serialises node-typed properties as JSON when they are read as a
    /// string, so the whole list arrives in a single FFI round-trip. Reading it
    /// field by field cost 8 calls per track — 161 for a 20-track file — each
    /// allocating a `CString` going in, a `strdup` inside mpv and a `String`
    /// coming back out, and `get_all` runs on every panel open and every track
    /// change.
    pub fn get_all(mpv: &MpvPlayer) -> Vec<TrackInfo> {
        mpv.get_property_string("track-list")
            .map(|json| parse_track_list(&json))
            .unwrap_or_default()
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
        mpv.set("sub-font-size", f64::from(style.size))?;
        mpv.set::<&str>("sub-color", &style.color)?;
        mpv.set::<&str>("sub-border-color", &style.border_color)?;
        mpv.set("sub-border-size", f64::from(style.border_size))?;
        mpv.set("sub-pos", f64::from(style.position))?;
        mpv.set::<&str>("sub-bold", if style.bold { "yes" } else { "no" })?;
        // A plain black drop shadow (offset in pixels; 0 = off). Cheap — mpv
        // rasterizes it only when the subtitle text changes, not per frame.
        mpv.set("sub-shadow-offset", f64::from(style.shadow))?;
        mpv.set::<&str>("sub-shadow-color", "#000000")?;
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
    #[serde(default)]
    pub shadow: u32,
}


#[cfg(test)]
mod tests {
    use super::parse_track_list;

    /// Real `track-list` output from libmpv, captured from a Matroska file
    /// muxed with one video track, two audio tracks (one titled, one not) and
    /// two subtitle tracks. Trimmed only of the `metadata` and `demux-*` keys
    /// we ignore; every key the parser reads is verbatim.
    const REAL: &str = r#"[
      {"id":1,"type":"video","src-id":1,"image":false,"default":false,"external":false,
       "selected":true,"ff-index":0,"decoder":"h264","codec":"h264"},
      {"id":1,"type":"audio","src-id":2,"title":"Japanese 5.1","lang":"jpn","default":true,
       "external":false,"selected":true,"ff-index":1,"codec":"aac"},
      {"id":2,"type":"audio","src-id":3,"lang":"eng","default":false,"external":false,
       "selected":false,"ff-index":2,"codec":"aac"},
      {"id":1,"type":"sub","src-id":4,"title":"Forced","lang":"por","default":true,
       "external":false,"selected":true,"ff-index":3,"codec":"subrip"},
      {"id":2,"type":"sub","src-id":5,"lang":"eng","default":false,"external":false,
       "selected":false,"ff-index":4,"codec":"subrip"}
    ]"#;

    #[test]
    fn reads_every_track_from_real_mpv_output() {
        let tracks = parse_track_list(REAL);
        assert_eq!(tracks.len(), 5);
        let types: Vec<&str> = tracks.iter().map(|t| t.track_type.as_str()).collect();
        assert_eq!(types, ["video", "audio", "audio", "sub", "sub"]);
        // mpv numbers ids per type, which is what `aid`/`sid` expect.
        assert_eq!(tracks[1].id, 1);
        assert_eq!(tracks[2].id, 2);
    }

    #[test]
    fn absent_title_and_lang_read_as_empty_rather_than_failing() {
        // mpv omits these keys entirely instead of sending empty strings.
        let tracks = parse_track_list(REAL);
        assert_eq!(tracks[0].title, "");
        assert_eq!(tracks[0].lang, "");
        assert_eq!(tracks[1].title, "Japanese 5.1");
        assert_eq!(tracks[2].title, "");
        assert_eq!(tracks[2].lang, "eng");
    }

    #[test]
    fn selection_and_external_are_json_booleans() {
        // Read field by field these came back as the string "yes"; in the JSON
        // they are real booleans, and reading them as strings would be wrong.
        let tracks = parse_track_list(REAL);
        assert!(tracks[1].selected);
        assert!(!tracks[2].selected);
        assert!(tracks.iter().all(|t| !t.external));
        assert!(tracks.iter().all(|t| t.external_filename.is_empty()));
    }

    #[test]
    fn an_added_external_subtitle_carries_its_filename() {
        let tracks = parse_track_list(
            r#"[{"id":3,"type":"sub","external":true,"selected":true,
                 "external-filename":"C:\\videos\\movie.pt.srt","codec":"subrip"}]"#,
        );
        assert_eq!(tracks.len(), 1);
        assert!(tracks[0].external);
        assert_eq!(tracks[0].external_filename, r"C:\videos\movie.pt.srt");
    }

    #[test]
    fn serialises_with_the_names_the_frontend_expects() {
        // The deserialise-only renames must not leak into what the frontend
        // receives: `src/lib/bindings/tracks.ts` reads `track_type` and
        // `external_filename`, not mpv's `type` and `external-filename`.
        let tracks = parse_track_list(REAL);
        let json = serde_json::to_string(&tracks[1]).unwrap();
        assert!(json.contains(r#""track_type":"audio""#), "{json}");
        assert!(json.contains(r#""external_filename":"""#), "{json}");
        assert!(!json.contains(r#""type":"#), "mpv's key must not reach the UI");
    }

    #[test]
    fn nothing_playing_yields_no_tracks() {
        assert!(parse_track_list("[]").is_empty());
    }

    #[test]
    fn malformed_output_yields_no_tracks_instead_of_panicking() {
        // A property read can fail mid-shutdown; the panel showing an empty
        // list beats the event loop unwinding.
        assert!(parse_track_list("not json").is_empty());
        assert!(parse_track_list(r#"[{"type":"sub"}]"#).is_empty(), "id is required");
    }
}
