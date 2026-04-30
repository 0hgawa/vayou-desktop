use std::path::Path;
use std::sync::Arc;
use tauri::{Emitter, State};
use crate::error::AppError;
use crate::services::{subtitle_extract, tracks::TracksService, translate};
use crate::state::{AppState, MpvState};

#[derive(Clone, serde::Serialize)]
struct TranslateProgress { current: usize, total: usize, done: bool }

#[tauri::command]
pub async fn translate_subtitles(
    target_lang: String, app: tauri::AppHandle,
    mpv_state: State<'_, MpvState>, app_state: State<'_, AppState>,
) -> Result<String, AppError> {
    let mpv = mpv_state.get()?;
    let video_path = app_state.with(|_, f| f.clone())?
        .ok_or_else(|| AppError::Config("No file playing".into()))?;

    let tracks = TracksService::get_all(mpv);
    let sub_track = tracks.iter().find(|t| t.track_type == "sub" && t.selected)
        .ok_or_else(|| AppError::Config("No subtitle track selected".into()))?;

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
            subtitle_extract::extract_ass_header_from_video(&video_path, sub_track.id).ok()
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
        subtitle_extract::extract_from_video(&video_path, Some(sub_track.id), is_ass)
    }.map_err(AppError::Config)?;

    if entries.is_empty() { return Err(AppError::Config("No subtitle entries found".into())); }

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

    let mut handles = Vec::with_capacity(total);
    for (idx, indices) in chunks.into_iter().enumerate() {
        let (lang, entries_ref, app_c) = (lang.clone(), entries_arc.clone(), app.clone());
        handles.push(tokio::spawn(async move {
            let combined: String = indices.iter()
                .map(|&i| entries_ref[i].text.as_str())
                .collect::<Vec<_>>().join("\n\n");
            let result = translate::translate(&combined, &lang).await;
            let _ = app_c.emit("translate:progress", TranslateProgress { current: idx + 1, total, done: false });
            (indices, result)
        }));
    }

    let mut translated = entries;
    for h in handles {
        let (indices, result) = h.await.map_err(|e| AppError::Config(e.to_string()))?;
        if let Ok(t) = result {
            let parts: Vec<&str> = t.split("\n\n").collect();
            for (j, &idx) in indices.iter().enumerate() {
                if j < parts.len() && idx < translated.len() {
                    translated[idx].text = parts[j].trim().to_string();
                }
            }
        }
    }

    // Write in original format
    if let Some(header) = ass_header {
        subtitle_extract::write_ass(&translated, &header, &out_path).map_err(AppError::Config)?;
    } else {
        subtitle_extract::write_srt(&translated, &out_path).map_err(AppError::Config)?;
    }

    mpv.command(&["sub-add", &out_path, "auto"])?;
    let _ = app.emit("translate:progress", TranslateProgress { current: total, total, done: true });
    Ok(out_path)
}

fn build_sub_path(video_path: &str, lang: &str, ext: &str) -> String {
    let p = Path::new(video_path);
    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("sub");
    let parent = p.parent().and_then(|p| p.to_str()).unwrap_or(".");
    format!("{parent}/{stem}.{lang}.{ext}")
}
