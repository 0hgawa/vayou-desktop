use std::path::Path;

use serde::Serialize;

use crate::error::MpvError;
use crate::mpv::player::MpvPlayer;

pub const MEDIA_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "mpg", "mpeg", "m4v", "3gp", "ts", "vob",
    "mp3", "flac", "wav", "ogg", "m4a", "aac", "opus", "wma",
];

#[derive(Serialize)]
pub struct PlaylistItem {
    pub index: i64,
    pub filename: String,
    pub current: bool,
    pub title: String,
}

pub struct PlaylistService;

impl PlaylistService {
    /// Add a file or folder to the playlist.
    pub fn add(mpv: &MpvPlayer, path: &str) -> Result<(), MpvError> {
        if Path::new(path).is_dir() {
            let files = scan_media_folder(path);
            for file in &files {
                mpv.command(&["loadfile", file, "append-play"])?;
            }
            return Ok(());
        }
        mpv.command(&["loadfile", path, "append-play"])
    }

    pub fn remove(mpv: &MpvPlayer, index: i64) -> Result<(), MpvError> {
        mpv.command(&["playlist-remove", &index.to_string()])
    }

    pub fn next(mpv: &MpvPlayer) -> Result<(), MpvError> {
        mpv.command(&["playlist-next"])
    }

    pub fn prev(mpv: &MpvPlayer) -> Result<(), MpvError> {
        mpv.command(&["playlist-prev"])
    }

    pub fn play_index(mpv: &MpvPlayer, index: i64) -> Result<(), MpvError> {
        mpv.set::<&str>("playlist-pos", &index.to_string())
    }

    pub fn clear(mpv: &MpvPlayer) -> Result<(), MpvError> {
        mpv.command(&["playlist-clear"])
    }

    /// Open a file and populate playlist with all sibling media files.
    /// Optionally resumes from a saved position.
    pub fn open_with_siblings(
        mpv: &MpvPlayer,
        path: &str,
    ) -> Result<(), MpvError> {
        let target = Path::new(path);
        let target_name = target.file_name();

        // Scan and sort siblings
        let siblings = match target.parent() {
            Some(parent) => scan_media_folder(&parent.to_string_lossy()),
            None => vec![],
        };

        if siblings.is_empty() {
            // No siblings found — just load the file directly
            return mpv.command(&["loadfile", path, "replace"]);
        }

        // Find target index in sorted list
        let target_idx = siblings
            .iter()
            .position(|s| Path::new(s).file_name() == target_name)
            .unwrap_or(0);

        // Load all in sorted order, then jump to target
        mpv.command(&["loadfile", &siblings[0], "replace"])?;
        for file in &siblings[1..] {
            mpv.command(&["loadfile", file, "append"])?;
        }
        if target_idx > 0 {
            mpv.set::<&str>("playlist-pos", &target_idx.to_string())?;
        }

        Ok(())
    }

    pub fn get_all(mpv: &MpvPlayer) -> Vec<PlaylistItem> {
        let count: i64 = mpv
            .get_property_string("playlist/count")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        let current: i64 = mpv
            .get_property_string("playlist-pos")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(-1);

        (0..count)
            .filter_map(|i| {
                let filename = mpv
                    .get_property_string(&format!("playlist/{i}/filename"))
                    .ok()?;
                let title = Path::new(&filename)
                    .file_name().map_or_else(|| filename.clone(), |s| s.to_string_lossy().to_string());
                Some(PlaylistItem {
                    index: i,
                    filename,
                    current: i == current,
                    title,
                })
            })
            .collect()
    }
}

/// Scan a directory for media files, sorted with natural ordering.
/// Single source of truth — used by both `add` and `open_with_siblings`.
fn scan_media_folder(dir: &str) -> Vec<String> {
    let mut files: Vec<String> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let p = entry.ok()?.path();
            if !p.is_file() {
                return None;
            }
            let ext = p.extension()?.to_str()?.to_lowercase();
            if MEDIA_EXTENSIONS.contains(&ext.as_str()) {
                Some(p.to_string_lossy().to_string())
            } else {
                None
            }
        })
        .collect();
    files.sort_by_key(|f| natural_sort_key(f));
    files
}

/// Natural sort key: "Episode 2" < "Episode 10".
///
/// Digit runs are zero-padded to a fixed width so they compare numerically.
/// The padding has to name `0` as an explicit fill character (`:0>20`), not
/// use the `0` flag (`:>020`): that flag is numeric-only and is silently
/// ignored for strings, so the runs padded with spaces instead — and since a
/// space sorts below every digit, "ep8" came out before "ep007".
fn natural_sort_key(s: &str) -> Vec<(bool, String)> {
    let mut result = Vec::new();
    let mut chunk = String::new();
    let mut is_digit = false;

    for c in s.chars() {
        let d = c.is_ascii_digit();
        if d != is_digit && !chunk.is_empty() {
            if is_digit {
                result.push((true, format!("{chunk:0>20}")));
            } else {
                result.push((false, chunk.to_lowercase()));
            }
            chunk.clear();
        }
        is_digit = d;
        chunk.push(c);
    }
    if !chunk.is_empty() {
        if is_digit {
            result.push((true, format!("{chunk:0>20}")));
        } else {
            result.push((false, chunk.to_lowercase()));
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::natural_sort_key;

    fn sorted(mut v: Vec<&str>) -> Vec<&str> {
        v.sort_by_key(|s| natural_sort_key(s));
        v
    }

    #[test]
    fn episode_numbers_sort_numerically_not_lexically() {
        // The whole point: plain string ordering puts "10" before "2".
        assert_eq!(
            sorted(vec!["Ep 10.mkv", "Ep 2.mkv", "Ep 1.mkv"]),
            ["Ep 1.mkv", "Ep 2.mkv", "Ep 10.mkv"]
        );
        assert_eq!(
            sorted(vec!["S01E100.mkv", "S01E9.mkv", "S01E10.mkv"]),
            ["S01E9.mkv", "S01E10.mkv", "S01E100.mkv"]
        );
    }

    #[test]
    fn leading_zeros_do_not_change_the_order() {
        assert_eq!(
            sorted(vec!["ep007.mkv", "ep8.mkv", "ep06.mkv"]),
            ["ep06.mkv", "ep007.mkv", "ep8.mkv"]
        );
    }

    #[test]
    fn text_comparison_is_case_insensitive() {
        assert_eq!(natural_sort_key("ABC"), natural_sort_key("abc"));
        assert_eq!(
            sorted(vec!["beta.mkv", "Alpha.mkv"]),
            ["Alpha.mkv", "beta.mkv"]
        );
    }

    #[test]
    fn digit_and_text_runs_alternate_in_order() {
        // "a" / "1" / "b" / "22" — four runs. Digit runs are zero-padded so
        // they compare numerically while text runs stay lexical.
        let key = natural_sort_key("a1b22");
        assert_eq!(key.len(), 4);
        assert_eq!(key[0], (false, "a".to_string()));
        assert_eq!(key[2], (false, "b".to_string()));
        assert!(key[1].0 && key[3].0, "digit runs are flagged");
        assert!(key[1].1 < key[3].1, "1 sorts before 22");
    }
}
