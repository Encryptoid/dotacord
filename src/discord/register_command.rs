use crate::database::{player_servers_db, players_db};
use crate::discord::discord_helper::{get_command_ctx, Ephemeral};
use crate::{Context, Error};

#[poise::command(slash_command, guild_only)]
#[tracing::instrument(level = "trace", skip(ctx))]
pub async fn register(
    ctx: Context<'_>,
    #[description = "Your Dota Player ID (from OpenDota/Dotabuff)"] player_id: i64,
) -> Result<(), Error> {
    let cmd_ctx = get_command_ctx(ctx).await?;

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

    if player_servers.iter().any(|ps| ps.player_id == player_id) {
        cmd_ctx
            .reply(
                Ephemeral::Private,
                format!("Dota player ID {player_id} is already registered on this server."),
            )
            .await?;
        return Ok(());
    }

    let discord_user = ctx.author();
    let discord_id = discord_user.id.get() as i64;
    let discord_name = discord_user.name.to_string();

    players_db::insert_player_and_server(
        cmd_ctx.guild_id,
        player_id,
        None,
        Some(discord_id),
        discord_name.clone(),
    )
    .await?;

    cmd_ctx
        .reply(
            Ephemeral::Private,
            format!("You have been registered with Dota player ID {player_id}."),
        )
        .await?;

    Ok(())
}
