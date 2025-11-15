use chrono::{Local, Timelike};
use tracing::info;

use crate::api::reload;
use crate::database::{database_access, player_servers_db};
use crate::scheduler::SchedulerContext;
use crate::Error;

#[tracing::instrument(level = "info", skip(ctx))]
pub async fn auto_reload(ctx: &SchedulerContext) -> Result<(), Error> {
    if !is_in_reload_window(ctx) {
        return Ok(());
    }

    info!("Starting auto-reload of player matches");
    let db = database_access::get_connection()?;
    let players = player_servers_db::query_server_players(db, None).await?;

    if players.is_empty() {
        info!("No players registered, skipping auto-reload");
        return Ok(());
    }

    let stats = reload::reload_all_players(db, players).await;

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
        failure_count, removed_count, "Completed auto-reload"
    );

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
