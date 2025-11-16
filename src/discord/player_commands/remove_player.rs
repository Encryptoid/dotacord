use poise::serenity_prelude::User;

use crate::database::{database_access, player_servers_db};
use crate::discord::discord_helper::{get_command_ctx, CommandCtx};
use crate::{fmt, Context, Error};

#[poise::command(slash_command, prefix_command)]
pub async fn remove_player(
    ctx: Context<'_>,
    #[description = "The Discord user"] discord_user: User,
) -> Result<(), Error> {
    let cmd_ctx = get_command_ctx(ctx).await?;
    remove_player_command(&cmd_ctx, discord_user).await?;
    Ok(())
}

async fn remove_player_command(ctx: &CommandCtx<'_>, discord_user: User) -> Result<(), Error> {
    let txn = database_access::get_transaction().await?;
    let removed = player_servers_db::remove_server_player_by_user_id(
        &txn,
        ctx.guild_id,
        discord_user.id.get() as i64,
    )
    .await?;
    txn.commit().await?;

    let display_name = discord_user.display_name();
    let message = if removed {
        fmt!("Removed player: {display_name} from this server.")
    } else {
        fmt!("Player: {display_name} does not exist on this server.")
    };
    ctx.private_reply(message).await?;
    Ok(())
}
