use sea_orm::*;
use tracing::info;

use crate::database::entities::{player, Player};
use crate::Error;

pub use player::Model as DotaPlayer;

pub async fn try_add_player(db: &DatabaseTransaction, player_id: i64) -> Result<DotaPlayer, Error> {
    if let Some(player) = query_player_by_id(db, player_id).await? {
        info!("Player found: {}", player.player_id);
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
