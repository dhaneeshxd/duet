pub mod queries;
pub mod schema;

use crate::errors::AppResult;
use rusqlite::{Connection, OpenFlags};

pub fn open(app_data_dir: &str) -> AppResult<Connection> {
    // Build the full path: C:\Users\you\AppData\...\duet\duet.db
    let db_path = format!("{}\\duet.db", app_data_dir);

    let conn = Connection::open_with_flags(
        &db_path,
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;

    // Enable WAL mode much faster for concurrent reads
    conn.execute_batch(
        "
        PRAGMA journal_mode=WAL;
        PRAGMA foreign_keys=ON;
    ",
    )?;

    // Run migrations every startup safe, IF NOT EXISTS protects us
    schema::run_migrations(&conn)?;

    Ok(conn)
}
