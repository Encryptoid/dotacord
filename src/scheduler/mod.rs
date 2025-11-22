mod leaderboard_task;
mod reload_task;

use std::sync::Arc;
use std::time::Duration;

use chrono::{Datelike, Local, Timelike, Utc};
use poise::serenity_prelude as serenity;
use tokio::time;
use tracing::{error, info};

use crate::database::{schedule_events_db, servers_db};
use crate::leaderboard::duration::Duration as LeaderboardDuration;
use crate::{config::AppConfig, Error};

pub struct SchedulerContext {
    pub config: AppConfig,
    pub http: Arc<serenity::Http>,
}

pub fn spawn_scheduler(config: AppConfig, http: Arc<serenity::Http>) {
    if !config.scheduler.enabled {
        info!("Scheduler is disabled in configuration");
        return;
    }

    info!("Spawning scheduler tasks");
    let ctx = Arc::new(SchedulerContext { config, http });

    spawn_unified_task(ctx);
}

fn spawn_unified_task(ctx: Arc<SchedulerContext>) {
    let interval_mins = ctx.config.scheduler.timer_check_mins;
    info!(interval_mins, "Starting unified scheduler task");

    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(interval_mins * 60));
        loop {
            interval.tick().await;
            if let Err(e) = check_all_tasks(&ctx).await {
                error!(error = ?e, "Unified scheduler task failed");
            }
        }
    });
}

async fn check_all_tasks(ctx: &SchedulerContext) -> Result<(), Error> {
    info!("Checking scheduled tasks for all subscribed servers");

    let servers = servers_db::query_all_servers().await?;

    for server in servers {
        if let Err(e) = check_server_tasks(ctx, &server).await {
            error!(
                error = ?e,
                server_id = server.server_id,
                server_name = ?server.server_name,
                "Failed to check tasks for server"
            );
        }
    }

    Ok(())
}

async fn check_server_tasks(
    ctx: &SchedulerContext,
    server: &servers_db::DiscordServer,
) -> Result<(), Error> {
    let now = Utc::now().timestamp();

    if server.is_sub_reload == 1 {
        check_reload_task(ctx, server, now).await?;
    }

    if server.is_sub_week == 1 {
        check_leaderboard_week_task(ctx, server, now).await?;
    }

    if server.is_sub_month == 1 {
        check_leaderboard_month_task(ctx, server, now).await?;
    }

    Ok(())
}

async fn check_reload_task(
    ctx: &SchedulerContext,
    server: &servers_db::DiscordServer,
    now: i64,
) -> Result<(), Error> {
    if !is_in_reload_window(ctx) {
        return Ok(());
    }

    let last_event = schedule_events_db::query_last_event(
        server.server_id,
        schedule_events_db::EventType::Reload,
    )
    .await?;

    let interval_secs = ctx.config.scheduler.auto_reload_interval_minutes * 60;

    let should_reload = match last_event {
        None => true,
        Some(event) => (now - event.event_time) >= interval_secs as i64,
    };

    if should_reload {
        info!(
            server_id = server.server_id,
            server_name = ?server.server_name,
            "Performing scheduled reload"
        );

        reload_task::auto_reload(ctx, server).await?;

        schedule_events_db::insert_event(
            server.server_id,
            schedule_events_db::EventType::Reload,
            schedule_events_db::EventSource::Schedule,
            now,
        )
        .await?;
    }

    Ok(())
}

async fn check_leaderboard_week_task(
    ctx: &SchedulerContext,
    server: &servers_db::DiscordServer,
    now: i64,
) -> Result<(), Error> {
    let config = &ctx.config.scheduler;

    let (Some(configured_day), Some(configured_hour)) = (
        config.weekly_leaderboard_day,
        config.weekly_leaderboard_hour,
    ) else {
        return Ok(());
    };

    let utc_now = Utc::now();
    let weekday = utc_now.weekday().num_days_from_monday() + 1;
    let hour = utc_now.hour();

    if weekday != configured_day as u32 || hour != configured_hour as u32 {
        return Ok(());
    }

    let last_event = schedule_events_db::query_last_event(
        server.server_id,
        schedule_events_db::EventType::LeaderboardWeek,
    )
    .await?;

    let one_week_secs = 7 * 24 * 60 * 60;
    let should_publish = match last_event {
        None => true,
        Some(event) => (now - event.event_time) >= one_week_secs,
    };

    if should_publish {
        info!(
            server_id = server.server_id, server_name = ?server.server_name, "Puishing weekly leaderboard"
        );

        leaderboard_task::publish_leaderboard(ctx, server, LeaderboardDuration::Week).await?;

        schedule_events_db::insert_event(
            server.server_id,
            schedule_events_db::EventType::LeaderboardWeek,
            schedule_events_db::EventSource::Schedule,
            now,
        )
        .await?;
    }

    Ok(())
}

async fn check_leaderboard_month_task(
    ctx: &SchedulerContext,
    server: &servers_db::DiscordServer,
    now: i64,
) -> Result<(), Error> {
    let config = &ctx.config.scheduler;

    let (Some(configured_day), Some(configured_hour)) = (
        config.monthly_leaderboard_day,
        config.monthly_leaderboard_hour,
    ) else {
        return Ok(());
    };

    let utc_now = Utc::now();
    let day_of_month = utc_now.day();
    let hour = utc_now.hour();

    if day_of_month != configured_day as u32 || hour != configured_hour as u32 {
        return Ok(());
    }

    let last_event = schedule_events_db::query_last_event(
        server.server_id,
        schedule_events_db::EventType::LeaderboardMonth,
    )
    .await?;

    let one_month_secs = 30 * 24 * 60 * 60;
    let should_publish = match last_event {
        None => true,
        Some(event) => (now - event.event_time) >= one_month_secs,
    };

    if should_publish {
        info!(
            server_id = server.server_id,
            server_name = ?server.server_name,
            "Publishing monthly leaderboard"
        );

        leaderboard_task::publish_leaderboard(ctx, server, LeaderboardDuration::Month).await?;

        schedule_events_db::insert_event(
            server.server_id,
            schedule_events_db::EventType::LeaderboardMonth,
            schedule_events_db::EventSource::Schedule,
            now,
        )
        .await?;
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
