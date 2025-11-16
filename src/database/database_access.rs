use std::path::Path;
use std::sync::OnceLock;

use sea_orm::{
    ConnectOptions, Database, DatabaseConnection, DatabaseTransaction, TransactionTrait,
};
use tracing::info;

use crate::{fmt, Error};

static SEA_ORM_CONNECTION: OnceLock<DatabaseConnection> = OnceLock::new();

pub async fn init_database(path: &Path) -> Result<(), Error> {
    let url = fmt!("sqlite://{}?mode=rwc", path.display());

    let mut opt = ConnectOptions::new(url);
    opt.sqlx_logging(false)
        .sqlx_logging_level(tracing::log::LevelFilter::Off);

    let conn = Database::connect(opt).await?;

    SEA_ORM_CONNECTION.set(conn).map_err(|_already| {
        Box::new(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "Database connection already initialized",
        )) as Error
    })?;

    info!("Database connection initialized");
    Ok(())
}

pub async fn get_transaction() -> Result<DatabaseTransaction, Error> {
    let conn = SEA_ORM_CONNECTION.get().ok_or_else(|| {
        Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Database connection not initialized. Call init_database(...) at startup.",
        )) as Error
    })?;
    let txn = conn.begin().await?;
    Ok(txn)
}
