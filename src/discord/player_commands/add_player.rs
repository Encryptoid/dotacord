use crate::database::{player_servers_db, players_db};
use crate::discord::discord_helper::{self, get_command_ctx, CmdCtx, Ephemeral};
use crate::{Context, Error};

#[poise::command(slash_command, prefix_command, guild_only, rename = "add")]
pub async fn add_player_command(
    ctx: Context<'_>,
    #[description = "Discord User to associate"] discord_user: serenity::model::user::User,
    #[description = "Dota Player Id(taken from OpenDota/Dotabuff)"] player_id: i64,
    #[description = "Name for the player to add to this server"] nickname: Option<String>,
) -> Result<(), Error> {
    let cmd_ctx = get_command_ctx(ctx).await?;
    if !discord_helper::ensure_admin(&cmd_ctx).await? {
        return Ok(());
    }
    add_player(&cmd_ctx, discord_user, player_id, nickname).await?;
    Ok(())
}

async fn add_player(
    ctx: &CmdCtx<'_>,
    discord_user: serenity::model::user::User,
    player_id: i64,
    nickname: Option<String>,
) -> Result<(), Error> {
    let player_servers = player_servers_db::query_server_players(ctx.guild_id).await?;

    if player_servers.len() >= ctx.app_cfg.max_players_per_server {
        ctx.reply(
            Ephemeral::Private,
            format!(
                "Maximum number of players ({}) reached for this server.",
                ctx.app_cfg.max_players_per_server
            ),
        )
        .await?;
        return Ok(());
    }

    let discord_id = discord_user.id.get() as i64;
    let discord_name = discord_user.name.to_string();
    let display_name = nickname
        .as_ref()
        .map(|s| s.clone())
        .unwrap_or_else(|| discord_name.clone());

    if player_servers.iter().any(|ps| ps.player_id == player_id) {
        ctx.reply(
            Ephemeral::Private,
            format!("Dota player {display_name} ({player_id}) is already on this server"),
        )
        .await?;
        return Ok(());
    }

    players_db::insert_player_and_server(
        ctx.guild_id,
        player_id,
        nickname,
        Some(discord_id),
        discord_name,
    )
    .await?;
    ctx.reply(
        Ephemeral::Private,
        format!("Player {display_name} ({player_id}) has been added to this server."),
    )
    .await?;
    Ok(())
}
