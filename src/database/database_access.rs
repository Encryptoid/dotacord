use std::path::Path;
use std::sync::OnceLock;

use sqlx::sqlite::{SqliteConnectOptions, SqliteConnection, SqliteJournalMode};
use sqlx::Connection;
use tracing::info;

use crate::Error;

static CONNECT_OPTIONS: OnceLock<SqliteConnectOptions> = OnceLock::new();

pub fn init_database(path: &Path) -> Result<(), Error> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(false)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true);

    CONNECT_OPTIONS.set(options).map_err(|_already| {
        Box::new(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "Database connection options already initialized",
        )) as Error
    })?;

    Ok(())
}

pub async fn get_new_connection() -> Result<SqliteConnection, Error> {
    let options = match CONNECT_OPTIONS.get() {
        Some(o) => o,
        None => {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Database connection options not initialized. Call database::init(...) at startup.",
            )) as Error);
        }
    };

    info!("Getting new database connection");

    return SqliteConnection::connect_with(options).await.map_err(|e| {
        Box::new(std::io::Error::other(format!(
            "Could not connect to database: {e}"
        ))) as Error
    });
}
