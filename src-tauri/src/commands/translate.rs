use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use tauri::{Emitter, State};
use tokio::sync::Semaphore;
use tracing::{info, warn};
use crate::error::AppError;
use crate::mpv::player::MpvPlayer;
use crate::services::{subtitle_extract, tracks::TracksService, translate};
use crate::state::MpvState;

/// Caps the number of concurrent HTTP requests to Google Translate. Higher
/// numbers trigger 429 rate-limits that look like the whole pipeline froze.
const MAX_CONCURRENT_CHUNKS: usize = 8;

/// Image-based subtitle codecs we cannot extract text from. Refusing them
/// up front gives a clear error instead of letting ffmpeg hang.
const UNSUPPORTED_CODECS: &[&str] = &["hdmv_pgs_subtitle", "dvd_subtitle", "dvb_subtitle", "pgs"];

/// Monotonic id of the most recently started translation. Older runs check
/// this before mutating mpv and bail when superseded — fixes the case
/// where switching language quickly used to leave several requests in
/// flight competing for the same sub-add slot.
fn current_run_id() -> &'static AtomicU64 {
    static R: OnceLock<AtomicU64> = OnceLock::new();
    R.get_or_init(|| AtomicU64::new(0))
}

#[derive(Clone, serde::Serialize)]
struct TranslateProgress { current: usize, total: usize, done: bool }

/// State of the last translated subtitle: file path we added to mpv plus
/// the source `sid` it was derived from. Callers use the `source_sid` to
/// put the user back on a real sub track after the translation is removed.
struct LastTranslation {
    path: String,
    source_sid: i64,
}

fn last_translation() -> &'static Mutex<Option<LastTranslation>> {
    static S: OnceLock<Mutex<Option<LastTranslation>>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(None))
}

/// Removes the previous translation track and returns the source `sid` it
/// was derived from. Callers decide what to do with that sid:
/// `translate_subtitles` only restores it as a fallback when no other sub
/// is currently selected; `clear_translation` always restores it so the
/// user keeps a real sub on screen after disabling translation.
fn remove_previous_translation(mpv: &MpvPlayer) -> Option<i64> {
    let prev = match last_translation().lock() { Ok(mut g) => g.take(), Err(_) => None }?;
    info!(path = %prev.path, "translate: removing previous translation");
    let tracks = TracksService::get_all(mpv);
    if let Some(t) = tracks.iter().find(|t| t.external && t.external_filename == prev.path) {
        match mpv.command(&["sub-remove", &t.id.to_string()]) {
            Ok(_) => info!(track_id = t.id, "translate: sub-remove ok"),
            Err(e) => warn!(track_id = t.id, error = %e, "translate: sub-remove failed"),
        }
    } else {
        info!("translate: previous track not in mpv list (already gone)");
    }
    if let Err(e) = std::fs::remove_file(&prev.path) {
        warn!(error = %e, "translate: temp file remove failed");
    }
    Some(prev.source_sid)
}

