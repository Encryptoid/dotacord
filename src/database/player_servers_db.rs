use sea_orm::*;
use tracing::info;

use crate::database::database_access;
use crate::database::entities::{player_server, PlayerServer};
use crate::database::players_db;
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
            ps_active.player_name = Set(if new_name.is_empty() {
                None
            } else {
                Some(new_name.to_string())
            });
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

pub async fn update_player_id(
    db: &DatabaseTransaction,
    server_id: i64,
    old_player_id: i64,
    new_player_id: i64,
) -> Result<bool, Error> {
    info!(
        server_id,
        old_player_id,
        new_player_id,
        "Attempting to update player ID"
    );

    let player_server = PlayerServer::find()
        .filter(player_server::Column::ServerId.eq(server_id))
        .filter(player_server::Column::PlayerId.eq(old_player_id))
        .one(db)
        .await?;

    match player_server {
        Some(ps) => {
            players_db::ensure_player_exists(db, new_player_id).await?;

            PlayerServer::delete_many()
                .filter(player_server::Column::ServerId.eq(server_id))
                .filter(player_server::Column::PlayerId.eq(old_player_id))
                .exec(db)
                .await?;

            let new_player_server = player_server::ActiveModel {
                server_id: Set(server_id),
                player_id: Set(new_player_id),
                player_name: Set(ps.player_name),
                discord_user_id: Set(ps.discord_user_id),
                discord_name: Set(ps.discord_name),
            };
            PlayerServer::insert(new_player_server).exec(db).await?;

            info!(
                server_id,
                old_player_id,
                new_player_id,
                "Player ID updated successfully"
            );
            Ok(true)
        }
        None => {
            info!(
                server_id,
                old_player_id,
                "No PlayerServer found for player ID update"
            );
            Ok(false)
        }
    }
}

pub async fn update_discord_user(
    db: &DatabaseTransaction,
    server_id: i64,
    player_id: i64,
    new_discord_user_id: i64,
    new_discord_name: String,
) -> Result<bool, Error> {
    info!(
        server_id,
        player_id,
        new_discord_user_id,
        new_discord_name,
        "Attempting to update discord user"
    );

    let player_server = PlayerServer::find()
        .filter(player_server::Column::ServerId.eq(server_id))
        .filter(player_server::Column::PlayerId.eq(player_id))
        .one(db)
        .await?;

    match player_server {
        Some(ps) => {
            let mut ps_active: player_server::ActiveModel = ps.into();
            ps_active.discord_user_id = Set(Some(new_discord_user_id));
            ps_active.discord_name = Set(new_discord_name.clone());
            ps_active.update(db).await?;

            info!(
                server_id,
                player_id,
                new_discord_user_id,
                new_discord_name,
                "Discord user updated successfully"
            );
            Ok(true)
        }
        None => {
            info!(
                server_id,
                player_id,
                "No PlayerServer found for discord user update"
            );
            Ok(false)
        }
    }
}
