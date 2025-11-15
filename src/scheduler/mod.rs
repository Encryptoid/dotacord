mod heartbeat_task;
mod leaderboard_task;
mod reload_task;

use std::sync::Arc;
use std::time::Duration;

use poise::serenity_prelude as serenity;
use tokio::time;
use tracing::{error, info};

use crate::{config::{AppConfig, SchedulerConfig}, scheduler::heartbeat_task::send_heartbeat};

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

    spawn_heartbeat_task(&ctx.config.scheduler);
    spawn_auto_reload_task(ctx.clone());
    spawn_leaderboard_checker_task(ctx);
}

pub fn spawn_heartbeat_task(schedule_config: &SchedulerConfig) {
    let interval_mins = schedule_config.heartbeat_interval_minutes;
    info!(interval_mins, "Starting heartbeat task");

    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(interval_mins * 60));
        loop {
            interval.tick().await;
            if let Err(e) = send_heartbeat().await {
                error!(error = ?e, "Heartbeat task failed");
            }
        }
    });
}

fn spawn_auto_reload_task(ctx: Arc<SchedulerContext>) {
    let interval_mins = ctx.config.scheduler.auto_reload_interval_minutes;
    info!(interval_mins, "Starting auto-reload task");

    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(interval_mins * 60));
        loop {
            interval.tick().await;
            if let Err(e) = reload_task::auto_reload(&ctx).await {
                error!(error = ?e, "Auto-reload task failed");
            }
        }
    });
}

fn spawn_leaderboard_checker_task(ctx: Arc<SchedulerContext>) {
    let interval_mins = ctx.config.scheduler.timer_check_mins;
    info!(interval_mins, "Starting leaderboard checker task");

    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(interval_mins * 60));
        loop {
            interval.tick().await;
            if let Err(e) = leaderboard_task::check_and_publish_leaderboards(&ctx).await {
                error!(error = ?e, "Leaderboard checker task failed");
            }
        }
    });
}
