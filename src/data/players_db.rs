use sqlx::{Error as SqlxError, FromRow, SqliteConnection};
use tracing::info;

use crate::Error;

#[derive(Debug, Clone, FromRow)]
pub(crate) struct DotaPlayer {
    pub player_id: i64,
}

pub async fn try_add_player(
    conn: &mut SqliteConnection,
    player_id: i64,
) -> Result<DotaPlayer, Error> {
    if let Some(player) = query_player_by_id(conn, &player_id).await? {
        info!("Player found: {}", player.player_id);
        Ok(player)
    } else {
        info!("Player not found. Adding new player: {}", player_id);
        insert_dota_player(conn, &player_id).await?;
        Ok(DotaPlayer { player_id })
    }
}

async fn query_player_by_id(
    conn: &mut SqliteConnection,
    player_id: &i64,
) -> Result<Option<DotaPlayer>, SqlxError> {
    let row: Option<DotaPlayer> = sqlx::query_as(
        r#"
            SELECT
                player_id
            FROM players
            WHERE player_id = ?
        "#,
    )
    .bind(*player_id as i64)
    .fetch_optional(conn)
    .await?;

    Ok(row)
}

async fn insert_dota_player(conn: &mut SqliteConnection, player_id: &i64) -> Result<(), SqlxError> {
    sqlx::query(
        r#"
            INSERT INTO players (player_id)
            VALUES (?)
        "#,
    )
    .bind(*player_id as i64)
    .execute(conn)
    .await?;

    Ok(())
}
