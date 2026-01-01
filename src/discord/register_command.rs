use crate::database::{player_servers_db, players_db};
use crate::discord::discord_helper::{get_command_ctx, Ephemeral};
use crate::{Context, Error};

/// Register your Dota Player ID to the server leaderboard
#[poise::command(slash_command, guild_only)]
#[tracing::instrument(level = "trace", skip(ctx))]
pub async fn register_to_leaderboard(
    ctx: Context<'_>,
    #[description = "Your Dota Player ID (from OpenDota/Dotabuff)"] dota_player_id: i64,
) -> Result<(), Error> {
    let cmd_ctx = get_command_ctx(ctx).await?;

    let discord_user = ctx.author();
    let discord_id = discord_user.id.get() as i64;

    let player_servers = player_servers_db::query_server_players(cmd_ctx.guild_id).await?;

    if player_servers.len() >= cmd_ctx.app_cfg.max_players_per_server {
        cmd_ctx
            .reply(
                Ephemeral::Private,
                format!(
                    "Maximum number of players ({}) reached for this server.",
                    cmd_ctx.app_cfg.max_players_per_server
                ),
            )
            .await?;
        return Ok(());
    }

    if player_servers.iter().any(|ps| ps.player_id == dota_player_id) {
        cmd_ctx
            .reply(
                Ephemeral::Private,
                format!("Dota player ID {dota_player_id} is already registered on this server."),
            )
            .await?;
        return Ok(());
    }

    if player_servers.iter().any(|ps| ps.discord_user_id == Some(discord_id)) {
        cmd_ctx
            .reply(
                Ephemeral::Private,
                "You are already registered on this server. If there is a mistake, contact an admin.",
            )
            .await?;
        return Ok(());
    }

    let discord_name = discord_user
        .global_name
        .as_ref()
        .map(|n| n.to_string())
        .unwrap_or_else(|| discord_user.name.to_string());

    players_db::insert_player_and_server(
        cmd_ctx.guild_id,
        dota_player_id,
        None,
        Some(discord_id),
        discord_name.clone(),
    )
        .await?;

    cmd_ctx
        .reply(
            Ephemeral::Private,
            format!("You have been registered with Dota player ID {dota_player_id}."),
        )
        .await?;

    Ok(())
}

