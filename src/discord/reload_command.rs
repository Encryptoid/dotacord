use poise::ReplyHandle;
use sqlx::Connection;
use tracing::info;

use crate::data::{
    database_access, open_dota_api, player_matches_db, player_servers_db, servers_db,
};
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
        let match_count = reload_player(&mut conn, player).await?;
        if let Some(count) = match_count {
            add_to_reply(
                ctx,
                &reply,
                &format!("Reloaded {} matches for {}\n", count, player.display_name()),
            )
            .await?;
        } else {
            add_to_reply(
                ctx,
                &reply,
                &format!(
                    "No dota matches found for {} with PlayerId={}. Removing player from cache.\n",
                    player.display_name(),
                    player.player_id
                ),
            )
            .await?;
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

async fn reload_player(
    conn: &mut sqlx::SqliteConnection,
    player: &player_servers_db::PlayerServer,
) -> Result<Option<usize>, Error> {
    info!(player_id = player.player_id, "Reloading matches for player");
    let db_matches = player_matches_db::query_matches_by_player_id(conn, player.player_id).await?;
    let api_matches = open_dota_api::get_player_matches(player.player_id).await?;

    info!(
        player_id = player.player_id,
        db_matches = db_matches.len(),
        api_matches = api_matches.len(),
        "Fetched matches from OpenDota API"
    );

    if api_matches.is_empty() {
        info!(
            player_id = player.player_id,
            server_id = player.server_id,
            "No matches found from OpenDota API. Removing player from server."
        );
        player_servers_db::remove_server_player_by_user_id(conn, player.server_id, player.user_id)
            .await?;
        return Ok(None);
    }

    let (header_count, player_match_count) =
        import_player_matches(conn, player.player_id, &db_matches, &api_matches).await?;

    info!(
        player_id = player.player_id,
        headers_inserted = header_count,
        player_matches_inserted = player_match_count,
        "Finished reloading matches for player"
    );

    Ok(Some(player_match_count))
}

#[tracing::instrument(level = "trace", skip(conn, db_matches, api_matches))]
async fn import_player_matches(
    conn: &mut sqlx::SqliteConnection,
    player_id: i64,
    db_matches: &[player_matches_db::PlayerMatch],
    api_matches: &[open_dota_api::ApiPlayerMatch],
) -> Result<(usize, usize), Error> {
    let header_count = 0;
    let mut player_match_count = 0;
    let mut tx = conn.begin().await?;

    for api_match in api_matches {
        if db_matches.iter().any(|m| m.match_id == api_match.match_id) {
            continue;
        }

        let Some(player_match) = player_matches_db::map_to_player_match(api_match, player_id)?
        else {
            continue;
        };

        player_matches_db::insert_player_match(tx.as_mut(), &player_match).await?;
        player_match_count += 1;
    }

    tx.commit().await?;

    Ok((header_count, player_match_count))
}
