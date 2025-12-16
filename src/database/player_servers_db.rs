use sea_orm::*;
use tracing::info;

use crate::database::database_access;
use crate::database::entities::{player_server, PlayerServer};
use crate::Error;

pub use player_server::Model as PlayerServerModel;

pub async fn query_server_players(server_id: i64) -> Result<Vec<PlayerServerModel>, Error> {
    info!("Querying player servers from database");
    let txn = database_access::get_transaction().await?;

    let rows = PlayerServer::find()
        .filter(player_server::Column::ServerId.eq(server_id))
        .all(&txn)
        .await?;

    info!(
        server_id,
        count = rows.len(),
        "Retrieved server players from database"
    );
    Ok(rows)
}

pub async fn insert_player_server(
    db: &DatabaseTransaction,
    server_id: i64,
    player_id: i64,
    player_name: Option<String>,
    discord_user_id: Option<i64>,
    discord_name: String,
) -> Result<(), Error> {
    let new_player_server = player_server::ActiveModel {
        server_id: Set(server_id),
        player_id: Set(player_id),
        player_name: Set(player_name),
        discord_user_id: Set(discord_user_id),
        discord_name: Set(discord_name),
    };

    PlayerServer::insert(new_player_server).exec(db).await?;
    Ok(())
}

pub async fn remove_server_player_by_user_id(
    db: &DatabaseTransaction,
    server_id: i64,
    user_id: i64,
) -> Result<bool, Error> {
    info!(
        "Attempting to remove PlayerServer for ServerId: {}, UserId: {}",
        server_id, user_id
    );

    let result = PlayerServer::delete_many()
        .filter(player_server::Column::ServerId.eq(server_id))
        .filter(player_server::Column::PlayerId.eq(user_id))
        .exec(db)
        .await?;

    let removed = result.rows_affected > 0;

    if removed {
        info!(
            RowsAffected = result.rows_affected,
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
    db: &DatabaseTransaction,
    server_id: i64,
    user_id: i64,
    new_name: &str,
) -> Result<bool, Error> {
    info!(
        "Attempting to rename PlayerServer for ServerId: {}, UserId: {} to NewName: {}",
        server_id, user_id, new_name
    );

    let player_server = PlayerServer::find()
        .filter(player_server::Column::ServerId.eq(server_id))
        .filter(player_server::Column::PlayerId.eq(user_id))
        .one(db)
        .await?;

    match player_server {
        Some(ps) => {
            let mut ps_active: player_server::ActiveModel = ps.into();
            ps_active.player_name = Set(Some(new_name.to_string()));
            ps_active.update(db).await?;

            info!(
                "Renamed PlayerServer for ServerId: {}, UserId: {} to NewName: {}",
                server_id, user_id, new_name
            );
            Ok(true)
        }
        None => {
            info!(
                "No PlayerServer found for rename. ServerId: {}, UserId: {}",
                server_id, user_id
            );
            Ok(false)
        }
    }
}
