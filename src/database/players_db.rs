use sea_orm::*;
use tracing::info;

use crate::database::entities::{player, Player};
use crate::database::{database_access, player_servers_db};
use crate::Error;

pub use player::Model as DotaPlayer;

/// Creates a transaction to group inserts for Player & PlayerServer
pub async fn insert_player_and_server(
    guild_id: i64,
    player_id: i64,
    name: &str,
) -> Result<(), Error> {
    let txn = database_access::get_transaction().await?;

    info!(
        guild_id,
        player_id, name, "Inserting Player & Player Server",
    );

    try_add_player(&txn, player_id).await?;
    player_servers_db::insert_player_server(&txn, guild_id, player_id, name).await?;

    txn.commit().await?;
    Ok(())
}

async fn try_add_player(db: &DatabaseTransaction, player_id: i64) -> Result<DotaPlayer, Error> {
    if let Some(player) = query_player_by_id(db, player_id).await? {
        info!("Player found, not inserting: {}", player.player_id);
        Ok(player)
    } else {
        info!("Player not found. Adding new player: {}", player_id);
        insert_dota_player(db, player_id).await?;
        Ok(DotaPlayer { player_id })
    }
}

async fn query_player_by_id(
    db: &DatabaseTransaction,
    player_id: i64,
) -> Result<Option<DotaPlayer>, Error> {
    let row = Player::find_by_id(player_id).one(db).await?;
    Ok(row)
}

async fn insert_dota_player(db: &DatabaseTransaction, player_id: i64) -> Result<(), Error> {
    let new_player = player::ActiveModel {
        player_id: Set(player_id),
    };

    Player::insert(new_player).exec(db).await?;
    Ok(())
}
