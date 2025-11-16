use poise::ReplyHandle;

use crate::api::reload;
use crate::database::{database_access, player_servers_db};
use crate::discord::discord_helper::{self, CommandCtx};
use crate::{Context, Error};

#[poise::command(slash_command, prefix_command)]
#[tracing::instrument(name = "RELOAD_MATCHES", level = "trace", skip(ctx))]
pub async fn reload_matches(ctx: Context<'_>) -> Result<(), Error> {
    let cmd_ctx = discord_helper::get_command_ctx(ctx).await?;
    reload_matches_command(&cmd_ctx).await?;
    Ok(())
}

async fn reload_matches_command(ctx: &CommandCtx<'_>) -> Result<(), Error> {
    let txn = database_access::get_transaction().await?;
    let players = player_servers_db::query_server_players(&txn, Some(ctx.guild_id)).await?;
    if players.is_empty() {
        ctx.discord_ctx.say("No players found for this server").await?;
        return Ok(());
    }

    let reply = ctx.discord_ctx
        .say(format!(
            "Reloading player matches for {} players. Message will be edited with progress updates.\n",
            players.len()
        ))
        .await?;
    for player in &players {
        let stat = reload::reload_player(&txn, player).await;
        match stat.result {
            Ok(Some(count)) => {
                add_to_reply(
                    ctx,
                    &reply,
                    &format!("Reloaded {} matches for {}\n", count, stat.display_name),
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
                    &format!("Failed to reload {} : {}\n", stat.display_name, e),
                )
                .await?;
            }
        }
    }
    txn.commit().await?;

    Ok(())
}

async fn add_to_reply(
    ctx: &CommandCtx<'_>,
    reply: &ReplyHandle<'_>,
    append_text: &str,
) -> Result<(), Error> {
    let message = reply.message().await?;
    let new_content = format!("{}\n{}", message.content, append_text);
    reply
        .edit(ctx.discord_ctx, poise::CreateReply::default().content(new_content))
        .await
        .ok();
    Ok(())
}
