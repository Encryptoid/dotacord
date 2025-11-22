use poise::serenity_prelude::User;
use tracing::info;

use super::super::discord_helper;
use crate::database::{database_access, player_servers_db};
use crate::discord::discord_helper::{get_command_ctx, CmdCtx, Ephemeral};
use crate::{Context, Error};

#[poise::command(slash_command, prefix_command)]
pub async fn rename_player(
    ctx: Context<'_>,
    #[description = "The Discord user"] discord_user: User,
    #[description = "The new custom name for the player on this server"] new_name: String,
) -> Result<(), Error> {
    let cmd_ctx = get_command_ctx(ctx).await?;
    rename_player_command(&cmd_ctx, discord_user, new_name).await?;
    Ok(())
}

async fn rename_player_command(
    ctx: &CmdCtx<'_>,
    discord_user: User,
    new_name: String,
) -> Result<(), Error> {
    let server_name = discord_helper::guild_name(&ctx.discord_ctx)?;
    let new_name = new_name.trim().to_string();
    if new_name.is_empty() {
        ctx.reply(
            Ephemeral::Private,
            "Player name cannot be empty.".to_owned(),
        )
        .await?;
        return Ok(());
    }

    let display_name = discord_user.display_name();
    info!(
        "Request to Rename Player: {display_name} to {new_name} on Server: {server_name} (ID: {})",
        ctx.guild_id
    );

    let txn = database_access::get_transaction().await?;
    let renamed = player_servers_db::rename_server_player_by_user_id(
        &txn,
        ctx.guild_id,
        discord_user.id.get() as i64,
        &new_name,
    )
    .await?;
    txn.commit().await?;

    let message = if renamed {
        format!("Renamed player: {display_name} to {new_name} on this server.")
    } else {
        format!("Player: {display_name} does not exist on this server.")
    };
    ctx.reply(Ephemeral::Private, message).await?;
    Ok(())
}
