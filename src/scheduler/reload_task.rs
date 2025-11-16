use chrono::{Local, Timelike};
use tracing::info;

use crate::api::api_wrapper::{self, ReloadPlayerStat};
use crate::database::{player_servers_db, servers_db};
use crate::scheduler::SchedulerContext;
use crate::Error;

#[tracing::instrument(level = "info", skip(ctx))]
pub async fn auto_reload(ctx: &SchedulerContext) -> Result<(), Error> {
    if !is_in_reload_window(ctx) {
        return Ok(());
    }

    info!("Starting auto-reload of player matches");
    let servers = servers_db::query_all_servers().await?;
    for server in servers {
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
    }

    Ok(())
}

fn is_in_reload_window(ctx: &SchedulerContext) -> bool {
    let local_time = Local::now();
    let current_hour = local_time.hour() as u8;

    let start_hour = ctx.config.scheduler.auto_reload_start_hour;
    let end_hour = ctx.config.scheduler.auto_reload_end_hour;

    let is_in_window = if start_hour <= end_hour {
        current_hour >= start_hour && current_hour < end_hour
    } else {
        current_hour >= start_hour || current_hour < end_hour
    };

    if !is_in_window {
        info!(
            current_hour = current_hour,
            start_hour = start_hour,
            end_hour = end_hour,
            "Outside reload window, skipping auto-reload"
        );
    }

    is_in_window
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