#[tauri::command]
pub async fn translate_subtitles(
    target_lang: String, app: tauri::AppHandle,
    mpv_state: State<'_, MpvState>,
) -> Result<String, AppError> {
    let my_run = current_run_id().fetch_add(1, Ordering::SeqCst) + 1;
    info!(target_lang = %target_lang, run = my_run, "translate_subtitles: START");
    let mpv = mpv_state.get()?;

    // Remove the previous translation track (if any) and remember the source
    // it was derived from. We only restore that source as a fallback below —
    // not unconditionally — so the user can switch to a different sub track
    // mid-flight and re-translate without first having to disable the
    // translation manually.
    let prev_translation_source = remove_previous_translation(mpv);

    // Resolve the video from what mpv is ACTUALLY playing — not app_state's
    // current_file, which only tracks explicit opens and goes stale on
    // playlist navigation (we'd translate the previous episode's subs).
    let video_path = mpv.get_property_string("path").ok()
        .filter(|p| !p.is_empty())
        .ok_or_else(|| AppError::Config("No file playing".into()))?;
    info!(video = %video_path, "translate_subtitles: video resolved");

    let tracks = TracksService::get_all(mpv);
    let sub_track = tracks.iter()
        .find(|t| t.track_type == "sub" && t.selected)
        .or_else(|| {
            // Nothing is selected after the remove — that only happens when
            // the translation was the only `selected=true` sub. Restore the
            // source it came from so the user keeps the same track they had.
            let sid = prev_translation_source?;
            mpv.set::<&str>("sid", &sid.to_string()).ok();
            tracks.iter().find(|t| t.track_type == "sub" && t.id == sid)
        })
        .ok_or_else(|| {
            warn!(track_count = tracks.len(), sub_count = tracks.iter().filter(|t| t.track_type == "sub").count(), "translate_subtitles: no selected sub track");
            AppError::Config("No subtitle track selected".into())
        })?;
    info!(sub_id = sub_track.id, codec = %sub_track.codec, external = sub_track.external, "translate_subtitles: source track resolved");

    if UNSUPPORTED_CODECS.contains(&sub_track.codec.as_str()) {
        warn!(codec = %sub_track.codec, "translate_subtitles: unsupported image-based codec");
        return Err(AppError::Config(format!(
            "'{}' subtitles are image-based and cannot be translated",
            sub_track.codec
        )));
    }

    let source_sid = sub_track.id;

    // Detect if source is ASS
    let is_ass = sub_track.codec == "ass" || sub_track.codec == "ssa"
        || (sub_track.external && matches!(
            Path::new(&sub_track.external_filename).extension().and_then(|e| e.to_str()),
            Some("ass" | "ssa")
        ));

    let out_ext = if is_ass { "ass" } else { "srt" };
    let out_path = build_sub_path(&video_path, &target_lang, out_ext);

    let _ = app.emit("translate:progress", TranslateProgress { current: 0, total: 0, done: false });

    // Extract ASS header if needed
    let ass_header = if is_ass {
        if sub_track.external && !sub_track.external_filename.is_empty() {
            subtitle_extract::extract_ass_header_from_file(&sub_track.external_filename).ok()
        } else {
            subtitle_extract::extract_ass_header_from_video(&video_path, sub_track.id).await.ok()
        }
    } else { None };

    // Extract subtitle entries
    let entries = if sub_track.external && !sub_track.external_filename.is_empty() {
        let ext = Path::new(&sub_track.external_filename).extension()
            .and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        match ext.as_str() {
            "ass" | "ssa" => subtitle_extract::extract_from_ass(&sub_track.external_filename),
            _ => subtitle_extract::extract_from_srt(&sub_track.external_filename),
        }
    } else {
        subtitle_extract::extract_from_video(&video_path, Some(sub_track.id), is_ass).await
    }.map_err(AppError::Config)?;

    if entries.is_empty() {
        warn!("translate_subtitles: extraction returned 0 entries");
        return Err(AppError::Config("No subtitle entries found".into()));
    }
    info!(entry_count = entries.len(), "translate_subtitles: entries extracted");

    // Chunk by ~4500 chars, translate in parallel
    let mut chunks: Vec<Vec<usize>> = Vec::new();
    let (mut cur, mut len) = (Vec::new(), 0usize);
    for (i, e) in entries.iter().enumerate() {
        let l = e.text.len() + 2;
        if len + l > 4500 && !cur.is_empty() { chunks.push(cur); cur = Vec::new(); len = 0; }
        cur.push(i); len += l;
    }
    if !cur.is_empty() { chunks.push(cur); }

    let total = chunks.len();
    let lang = Arc::new(target_lang);
    let entries_arc = Arc::new(entries.clone());
    let sem = Arc::new(Semaphore::new(MAX_CONCURRENT_CHUNKS));

    let mut handles = Vec::with_capacity(total);
    for (idx, indices) in chunks.into_iter().enumerate() {
        let (lang, entries_ref, app_c, sem_c) = (lang.clone(), entries_arc.clone(), app.clone(), sem.clone());
        handles.push(tokio::spawn(async move {
            let _permit = sem_c.acquire_owned().await.ok();
            let combined: String = indices.iter()
                .map(|&i| entries_ref[i].text.as_str())
                .collect::<Vec<_>>().join("\n\n");
            let result = translate::translate(&combined, &lang).await;
            let _ = app_c.emit("translate:progress", TranslateProgress { current: idx + 1, total, done: false });
            (indices, result)
        }));
    }

    let mut translated = entries;
    let mut failed_chunks = 0usize;
    for h in handles {
        let (indices, result) = h.await.map_err(|e| AppError::Config(e.to_string()))?;
        match result {
            Ok(t) => {
                let parts: Vec<&str> = t.split("\n\n").collect();
                for (j, &idx) in indices.iter().enumerate() {
                    if j < parts.len() && idx < translated.len() {
                        translated[idx].text = parts[j].trim().to_string();
                    }
                }
            }
            Err(e) => {
                failed_chunks += 1;
                warn!(chunk_size = indices.len(), error = %e, "translate_subtitles: chunk failed");
            }
        }
    }
    // If every single chunk failed, the output file would just be the
    // original subtitle reformatted — surfacing that as a real error so
    // the UI can show "translation failed" instead of silently switching
    // to an unchanged track.
    if failed_chunks == total {
        return Err(AppError::Config(
            "Translation failed: all chunks were rate-limited or rejected by the upstream service".into()
        ));
    }
    if failed_chunks > 0 {
        warn!(failed = failed_chunks, total, "translate_subtitles: some chunks failed; output is partially translated");
    }

    // Bail if a newer translation request came in while we were translating
    // — otherwise we'd add a stale track on top of theirs.
    if current_run_id().load(Ordering::SeqCst) != my_run {
        info!(run = my_run, "translate_subtitles: superseded — skipping sub-add");
        return Err(AppError::Config("Superseded by a newer translation".into()));
    }

    // Write in original format
    if let Some(header) = ass_header {
        subtitle_extract::write_ass(&translated, &header, &out_path).map_err(AppError::Config)?;
    } else {
        subtitle_extract::write_srt(&translated, &out_path).map_err(AppError::Config)?;
    }

    // `select` makes mpv switch to the new track immediately; passing an
    // explicit title + lang keeps the dropdown name clean (otherwise mpv
    // falls back to the temp filename like "pt.ass").
    let lang_str = lang.as_str();
    info!(path = %out_path, lang = lang_str, "translate_subtitles: sub-add");
    mpv.command(&["sub-add", &out_path, "select", lang_to_name(lang_str), lang_str])?;
    if let Ok(mut g) = last_translation().lock() {
        *g = Some(LastTranslation { path: out_path.clone(), source_sid });
    }
    let _ = app.emit("translate:progress", TranslateProgress { current: total, total, done: true });
    info!("translate_subtitles: DONE");
    Ok(out_path)
}

