mod leaderboard_task;
mod reload_task;

use std::sync::Arc;
use std::time::Duration;

use chrono::{Datelike, Local, Timelike, Utc};
use poise::serenity_prelude as serenity;
use tokio::time;
use tracing::{error, info};

use crate::database::servers_db;
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

    start_schedule_task(ctx);
}

fn start_schedule_task(ctx: Arc<SchedulerContext>) {
    let interval_mins = ctx.config.scheduler.timer_check_mins;
    info!(interval_mins, "Starting main scheduler task");

    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(interval_mins * 60));
        loop {
            interval.tick().await;
            if let Err(e) = check_all_tasks(&ctx).await {
                error!(error = ?e, "Main scheduler task failed");
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
    if server.is_sub_reload == 1 {
        check_reload_task(ctx, server).await?;
    }

    if server.is_sub_week == 1 {
        check_leaderboard_week_task(ctx, server).await?;
    }

    if server.is_sub_month == 1 {
        check_leaderboard_month_task(ctx, server).await?;
    }

    Ok(())
}

async fn check_reload_task(
    ctx: &SchedulerContext,
    server: &servers_db::DiscordServer,
) -> Result<(), Error> {
    if !ctx.config.scheduler.auto_reload.enabled {
        return Ok(());
    }

    if !is_in_reload_window(ctx) {
        return Ok(());
    }

    let utc_now = Utc::now();
    let current_minute = utc_now.minute() as u64;
    let interval = ctx.config.scheduler.auto_reload.interval_minutes;

    if current_minute % interval != 0 {
        return Ok(());
    }

    info!(
        server_id = server.server_id,
        server_name = ?server.server_name,
        "Performing scheduled reload"
    );

    reload_task::auto_reload(ctx, server).await?;

    Ok(())
}

async fn check_leaderboard_week_task(
    ctx: &SchedulerContext,
    server: &servers_db::DiscordServer,
) -> Result<(), Error> {
    let config = &ctx.config.scheduler.weekly_leaderboard;

    if !config.enabled {
        return Ok(());
    }

    let target_day = match server.weekly_day {
        Some(d) => d as u32,
        None => return Ok(()),
    };
    let target_hour = match server.weekly_hour {
        Some(h) => h as u32,
        None => return Ok(()),
    };
    let target_minute = config.minute as u32;

    let utc_now = Utc::now();
    let weekday = utc_now.weekday().num_days_from_monday() + 1;
    let hour = utc_now.hour();
    let minute = utc_now.minute();

    info!(
        server_id = server.server_id,
        target_day,
        target_hour,
        target_minute,
        weekday,
        hour,
        minute,
        "Checking weekly leaderboard schedule"
    );

    if weekday != target_day || hour != target_hour || minute != target_minute {
        return Ok(());
    }

    info!(
        server_id = server.server_id,
        server_name = ?server.server_name,
        "Publishing weekly leaderboard"
    );

    leaderboard_task::publish_leaderboard(ctx, server, LeaderboardDuration::Week).await?;

    Ok(())
}

async fn check_leaderboard_month_task(
    ctx: &SchedulerContext,
    server: &servers_db::DiscordServer,
) -> Result<(), Error> {
    let config = &ctx.config.scheduler.monthly_leaderboard;

    if !config.enabled {
        return Ok(());
    }

    let target_hour = match server.monthly_hour {
        Some(h) => h as u32,
        None => return Ok(()),
    };
    let (target_week, target_weekday) = match (server.monthly_week, server.monthly_weekday) {
        (Some(w), Some(wd)) => (w, wd),
        _ => return Ok(()),
    };
    let target_minute = config.minute as u32;

    let utc_now = Utc::now();
    let hour = utc_now.hour();
    let minute = utc_now.minute();

    if hour != target_hour || minute != target_minute {
        return Ok(());
    }

    if !is_nth_weekday_of_month(utc_now, target_week, target_weekday) {
        return Ok(());
    }

    info!(
        server_id = server.server_id,
        server_name = ?server.server_name,
        "Publishing monthly leaderboard"
    );

    leaderboard_task::publish_leaderboard(ctx, server, LeaderboardDuration::Month).await?;

    Ok(())
}

fn is_nth_weekday_of_month(date: chrono::DateTime<Utc>, week: i32, weekday: i32) -> bool {
    let current_weekday = date.weekday().num_days_from_monday() + 1;
    if current_weekday != weekday as u32 {
        return false;
    }

    let day = date.day() as i32;

    if week == 5 {
        let days_in_month = days_in_month(date.year(), date.month());
        let days_remaining = days_in_month - day;
        return days_remaining < 7;
    }

    let week_of_month = (day - 1) / 7 + 1;
    week_of_month == week
}

fn days_in_month(year: i32, month: u32) -> i32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

fn is_in_reload_window(ctx: &SchedulerContext) -> bool {
    let local_time = Local::now();
    let current_hour = local_time.hour() as u8;

    let start_hour = ctx.config.scheduler.auto_reload.start_hour;
    let end_hour = ctx.config.scheduler.auto_reload.end_hour;

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

