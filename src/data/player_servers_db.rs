use sqlx::{FromRow, SqliteConnection};
use tracing::info;

use crate::Error;

#[derive(Debug, Clone, FromRow)]
pub(crate) struct PlayerServer {
    pub player_id: i64,
    pub server_id: i64,
    pub user_id: i64,
    pub discord_name: String,
    pub player_name: Option<String>,
}

impl PlayerServer {
    /// Returns the display name for this player, preferring player_name if set, otherwise discord_name
    pub fn display_name(&self) -> &str {
        self.player_name.as_deref().unwrap_or(&self.discord_name)
    }
}

pub async fn query_server_players(
    conn: &mut SqliteConnection,
    server_id: Option<i64>,
) -> Result<Vec<PlayerServer>, Error> {
    info!("Querying player servers from database");
    let rows: Vec<PlayerServer> = match server_id {
        None => {
            sqlx::query_as(
                r#"
                    SELECT
                        player_id,
                        server_id,
                        user_id,
                        discord_name,
                        player_name
                    FROM player_servers
                "#,
            )
            .fetch_all(conn)
            .await?
        }
        Some(server_id) => {
            sqlx::query_as(
                r#"
                    SELECT
                        player_id,
                        server_id,
                        user_id,
                        discord_name,
                        player_name
                    FROM player_servers
                    WHERE server_id = ?
                "#,
            )
            .bind(server_id)
            .fetch_all(conn)
            .await?
        }
    };

    info!(Count = rows.len(), "Retrieved server players from database");
    Ok(rows)
}

pub async fn remove_server_player_by_name(
    conn: &mut SqliteConnection,
    server_id: i64,
    player_name: &str,
) -> Result<bool, Error> {
    info!(
        "Attempting to remove PlayerServer for ServerId: {}, PlayerName: {}",
        server_id, player_name
    );

    let result = sqlx::query(
        r#"
            DELETE FROM player_servers
            WHERE server_id = ? AND LOWER(player_name) = LOWER(?)
        "#,
    )
    .bind(server_id as i64)
    .bind(player_name)
    .execute(conn)
    .await?;

    let removed = result.rows_affected() > 0;

    if removed {
        info!(
            RowsAffected = result.rows_affected(),
            "Removed PlayerServer for ServerId: {}, PlayerName: {}", server_id, player_name
        );
    } else {
        info!(
            "No PlayerServer found for removal. ServerId: {}, PlayerName: {}",
            server_id, player_name
        );
    }

    Ok(removed)
}

pub async fn insert_player_server(
    conn: &mut SqliteConnection,
    server_id: i64,
    player_id: i64,
    user_id: i64,
    discord_name: &str,
    player_name: Option<&str>,
) -> Result<(), Error> {
    sqlx::query(
        r#"
            INSERT INTO player_servers (server_id, player_id, user_id, discord_name, player_name)
            VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(server_id as i64)
    .bind(player_id as i64)
    .bind(user_id as i64)
    .bind(discord_name)
    .bind(player_name)
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn remove_server_player_by_user_id(
    conn: &mut SqliteConnection,
    server_id: i64,
    user_id: i64,
) -> Result<bool, Error> {
    info!(
        "Attempting to remove PlayerServer for ServerId: {}, UserId: {}",
        server_id, user_id
    );

    let result = sqlx::query(
        r#"
            DELETE FROM player_servers
            WHERE server_id = ? AND user_id = ?
        "#,
    )
    .bind(server_id as i64)
    .bind(user_id as i64)
    .execute(conn)
    .await?;

    let removed = result.rows_affected() > 0;

    if removed {
        info!(
            RowsAffected = result.rows_affected(),
            "Removed PlayerServer for ServerId: {}, UserId: {}", server_id, user_id
        );
    } else {
        info!(
            "No PlayerServer found for removal. ServerId: {}, UserId: {}",
            server_id, user_id
        );
    }

    Ok(removed)
}

pub async fn rename_server_player_by_user_id(
    conn: &mut SqliteConnection,
    server_id: i64,
    user_id: i64,
    new_name: &str,
) -> Result<bool, Error> {
    info!(
        "Attempting to rename PlayerServer for ServerId: {}, UserId: {} to NewName: {}",
        server_id, user_id, new_name
    );

    let result = sqlx::query(
        r#"
            UPDATE player_servers
            SET player_name = ?
            WHERE server_id = ? AND user_id = ?
        "#,
    )
    .bind(new_name)
    .bind(server_id as i64)
    .bind(user_id as i64)
    .execute(conn)
    .await?;

    let renamed = result.rows_affected() > 0;

    if renamed {
        info!(
            RowsAffected = result.rows_affected(),
            "Renamed PlayerServer for ServerId: {}, UserId: {} to NewName: {}",
            server_id,
            user_id,
            new_name
        );
    } else {
        info!(
            "No PlayerServer found for rename. ServerId: {}, UserId: {}",
            server_id, user_id
        );
    }

    Ok(renamed)
}
