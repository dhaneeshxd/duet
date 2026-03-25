mod audio;
mod db;
mod errors;
mod library;

use errors::AppResult;
use std::sync::{Arc, Mutex};
use tauri::Manager;

pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    pub audio: Arc<Mutex<audio::engine::AudioEngine>>,
    pub queue: Mutex<audio::queue::Queue>,
}

impl AppState {
    pub fn new(app_data_dir: &str) -> AppResult<Self> {
        let conn = db::open(app_data_dir)?;
        let engine = audio::engine::AudioEngine::new()?;
        let queue = audio::queue::Queue::new();
        Ok(AppState {
            db: Mutex::new(conn),
            audio: Arc::new(Mutex::new(engine)),
            queue: Mutex::new(queue),
        })
    }
}

#[tauri::command]
fn play_track(state: tauri::State<AppState>, track_id: i64) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    let all_tracks = db::queries::get_all_tracks(&conn).map_err(|e| e.to_string())?;
    let track = all_tracks
        .into_iter()
        .find(|t| t.id == track_id)
        .ok_or("Track not found".to_string())?;
    drop(conn);
    let mut audio = state.audio.lock().unwrap();
    audio.play(&track.path).map_err(|e| e.to_string())
}

#[tauri::command]
fn pause(state: tauri::State<AppState>) -> Result<(), String> {
    let audio = state.audio.lock().unwrap();
    audio.pause();
    Ok(())
}

#[tauri::command]
fn resume(state: tauri::State<AppState>) -> Result<(), String> {
    let audio = state.audio.lock().unwrap();
    audio.resume();
    Ok(())
}

#[tauri::command]
fn stop(state: tauri::State<AppState>) -> Result<(), String> {
    let mut audio = state.audio.lock().unwrap();
    audio.stop();
    Ok(())
}

#[tauri::command]
fn set_volume(state: tauri::State<AppState>, volume: f32) -> Result<(), String> {
    let mut audio = state.audio.lock().unwrap();
    audio.set_volume(volume);
    Ok(())
}

#[tauri::command]
fn seek_to(state: tauri::State<AppState>, secs: f64) -> Result<(), String> {
    let path = {
        let audio = state.audio.lock().unwrap();
        audio.current_path.clone()
    };
    let path = match path {
        Some(p) => p,
        None => return Ok(()),
    };
    let mut audio = state.audio.lock().unwrap();
    audio.play_from(&path, secs).map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
pub struct PlayerStatus {
    pub is_playing: bool,
    pub is_paused: bool,
    pub volume: f32,
    pub current_track: Option<db::queries::Track>,
}

#[tauri::command]
fn get_player_status(state: tauri::State<AppState>) -> Result<PlayerStatus, String> {
    let audio = state.audio.lock().unwrap();
    let queue = state.queue.lock().unwrap();
    let current_track = queue.current().cloned();
    Ok(PlayerStatus {
        is_playing: audio.is_playing(),
        is_paused: audio.is_paused(),
        volume: audio.volume(),
        current_track,
    })
}

#[tauri::command]
fn next_track(state: tauri::State<AppState>) -> Result<Option<db::queries::Track>, String> {
    let mut queue = state.queue.lock().unwrap();
    let next = queue.next().cloned();
    drop(queue);
    if let Some(ref track) = next {
        let mut audio = state.audio.lock().unwrap();
        audio.play(&track.path).map_err(|e| e.to_string())?;
    }
    Ok(next)
}

#[tauri::command]
fn prev_track(state: tauri::State<AppState>) -> Result<Option<db::queries::Track>, String> {
    let mut queue = state.queue.lock().unwrap();
    let prev = queue.previous().cloned();
    drop(queue);
    if let Some(ref track) = prev {
        let mut audio = state.audio.lock().unwrap();
        audio.play(&track.path).map_err(|e| e.to_string())?;
    }
    Ok(prev)
}

#[tauri::command]
fn load_queue(
    state: tauri::State<AppState>,
    track_ids: Vec<i64>,
    start_index: usize,
) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    let all_tracks = db::queries::get_all_tracks(&conn).map_err(|e| e.to_string())?;
    drop(conn);
    let queue_tracks: Vec<db::queries::Track> = track_ids
        .iter()
        .filter_map(|id| all_tracks.iter().find(|t| t.id == *id).cloned())
        .collect();
    let mut queue = state.queue.lock().unwrap();
    queue.load(queue_tracks, start_index);
    Ok(())
}

