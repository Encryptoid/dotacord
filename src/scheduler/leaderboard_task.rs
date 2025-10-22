use std::sync::Arc;

use chrono::{Datelike, Timelike, Utc};
use poise::serenity_prelude as serenity;
use tracing::{error, info, warn};

use crate::database::{database_access, player_servers_db, servers_db};
use crate::leaderboard::duration::Duration;
use crate::leaderboard::leaderboard_stats;
use crate::scheduler::SchedulerContext;
use crate::Error;

#[tracing::instrument(level = "info", skip(ctx))]
pub async fn check_and_publish_leaderboards(ctx: Arc<SchedulerContext>) -> Result<(), Error> {
    let now = Utc::now();
    let weekday = now.weekday().num_days_from_monday() + 1;
    let day_of_month = now.day();
    let hour = now.hour();

    let config = &ctx.config.scheduler;

    check_weekly_leaderboard(ctx.clone(), weekday, hour, config).await?;
    check_monthly_leaderboard(ctx.clone(), day_of_month, hour, config).await?;

    Ok(())
}

async fn check_weekly_leaderboard(
    ctx: Arc<SchedulerContext>,
    weekday: u32,
    hour: u32,
    config: &crate::config::SchedulerConfig,
) -> Result<(), Error> {
    if let (Some(configured_day), Some(configured_hour)) =
        (config.weekly_leaderboard_day, config.weekly_leaderboard_hour)
    {
        if weekday == configured_day as u32 && hour == configured_hour as u32 {
            info!("Weekly leaderboard trigger matched, publishing");
            publish_leaderboard(ctx, Duration::Week).await?;
        }
    }
    Ok(())
}

async fn check_monthly_leaderboard(
    ctx: Arc<SchedulerContext>,
    day_of_month: u32,
    hour: u32,
    config: &crate::config::SchedulerConfig,
) -> Result<(), Error> {
    if let (Some(configured_day), Some(configured_hour)) = (
        config.monthly_leaderboard_day,
        config.monthly_leaderboard_hour,
    ) {
        if day_of_month == configured_day as u32 && hour == configured_hour as u32 {
            info!("Monthly leaderboard trigger matched, publishing");
            publish_leaderboard(ctx, Duration::Month).await?;
        }
    }
    Ok(())
}

#[tracing::instrument(level = "info", skip(ctx))]
async fn publish_leaderboard(ctx: Arc<SchedulerContext>, duration: Duration) -> Result<(), Error> {
    info!(duration = ?duration, "Publishing leaderboard");
    
    let mut conn = database_access::get_new_connection().await?;
    let servers = servers_db::query_all_servers(&mut conn).await?;

    if servers.is_empty() {
        info!("No servers registered, skipping leaderboard publication");
        return Ok(());
    }

    for server in servers {
        if let Err(e) = publish_to_server(ctx.clone(), &mut conn, server, duration).await {
            error!(error = ?e, "Failed to publish leaderboard to server");
        }
    }

    Ok(())
}

async fn publish_to_server(
    ctx: Arc<SchedulerContext>,
    conn: &mut sqlx::SqliteConnection,
    server: servers_db::DiscordServer,
    duration: Duration,
) -> Result<(), Error> {
    let channel_id_value = match server.channel_id {
        Some(id) => id,
        None => {
            warn!(
                server_id = server.server_id,
                server_name = ?server.server_name,
                "Server has no channel_id configured, skipping"
            );
            return Ok(());
        }
    };

    let channel = get_channel(&ctx, channel_id_value, &server).await?;
    let players = get_server_players(conn, &server).await?;
    
    if players.is_empty() {
        info!(
            server_id = server.server_id,
            server_name = ?server.server_name,
            "No players registered for server, skipping"
        );
        return Ok(());
    }

    let messages = generate_leaderboard_messages(conn, players, duration, &server).await?;
    
    if messages.is_empty() {
        info!(
            server_id = server.server_id,
            server_name = ?server.server_name,
            duration = ?duration,
            "No matches found for leaderboard period"
        );
        return Ok(());
    }

    send_leaderboard_messages(&ctx, &channel, &server, channel_id_value, messages).await?;
    
    info!(
        server_id = server.server_id,
        server_name = ?server.server_name,
        channel_id = channel_id_value,
        duration = ?duration,
        "Successfully published leaderboard"
    );

    Ok(())
}

async fn get_channel(
    ctx: &SchedulerContext,
    channel_id_value: i64,
    server: &servers_db::DiscordServer,
) -> Result<serenity::Channel, Error> {
    let channel_id = serenity::ChannelId::new(channel_id_value as u64);
    ctx.http.get_channel(channel_id.into()).await.map_err(|e| {
        error!(
            server_id = server.server_id,
            server_name = ?server.server_name,
            channel_id = channel_id_value,
            error = ?e,
            "Failed to get channel"
        );
        e.into()
    })
}

async fn get_server_players(
    conn: &mut sqlx::SqliteConnection,
    server: &servers_db::DiscordServer,
) -> Result<Vec<player_servers_db::PlayerServer>, Error> {
    player_servers_db::query_server_players(conn, Some(server.server_id)).await
}

async fn generate_leaderboard_messages(
    conn: &mut sqlx::SqliteConnection,
    players: Vec<player_servers_db::PlayerServer>,
    duration: Duration,
    server: &servers_db::DiscordServer,
) -> Result<Vec<String>, Error> {
    let end_utc = Utc::now();
    let start_utc = duration.start_date(end_utc);
    let duration_label = duration.to_label();

    leaderboard_stats::get_leaderboard_messages(
        conn,
        players,
        &start_utc,
        &end_utc,
        &duration_label,
    )
    .await
    .map_err(|e| {
        error!(
            server_id = server.server_id,
            server_name = ?server.server_name,
            duration = ?duration,
            error = ?e,
            "Failed to generate leaderboard"
        );
        e
    })
}

async fn send_leaderboard_messages(
    ctx: &SchedulerContext,
    channel: &serenity::Channel,
    server: &servers_db::DiscordServer,
    channel_id_value: i64,
    messages: Vec<String>,
) -> Result<(), Error> {
    let batches = batch_contents(messages, ctx.config.max_message_length);
    
    for batch in batches {
        let message = serenity::CreateMessage::default()
            .content(batch)
            .flags(serenity::MessageFlags::SUPPRESS_EMBEDS);
        
        if let Err(e) = channel.id().send_message(&ctx.http, message).await {
            error!(
                server_id = server.server_id,
                server_name = ?server.server_name,
                channel_id = channel_id_value,
                error = ?e,
                "Failed to send leaderboard message"
            );
        }
    }
    
    Ok(())
}

fn batch_contents(contents: Vec<String>, max_length: usize) -> Vec<String> {
    let mut batches = Vec::new();
    let mut current_batch = String::new();
    
    for content in contents {
        let separator_len = if current_batch.is_empty() { 0 } else { 1 };
        if current_batch.len() + content.len() + separator_len > max_length {
            batches.push(current_batch);
            current_batch = content;
        } else {
            current_batch.push_str(&content);
        }
    }
    
    if !current_batch.is_empty() {
        batches.push(current_batch);
    }
    
    batches
}
