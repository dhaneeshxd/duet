use crate::db::queries::insert_track;
use crate::errors::AppResult;
use crate::library::metadata::extract_metadata;
use rusqlite::Connection;
use walkdir::WalkDir;

// Audio file extensions we support
const AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "wav", "ogg", "aac", "m4a", "opus", "wma"];

pub struct ScanResult {
    pub scanned: usize,
    pub inserted: usize,
    pub failed: usize,
}

pub fn scan_folder(conn: &Connection, folder_path: &str) -> AppResult<ScanResult> {
    let mut result = ScanResult {
        scanned: 0,
        inserted: 0,
        failed: 0,
    };

    // WalkDir recursively visits every file and folder
    for entry in WalkDir::new(folder_path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    // skip entries we can't read
    {
        // Skip directories we only want files
        if !entry.file_type().is_file() {
            continue;
        }

        // Get the file extension
        let ext = match entry.path().extension().and_then(|e| e.to_str()) {
            Some(e) => e.to_lowercase(),
            None => continue, // no extension skip
        };

        // Skip non audio files
        if !AUDIO_EXTENSIONS.contains(&ext.as_str()) {
            continue;
        }

        result.scanned += 1;

        let path = entry.path().to_string_lossy().to_string();

        // Extract metadata and insert into DB
        match extract_metadata(&path) {
            Ok(track) => match insert_track(conn, &track) {
                Ok(_) => result.inserted += 1,
                Err(_) => result.failed += 1,
            },
            Err(_) => {
                result.failed += 1;
            }
        }
    }

    Ok(result)
}
