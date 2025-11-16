use crate::database::{player_servers_db, players_db};
use crate::discord::discord_helper::{get_command_ctx, CommandCtx};
use crate::{fmt, Context, Error};

#[poise::command(slash_command, prefix_command)]
pub async fn add_player(
    ctx: Context<'_>,
    #[description = "Name for the player to add to this server"] name: String,
    #[description = "Dota Player Id(taken from OpenDota/Dotabuff)"] player_id: i64,
) -> Result<(), Error> {
    let cmd_ctx = get_command_ctx(ctx).await?;
    add_player_command(&cmd_ctx, name, player_id).await?;
    Ok(())
}

async fn add_player_command(
    ctx: &CommandCtx<'_>,
    name: String,
    player_id: i64,
) -> Result<(), Error> {
    let player_servers = player_servers_db::query_server_players(ctx.guild_id).await?;

    if player_servers.len() >= ctx.app_cfg.max_players_per_server {
        ctx.private_reply(fmt!(
            "Maximum number of players ({}) reached for this server.",
            ctx.app_cfg.max_players_per_server
        ))
        .await?;
        return Ok(());
    }

    if player_servers.iter().any(|ps| ps.player_id == player_id) {
        ctx.private_reply(fmt!(
            "Dota player {name} ({player_id}) is already on this server"
        ))
        .await?;
        return Ok(());
    }

    players_db::insert_player_and_server(ctx.guild_id, player_id, &name).await?;
    ctx.private_reply(fmt!(
        "Player {name} ({player_id}) has been added to this server."
    ))
    .await?;
    Ok(())
}