/// Remove the loaded translation track. Called when the user picks "Off".
/// (When mpv loads a new file it already drops external tracks, so the cached
/// path is reset by the next translate call.)
#[tauri::command]
pub async fn clear_translation(mpv_state: State<'_, MpvState>) -> Result<(), AppError> {
    info!("clear_translation");
    if let Ok(mpv) = mpv_state.get() {
        if let Some(sid) = remove_previous_translation(mpv) {
            let _ = mpv.set::<&str>("sid", &sid.to_string());
        }
    }
    Ok(())
}

fn lang_to_name(code: &str) -> &'static str {
    match code {
        "pt" => "Português",
        "en" => "English",
        "es" => "Español",
        "fr" => "Français",
        "de" => "Deutsch",
        "it" => "Italiano",
        "ja" => "日本語",
        "ko" => "한국어",
        "zh" => "中文",
        "ru" => "Русский",
        "ar" => "العربية",
        "hi" => "हिन्दी",
        _ => "Translated",
    }
}

/// Build a path for the translated subtitle inside the OS temp dir, so we
/// don't pollute the user's video folder. The OS clears its temp dir
/// periodically, which is the behavior the user wants ("apenas no cache").
fn build_sub_path(video_path: &str, lang: &str, ext: &str) -> String {
    let stem = Path::new(video_path).file_stem().and_then(|s| s.to_str()).unwrap_or("sub");
    let dir = std::env::temp_dir().join("vayou-translate");
    let _ = std::fs::create_dir_all(&dir);
    dir.join(format!("{stem}.{lang}.{ext}")).to_string_lossy().into_owned()
}
