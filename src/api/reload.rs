use sea_orm::DatabaseConnection;
use tracing::info;

use crate::api::open_dota_api;
use crate::database::{player_matches_db, player_servers_db};
use crate::Error;

pub struct ReloadPlayerStat {
    pub player_id: i64,
    pub display_name: String,
    pub result: Result<Option<usize>, String>,
}

#[tracing::instrument(level = "trace", skip(db))]
pub async fn reload_player(
    db: &DatabaseConnection,
    player: &player_servers_db::PlayerServerModel,
) -> ReloadPlayerStat {
    info!(player_id = player.player_id, "Reloading matches for player");

    let result = async {
        let db_matches =
            player_matches_db::query_matches_by_player_id(db, player.player_id).await?;
        let api_matches = open_dota_api::get_player_matches(player.player_id).await?;

        info!(
            player_id = player.player_id,
            db_matches = db_matches.len(),
            api_matches = api_matches.len(),
            "Fetched matches from OpenDota API"
        );

        if api_matches.is_empty() {
            info!(
                player_id = player.player_id,
                server_id = player.server_id,
                "No matches found from OpenDota API. Player may need to be removed."
            );
            return Ok(None);
        }

        let match_count =
            import_new_matches(db, player.player_id, &db_matches, &api_matches).await?;

        info!(
            player_id = player.player_id,
            matches_inserted = match_count,
            "Finished reloading matches for player"
        );

        Ok(Some(match_count))
    }
    .await
    .map_err(|e: Error| e.to_string());

    ReloadPlayerStat {
        player_id: player.player_id,
        display_name: player.player_name.clone(),
        result,
    }
}

#[tracing::instrument(level = "trace", skip(db, db_matches, api_matches))]
async fn import_new_matches(
    db: &DatabaseConnection,
    player_id: i64,
    db_matches: &[player_matches_db::PlayerMatchModel],
    api_matches: &[open_dota_api::ApiPlayerMatch],
) -> Result<usize, Error> {
    let mut player_match_count = 0;

    for api_match in api_matches {
        if db_matches.iter().any(|m| m.match_id == api_match.match_id) {
            continue;
        }

        let Some(player_match) = player_matches_db::map_to_player_match(api_match, player_id)?
        else {
            continue;
        };

        player_matches_db::insert_player_match(db, player_match).await?;
        player_match_count += 1;
    }

    Ok(player_match_count)
}

pub async fn reload_all_players(
    db: &DatabaseConnection,
    players: Vec<player_servers_db::PlayerServerModel>,
) -> Vec<ReloadPlayerStat> {
    let mut stats = Vec::new();

    for player in players {
        let stat = reload_player(db, &player).await;
        stats.push(stat);
    }

    stats
}
