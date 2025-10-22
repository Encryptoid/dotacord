mod heartbeat_task;
mod leaderboard_task;
mod reload_task;

use std::sync::Arc;
use std::time::Duration;

use poise::serenity_prelude as serenity;
use tokio::time;
use tracing::{error, info};

use crate::config::AppConfig;

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

    spawn_heartbeat_task(ctx.clone());
    spawn_auto_reload_task(ctx.clone());
    spawn_leaderboard_checker_task(ctx.clone());
}

fn spawn_heartbeat_task(ctx: Arc<SchedulerContext>) {
    let interval_minutes = ctx.config.scheduler.heartbeat_interval_minutes;
    info!(
        interval_minutes = interval_minutes,
        "Starting heartbeat task"
    );

    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(interval_minutes * 60));
        loop {
            interval.tick().await;
            if let Err(e) = heartbeat_task::heartbeat().await {
                error!(error = ?e, "Heartbeat task failed");
            }
        }
    });
}

fn spawn_auto_reload_task(ctx: Arc<SchedulerContext>) {
    let interval_minutes = ctx.config.scheduler.auto_reload_interval_minutes;
    info!(
        interval_minutes = interval_minutes,
        "Starting auto-reload task"
    );

    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(interval_minutes * 60));
        loop {
            interval.tick().await;
            if let Err(e) = reload_task::auto_reload(ctx.clone()).await {
                error!(error = ?e, "Auto-reload task failed");
            }
        }
    });
}

fn spawn_leaderboard_checker_task(ctx: Arc<SchedulerContext>) {
    info!("Starting leaderboard checker task");

    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(3600));
        loop {
            interval.tick().await;
            if let Err(e) = leaderboard_task::check_and_publish_leaderboards(ctx.clone()).await {
                error!(error = ?e, "Leaderboard checker task failed");
            }
        }
    });
}
