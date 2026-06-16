mod commands;
mod error;
mod mpv;
mod services;
mod state;

use tauri::Emitter;
use tracing_subscriber::EnvFilter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Force GTK/Tauri to use the X11 backend even on Wayland sessions, so the
    // X11 Window ID we hand to mpv exists. Native Wayland would need
    // mpv_render_context, which is out of scope for v0.1.x.
    #[cfg(target_os = "linux")]
    if std::env::var_os("GDK_BACKEND").is_none() {
        std::env::set_var("GDK_BACKEND", "x11");
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("vayou=info")),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(state::MpvState::new())
        .manage(state::AppState::new())
        .setup(|app| {
            let args: Vec<String> = std::env::args().skip(1)
                .filter(|a| !a.starts_with('-') && std::path::Path::new(a).exists())
                .collect();
            if !args.is_empty() {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    let _ = handle.emit("open-files", args);
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Playback
            commands::playback::init_player,
            commands::playback::open_file,
            commands::playback::toggle_pause,
            commands::playback::play,
            commands::playback::pause,
            commands::playback::stop,
            commands::playback::seek_relative,
            commands::playback::seek_absolute,
            commands::playback::set_volume,
            commands::playback::set_speed,
            commands::playback::get_playback_state,
            commands::playback::screenshot,
            commands::playback::frame_step,
            commands::playback::frame_back_step,
            commands::playback::cycle_ab_loop,
            commands::playback::set_ab_loop_a,
            commands::playback::set_ab_loop_b,
            commands::playback::clear_ab_loop,
            commands::playback::get_chapters,
            commands::playback::seek_chapter,
            commands::playback::open_url,
            commands::playback::set_mpv_property,
            commands::playback::mpv_command,
            // Tracks
            commands::tracks::get_tracks,
            commands::tracks::select_subtitle,
            commands::tracks::select_audio_track,
            commands::tracks::load_subtitle,
            commands::tracks::toggle_subtitles,
            commands::tracks::set_subtitle_delay,
            commands::tracks::set_audio_delay,
            commands::tracks::set_sub_style,
            // Playlist
            commands::playlist::playlist_add,
            commands::playlist::playlist_remove,
            commands::playlist::playlist_next,
            commands::playlist::playlist_prev,
            commands::playlist::playlist_play_index,
            commands::playlist::playlist_clear,
            commands::playlist::get_playlist,
            // Settings
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::settings::get_recent_files,
            commands::settings::clear_recent_files,
            // Video
            commands::video::set_brightness,
            commands::video::set_contrast,
            commands::video::set_saturation,
            commands::video::toggle_deinterlace,
            commands::video::set_video_zoom,
            commands::video::set_video_pan,
            commands::video::get_video_zoom_pan,
            commands::video::reset_video_zoom_pan,
            commands::video::set_aspect_ratio,
            commands::video::get_aspect_ratio,
            // Audio FX
            commands::audio_fx::set_audio_normalization,
            commands::audio_fx::set_audio_equalizer,
            commands::audio_fx::reset_audio_equalizer,
            // Media Info
            commands::media_info::get_media_info,
            // Translate
            commands::translate::translate_subtitles,
            commands::translate::clear_translation,
            // OpenSubtitles
            commands::opensubtitles::search_subtitles,
            commands::opensubtitles::download_subtitle,
            // Updates (signed self-update)
            commands::update::check_update,
            commands::update::install_update,
            commands::update::relaunch_app,
            commands::update::open_release_page,
            // Window
            commands::window::toggle_fullscreen,
            commands::window::set_always_on_top,
            commands::window::minimize_window,
            commands::window::maximize_window,
            commands::window::close_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Vayou");
}