#[tauri::command]
fn get_tracks(state: tauri::State<AppState>) -> Result<Vec<db::queries::Track>, String> {
    let conn = state.db.lock().unwrap();
    db::queries::get_all_tracks(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn search_tracks(
    state: tauri::State<AppState>,
    query: String,
) -> Result<Vec<db::queries::Track>, String> {
    let conn = state.db.lock().unwrap();
    db::queries::search_tracks(&conn, &query).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_playlists(state: tauri::State<AppState>) -> Result<Vec<db::queries::Playlist>, String> {
    let conn = state.db.lock().unwrap();
    db::queries::get_all_playlists(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_playlist(state: tauri::State<AppState>, name: String) -> Result<i64, String> {
    let conn = state.db.lock().unwrap();
    db::queries::create_playlist(&conn, &name).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_playlist(state: tauri::State<AppState>, id: i64) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    db::queries::delete_playlist(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
fn add_track_to_playlist(
    state: tauri::State<AppState>,
    playlist_id: i64,
    track_id: i64,
    position: i64,
) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    db::queries::add_track_to_playlist(&conn, playlist_id, track_id, position)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_playlist_tracks(
    state: tauri::State<AppState>,
    playlist_id: i64,
) -> Result<Vec<db::queries::Track>, String> {
    let conn = state.db.lock().unwrap();
    db::queries::get_playlist_tracks(&conn, playlist_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_track_from_playlist(
    state: tauri::State<AppState>,
    playlist_id: i64,
    track_id: i64,
) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    db::queries::remove_track_from_playlist(&conn, playlist_id, track_id).map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
struct ScanResponse {
    scanned: usize,
    inserted: usize,
    failed: usize,
}

#[tauri::command]
fn scan_folder(state: tauri::State<AppState>, folder_path: String) -> Result<ScanResponse, String> {
    let conn = state.db.lock().unwrap();
    // Save folder path
    let _ = db::queries::add_folder(&conn, &folder_path);
    let result = library::scanner::scan_folder(&conn, &folder_path).map_err(|e| e.to_string())?;
    Ok(ScanResponse {
        scanned: result.scanned,
        inserted: result.inserted,
        failed: result.failed,
    })
}

#[tauri::command]
fn get_album_art(path: String) -> Result<Option<String>, String> {
    match library::metadata::extract_rich_metadata(&path) {
        Ok(rich) => {
            let b64 = rich.album_art.map(|bytes| {
                use base64::{engine::general_purpose, Engine as _};
                general_purpose::STANDARD.encode(&bytes)
            });
            Ok(b64)
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
fn get_lyrics(path: String) -> Result<Option<String>, String> {
    // First try embedded lyrics in audio file tags
    if let Ok(rich) = library::metadata::extract_rich_metadata(&path) {
        if rich.lyrics.is_some() {
            return Ok(rich.lyrics);
        }
    }

    // Fall back to .lrc sidecar file (same name, .lrc extension)
    let lrc_path = std::path::Path::new(&path).with_extension("lrc");

    if lrc_path.exists() {
        match std::fs::read_to_string(&lrc_path) {
            Ok(content) => return Ok(Some(content)),
            Err(e) => eprintln!("Failed to read lrc: {}", e),
        }
    }

    Ok(None)
}

#[tauri::command]
fn get_position() -> Result<f64, String> {
    Ok(0.0)
}
#[tauri::command]
fn clear_library(state: tauri::State<AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    conn.execute_batch("DELETE FROM tracks; DELETE FROM playlist_tracks;")
        .map_err(|e| e.to_string())
}
#[tauri::command]
fn get_audio_info(path: String) -> Result<Option<library::metadata::AudioInfo>, String> {
    match library::metadata::extract_rich_metadata(&path) {
        Ok(rich) => Ok(rich.audio_info),
        Err(e) => Err(e.to_string()),
    }
}
#[tauri::command]
fn set_queue_repeat(state: tauri::State<AppState>, mode: String) -> Result<(), String> {
    let mut queue = state.queue.lock().unwrap();

    match mode.as_str() {
        "one" => queue.repeat = crate::audio::queue::RepeatMode::One,
        "all" => queue.repeat = crate::audio::queue::RepeatMode::All,
        _ => queue.repeat = crate::audio::queue::RepeatMode::Off,
    }
    Ok(())
}
#[tauri::command]
fn list_audio_devices() -> Vec<String> {
    use rodio::cpal::traits::{DeviceTrait, HostTrait};
    let host = rodio::cpal::default_host();
    host.output_devices()
        .map(|devs| devs.filter_map(|d| d.name().ok()).collect())
        .unwrap_or_default()
}

#[tauri::command]
fn get_current_audio_device() -> String {
    use rodio::cpal::traits::{DeviceTrait, HostTrait};
    let host = rodio::cpal::default_host();
    host.default_output_device()
        .and_then(|d| d.name().ok())
        .unwrap_or_else(|| "Unknown".to_string())
}

#[tauri::command]
fn set_audio_device(state: tauri::State<AppState>, device_name: String) -> Result<(), String> {
    use audio::engine::spawn_sink_for_device;

    let new_sink = spawn_sink_for_device(&device_name).map_err(|e| e.to_string())?;

    let mut eng = state.audio.lock().unwrap();
    let was_playing = eng.is_playing();
    let path = eng.current_path.clone();
    let pos = eng.position();

    eng.manual_device = Some(device_name.clone());
    eng.preferred_device = Some(device_name.clone());
    new_sink.set_volume(eng.volume());
    eng.set_sink(new_sink);

    if was_playing {
        if let Some(p) = path {
            eng.play_from(&p, pos).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[tauri::command]
fn clear_audio_device_override(state: tauri::State<AppState>) -> Result<(), String> {
    let mut eng = state.audio.lock().unwrap();
    eng.manual_device = None;
    eng.preferred_device = None;
    Ok(())
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");
            std::fs::create_dir_all(&app_data_dir)?;
            let dir_str = app_data_dir.to_string_lossy().to_string();
            let state = AppState::new(&dir_str).expect("failed to initialize app state");
            let audio_arc = Arc::clone(&state.audio);
            app.manage(state);
            audio::engine::AudioEngine::start_device_watcher(audio_arc);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            play_track,
            pause,
            resume,
            stop,
            set_volume,
            seek_to,
            get_player_status,
            next_track,
            prev_track,
            load_queue,
            get_tracks,
            search_tracks,
            get_playlists,
            create_playlist,
            delete_playlist,
            add_track_to_playlist,
            get_playlist_tracks,
            remove_track_from_playlist,
            scan_folder,
            get_album_art,
            get_lyrics,
            get_position,
            get_folders,
            remove_folder,
            rescan_all,
            clear_library,
            get_audio_info,
            set_queue_repeat,
            list_audio_devices,
            get_current_audio_device,
            set_audio_device,
            clear_audio_device_override,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
#[tauri::command]
fn get_folders(state: tauri::State<AppState>) -> Result<Vec<db::queries::Folder>, String> {
    let conn = state.db.lock().unwrap();
    db::queries::get_folders(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_folder(state: tauri::State<AppState>, id: i64) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    let folders = db::queries::get_folders(&conn).map_err(|e| e.to_string())?;
    let folder = folders
        .iter()
        .find(|f| f.id == id)
        .ok_or("Folder not found".to_string())?;
    let path = folder.path.clone();
    db::queries::remove_folder(&conn, id).map_err(|e| e.to_string())?;
    db::queries::delete_tracks_by_folder(&conn, &path).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn rescan_all(state: tauri::State<AppState>) -> Result<ScanResponse, String> {
    let conn = state.db.lock().unwrap();
    let folders = db::queries::get_folders(&conn).map_err(|e| e.to_string())?;
    drop(conn);

    let mut total = ScanResponse {
        scanned: 0,
        inserted: 0,
        failed: 0,
    };
    for folder in folders {
        let conn = state.db.lock().unwrap();
        if let Ok(result) = library::scanner::scan_folder(&conn, &folder.path) {
            total.scanned += result.scanned;
            total.inserted += result.inserted;
            total.failed += result.failed;
        }
    }
    Ok(total)
}
