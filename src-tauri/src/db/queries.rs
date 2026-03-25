use crate::errors::AppResult;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: i64,
    pub path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_secs: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Playlist {
    pub id: i64,
    pub name: String,
    pub created_at: i64,
}

// Track queries

pub fn insert_track(conn: &Connection, track: &Track) -> AppResult<i64> {
    conn.execute(
        "INSERT OR IGNORE INTO tracks (path, title, artist, album, duration_secs)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            track.path,
            track.title,
            track.artist,
            track.album,
            track.duration_secs
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_all_tracks(conn: &Connection) -> AppResult<Vec<Track>> {
    let mut stmt = conn.prepare(
        "SELECT id, path, title, artist, album, duration_secs
         FROM tracks
         ORDER BY artist, album, title",
    )?;
    let tracks = stmt
        .query_map([], |row| {
            Ok(Track {
                id: row.get(0)?,
                path: row.get(1)?,
                title: row.get(2)?,
                artist: row.get(3)?,
                album: row.get(4)?,
                duration_secs: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(tracks)
}

pub fn search_tracks(conn: &Connection, query: &str) -> AppResult<Vec<Track>> {
    let pattern = format!("%{}%", query);
    let mut stmt = conn.prepare(
        "SELECT id, path, title, artist, album, duration_secs
         FROM tracks
         WHERE title LIKE ?1 OR artist LIKE ?1 OR album LIKE ?1
         ORDER BY artist, title",
    )?;

    let tracks = stmt
        .query_map(params![pattern], |row| {
            Ok(Track {
                id: row.get(0)?,
                path: row.get(1)?,
                title: row.get(2)?,
                artist: row.get(3)?,
                album: row.get(4)?,
                duration_secs: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(tracks)
}

// Playlist queries

pub fn create_playlist(conn: &Connection, name: &str) -> AppResult<i64> {
    conn.execute("INSERT INTO playlists (name) VALUES (?1)", params![name])?;
    Ok(conn.last_insert_rowid())
}

pub fn get_all_playlists(conn: &Connection) -> AppResult<Vec<Playlist>> {
    let mut stmt = conn.prepare("SELECT id, name, created_at FROM playlists ORDER BY name")?;

    let playlists = stmt
        .query_map([], |row| {
            Ok(Playlist {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(playlists)
}

pub fn delete_playlist(conn: &Connection, id: i64) -> AppResult<()> {
    // playlist_tracks rows auto-delete via ON DELETE CASCADE
    conn.execute("DELETE FROM playlists WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn add_track_to_playlist(
    conn: &Connection,
    playlist_id: i64,
    track_id: i64,
    position: i64,
) -> AppResult<()> {
    conn.execute(
        "INSERT OR IGNORE INTO playlist_tracks (playlist_id, track_id, position)
         VALUES (?1, ?2, ?3)",
        params![playlist_id, track_id, position],
    )?;
    Ok(())
}

pub fn get_playlist_tracks(conn: &Connection, playlist_id: i64) -> AppResult<Vec<Track>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.path, t.title, t.artist, t.album, t.duration_secs
         FROM tracks t
         JOIN playlist_tracks pt ON pt.track_id = t.id
         WHERE pt.playlist_id = ?1
         ORDER BY pt.position",
    )?;

    let tracks = stmt
        .query_map(params![playlist_id], |row| {
            Ok(Track {
                id: row.get(0)?,
                path: row.get(1)?,
                title: row.get(2)?,
                artist: row.get(3)?,
                album: row.get(4)?,
                duration_secs: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(tracks)
}

pub fn remove_track_from_playlist(
    conn: &Connection,
    playlist_id: i64,
    track_id: i64,
) -> AppResult<()> {
    conn.execute(
        "DELETE FROM playlist_tracks WHERE playlist_id = ?1 AND track_id = ?2",
        params![playlist_id, track_id],
    )?;
    Ok(())
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: i64,
    pub path: String,
}

pub fn add_folder(conn: &Connection, path: &str) -> AppResult<i64> {
    conn.execute(
        "INSERT OR IGNORE INTO folders (path) VALUES (?1)",
        params![path],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_folders(conn: &Connection) -> AppResult<Vec<Folder>> {
    let mut stmt = conn.prepare("SELECT id, path FROM folders ORDER BY path")?;
    let folders = stmt
        .query_map([], |row| {
            Ok(Folder {
                id: row.get(0)?,
                path: row.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(folders)
}

pub fn remove_folder(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM folders WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn delete_tracks_by_folder(conn: &Connection, folder_path: &str) -> AppResult<()> {
    conn.execute(
        "DELETE FROM tracks WHERE path LIKE ?1",
        params![format!("{}%", folder_path)],
    )?;
    Ok(())
}
