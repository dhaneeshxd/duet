use crate::errors::AppResult;
use rusqlite::Connection;

pub fn run_migrations(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS tracks (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            path            TEXT NOT NULL UNIQUE,
            title           TEXT NOT NULL DEFAULT '',
            artist          TEXT NOT NULL DEFAULT '',
            album           TEXT NOT NULL DEFAULT '',
            duration_secs   INTEGER NOT NULL DEFAULT 0,
            added_at        INTEGER NOT NULL DEFAULT (unixepoch())
        );

        CREATE TABLE IF NOT EXISTS playlists (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            name        TEXT NOT NULL UNIQUE,
            created_at  INTEGER NOT NULL DEFAULT (unixepoch())
        );

        CREATE TABLE IF NOT EXISTS playlist_tracks (
            playlist_id INTEGER NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
            track_id    INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
            position    INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (playlist_id, track_id)
        );

        CREATE TABLE IF NOT EXISTS folders (
            id    INTEGER PRIMARY KEY AUTOINCREMENT,
            path  TEXT NOT NULL UNIQUE
        );
    ",
    )?;
    Ok(())
}
