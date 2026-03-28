use sea_orm::*;

use crate::database::database_access;
use crate::database::entities::{player_rule, PlayerRule};
use crate::Error;

pub use player_rule::Model as PlayerRuleModel;

pub async fn query_rules_by_server(server_id: i64) -> Result<Vec<PlayerRuleModel>, Error> {
    let txn = database_access::get_transaction().await?;
    let rows = PlayerRule::find()
        .filter(player_rule::Column::ServerId.eq(server_id))
        .all(&txn)
        .await?;
    Ok(rows)
}

pub async fn query_rules_by_player(
    server_id: i64,
    discord_user_id: i64,
) -> Result<Vec<PlayerRuleModel>, Error> {
    let txn = database_access::get_transaction().await?;
    let rows = PlayerRule::find()
        .filter(player_rule::Column::ServerId.eq(server_id))
        .filter(player_rule::Column::DiscordUserId.eq(discord_user_id))
        .all(&txn)
        .await?;
    Ok(rows)
}

pub async fn insert_rule(
    txn: &DatabaseTransaction,
    server_id: i64,
    discord_user_id: i64,
    rule_text: &str,
) -> Result<(), Error> {
    let new_rule = player_rule::ActiveModel {
        id: NotSet,
        server_id: Set(server_id),
        discord_user_id: Set(discord_user_id),
        rule_text: Set(rule_text.to_string()),
    };
    PlayerRule::insert(new_rule).exec(txn).await?;
    Ok(())
}

pub async fn delete_rule(txn: &DatabaseTransaction, rule_id: i32) -> Result<bool, Error> {
    let result = PlayerRule::delete_by_id(rule_id).exec(txn).await?;
    Ok(result.rows_affected > 0)
}
