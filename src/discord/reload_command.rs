use chrono::Utc;
use poise::ReplyHandle;

use crate::api::api_wrapper;
use crate::database::{command_events_db, player_servers_db};
use crate::discord::discord_helper::{self, CmdCtx, Ephemeral};
use crate::util::dates;
use crate::{Context, Error};

/// Refresh your own matches from the Dota API
#[poise::command(slash_command, prefix_command)]
#[tracing::instrument(level = "trace", skip(ctx))]
pub async fn refresh_matches(ctx: Context<'_>) -> Result<(), Error> {
    let cmd_ctx = discord_helper::get_command_ctx(ctx).await?;
    refresh_user_matches_command(&cmd_ctx).await?;
    Ok(())
}

/// [Admin] Refresh all matches for all players on the server
#[poise::command(slash_command, prefix_command)]
#[tracing::instrument(level = "trace", skip(ctx))]
pub async fn refresh_server_matches(ctx: Context<'_>) -> Result<(), Error> {
    let cmd_ctx = discord_helper::get_command_ctx(ctx).await?;
    refresh_server_matches_command(&cmd_ctx).await?;
    Ok(())
}

async fn refresh_user_matches_command(ctx: &CmdCtx<'_>) -> Result<(), Error> {
    let author = ctx.discord_ctx.author();
    let discord_user_id = author.id.get() as i64;

    let player =
        player_servers_db::query_player_by_discord_user(ctx.guild_id, discord_user_id).await?;
    let Some(player) = player else {
        ctx.reply(
            Ephemeral::Private,
            "You are not linked to a player on this server. Ask an admin to link your Discord account.",
        )
        .await?;
        return Ok(());
    };

    let cooldown_min = ctx.app_cfg.cooldowns.user_refresh_min;
    if let Some(remaining) = check_cooldown(
        ctx.guild_id,
        command_events_db::EventType::UserRefresh,
        Some(discord_user_id),
        cooldown_min,
    )
    .await?
    {
        let next_available = Utc::now().timestamp() + remaining;
        ctx.reply(
            Ephemeral::Private,
            format!(
                "Command on cooldown. You can use this again {}",
                dates::discord_relative_from_timestamp(next_available)
            ),
        )
        .await?;
        return Ok(());
    }

    let display_name = player
        .player_name
        .as_ref()
        .unwrap_or(&player.discord_name);
    let reply = ctx
        .reply(
            Ephemeral::Private,
            format!("Refreshing matches for {}...", display_name),
        )
        .await?;

    let stat = api_wrapper::reload_player(&player).await;

    let message = match stat.result {
        Ok(Some(count)) => format!("Refreshed {} matches for {}", count, stat.display_name),
        Ok(None) => format!(
            "No dota matches found for {} with PlayerId={}",
            stat.display_name, stat.player_id
        ),
        Err(e) => format!("Failed to refresh {}: {}", stat.display_name, e),
    };

    ctx.edit(&reply, message).await?;

    command_events_db::insert_event(
        ctx.guild_id,
        command_events_db::EventType::UserRefresh,
        discord_user_id,
        Utc::now().timestamp(),
    )
    .await?;

    Ok(())
}

async fn refresh_server_matches_command(ctx: &CmdCtx<'_>) -> Result<(), Error> {
    let author = ctx.discord_ctx.author();
    let discord_user_id = author.id.get() as i64;

    let cooldown_min = ctx.app_cfg.cooldowns.admin_refresh_min;
    if let Some(remaining) = check_cooldown(
        ctx.guild_id,
        command_events_db::EventType::AdminRefresh,
        None,
        cooldown_min,
    )
    .await?
    {
        let next_available = Utc::now().timestamp() + remaining;
        ctx.reply(
            Ephemeral::Private,
            format!(
                "Command on cooldown for this server. Available again {}",
                dates::discord_relative_from_timestamp(next_available)
            ),
        )
        .await?;
        return Ok(());
    }

    let players = player_servers_db::query_server_players(ctx.guild_id).await?;
    if players.is_empty() {
        ctx.discord_ctx
            .say("No players found for this server")
            .await?;
        return Ok(());
    }

    let reply = ctx
        .reply(
            Ephemeral::Public,
            format!(
                "Refreshing player matches for {} players. Message will be edited with progress updates.\n",
                players.len()
            ),
        )
        .await?;

    for player in &players {
        let stat = api_wrapper::reload_player(player).await;
        match stat.result {
            Ok(Some(count)) => {
                add_to_reply(
                    ctx,
                    &reply,
                    &format!("Refreshed {} matches for {}\n", count, stat.display_name),
                )
                .await?;
            }
            Ok(None) => {
                add_to_reply(
                    ctx,
                    &reply,
                    &format!(
                        "No dota matches found for {} with PlayerId={}. Removing player from cache.\n",
                        stat.display_name,
                        stat.player_id
                    ),
                )
                .await?;
            }
            Err(e) => {
                add_to_reply(
                    ctx,
                    &reply,
                    &format!("Failed to refresh {} : {}\n", stat.display_name, e),
                )
                .await?;
            }
        }
    }

    command_events_db::insert_event(
        ctx.guild_id,
        command_events_db::EventType::AdminRefresh,
        discord_user_id,
        Utc::now().timestamp(),
    )
    .await?;

    Ok(())
}

async fn check_cooldown(
    server_id: i64,
    event_type: command_events_db::EventType,
    user_id: Option<i64>,
    cooldown_min: u64,
) -> Result<Option<i64>, Error> {
    let cooldown = command_events_db::query_last_event(server_id, event_type, user_id).await?;

    if let Some(cd) = cooldown {
        let now = Utc::now().timestamp();
        let elapsed = now - cd.event_time;
        let cooldown_secs = (cooldown_min * 60) as i64;

        if elapsed < cooldown_secs {
            return Ok(Some(cooldown_secs - elapsed));
        }
    }

    Ok(None)
}

async fn add_to_reply(
    ctx: &CmdCtx<'_>,
    reply: &ReplyHandle<'_>,
    append_text: &str,
) -> Result<(), Error> {
    let message = reply.message().await?;
    let new_content = format!("{}\n{}", message.content, append_text);
    reply
        .edit(
            ctx.discord_ctx,
            poise::CreateReply::default().content(new_content),
        )
        .await
        .ok();
    Ok(())
}

