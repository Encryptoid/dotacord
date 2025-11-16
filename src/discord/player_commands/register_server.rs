use tracing::info;

use super::super::discord_helper;
use crate::database::{database_access, servers_db};
use crate::discord::discord_helper::{get_command_ctx, CommandCtx};
use crate::{Context, Error};

#[poise::command(slash_command, prefix_command)]
pub async fn register_server(
    ctx: Context<'_>,
    #[description = "The server id to register"] server_id: Option<i64>,
) -> Result<(), Error> {
    let guild_id = match server_id {
        Some(id) => id,
        None => discord_helper::guild_id(&ctx)?,
    };
    let server_name = discord_helper::guild_name(&ctx)?;
    let cmd_ctx = get_command_ctx(ctx).await?;
    register_server_command(&cmd_ctx, guild_id, server_name).await?;
    Ok(())
}

async fn register_server_command(
    ctx: &CommandCtx<'_>,
    guild_id: i64,
    server_name: String,
) -> Result<(), Error> {
    match servers_db::query_server_by_id(guild_id).await? {
        Some(_) => {
            ctx.private_reply("This server is already registered as a Dotacord server.")
                .await?;
            Ok(())
        }
        None => {
            info!(server_name, guild_id, "Registering new discord server");

            let txn = database_access::get_transaction().await?;
            servers_db::insert_server(&txn, guild_id, server_name, None).await?;
            txn.commit().await?;

            ctx.private_reply("Server has been registered as a Dotacord server.")
                .await?;
            Ok(())
        }
    }
}
