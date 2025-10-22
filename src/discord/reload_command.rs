use poise::ReplyHandle;

use crate::api::reload;
use crate::database::{database_access, player_servers_db};
use crate::discord::discord_helper;
use crate::{Context, Error};

#[poise::command(slash_command, prefix_command)]
#[tracing::instrument(name = "RELOAD_MATCHES", level = "trace", skip(ctx))]
pub async fn reload_matches(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = discord_helper::guild_id(&ctx)?;
    let mut conn = database_access::get_new_connection().await?;
    if !discord_helper::validate_command(&ctx, &mut conn, guild_id).await? {
        return Ok(());
    }

    let players = player_servers_db::query_server_players(&mut conn, Some(guild_id)).await?;
    if players.is_empty() {
        ctx.say("No players found for this server").await?;
        return Ok(());
    }

    // Don't say ephemeral so that other people can see that a reload has happened
    let reply = ctx
        .say(format!(
            "Reloading player matches for {} players. Message will be edited with progress updates.\n",
            players.len()
        ))
        .await?;
    for player in &players {
        let stat = reload::reload_player(&mut conn, player).await;
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

    Ok(())
}

async fn add_to_reply(
    ctx: Context<'_>,
    reply: &ReplyHandle<'_>,
    append_text: &str,
) -> Result<(), Error> {
    let message = reply.message().await?;
    let new_content = format!("{}\n{}", message.content, append_text);
    reply
        .edit(ctx, poise::CreateReply::default().content(new_content))
        .await
        .ok();
    Ok(())
}
