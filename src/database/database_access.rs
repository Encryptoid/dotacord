use std::path::Path;
use std::sync::OnceLock;

use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sqlx::sqlite::{SqliteConnectOptions, SqliteConnection, SqliteJournalMode};
use sqlx::Connection;
use tracing::info;

use crate::Error;

static CONNECT_OPTIONS: OnceLock<SqliteConnectOptions> = OnceLock::new();
static SEA_ORM_CONNECTION: OnceLock<DatabaseConnection> = OnceLock::new();

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

pub async fn init_sea_orm_database(path: &Path) -> Result<(), Error> {
    let url = format!("sqlite://{}?mode=rwc", path.display());
    
    let mut opt = ConnectOptions::new(url);
    opt.sqlx_logging(false);
    
    let conn = Database::connect(opt).await?;
    
    SEA_ORM_CONNECTION.set(conn).map_err(|_already| {
        Box::new(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "SeaORM database connection already initialized",
        )) as Error
    })?;
    
    info!("SeaORM database connection initialized");
    Ok(())
}

pub fn get_sea_orm_connection() -> Result<&'static DatabaseConnection, Error> {
    SEA_ORM_CONNECTION.get().ok_or_else(|| {
        Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "SeaORM database connection not initialized. Call init_sea_orm_database(...) at startup.",
        )) as Error
    })
}
