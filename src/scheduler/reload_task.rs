use serenity::model::event;
use tracing::{info, info_span, span};

use crate::api::api_wrapper::{self, ReloadPlayerStat};
use crate::database::{player_servers_db, servers_db};
use crate::scheduler::SchedulerContext;
use crate::{seq_span, Error};

#[tracing::instrument(level = "info", skip(_ctx, server))]
pub async fn auto_reload(
    _ctx: &SchedulerContext,
    server: &servers_db::DiscordServer,
) -> Result<(), Error> {
    // let span = info_span!("auto_reload",
    // span_name = "auto_reload",
    // server_id = server.server_id, name = %server.server_name);
    let span = seq_span!("auto_reload");
    let _enter = span.enter();

    info!("About to fetch players");
    reload_players(server).await?;
    Ok(())
}

async fn reload_players(server: &servers_db::DiscordServer) -> Result<(), Error> {
    info!(server_id = server.server_id, "Reloading players for server");
    let players = player_servers_db::query_server_players(server.server_id).await?;

    if players.is_empty() {
        info!("No players registered, skipping auto-reload");
        return Ok(());
    }

    let stats = reload_all_players(players).await;

    let success_count = stats
        .iter()
        .filter(|s| matches!(s.result, Ok(Some(_))))
        .count();
    let failure_count = stats.iter().filter(|s| s.result.is_err()).count();
    let removed_count = stats
        .iter()
        .filter(|s| matches!(s.result, Ok(None)))
        .count();

    info!(
        success_count,
        failure_count, removed_count, server.server_name, "Completed auto-reload for server"
    );

    Ok(())
}

async fn reload_all_players(
    players: Vec<player_servers_db::PlayerServerModel>,
) -> Vec<ReloadPlayerStat> {
    let mut stats = Vec::new();

    for player in players {
        let stat = api_wrapper::reload_player(&player).await;
        stats.push(stat);
    }

    stats
}
