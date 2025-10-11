use chrono::Utc;
use poise::CreateReply;
use tracing::info;

use crate::data::{database_access, player_servers_db};
use crate::discord::discord_helper;
use crate::leaderboard::duration::Duration;
use crate::leaderboard::leaderboard_stats;
use crate::util::dates;
use crate::{Context, Error};

#[poise::command(slash_command)]
pub async fn dev_leaderboard(
    ctx: Context<'_>,
    #[description = "The duration for the leaderboard"] duration: Duration,
) -> Result<(), Error> {
    let mut conn = database_access::get_new_connection().await?;
    let guild_id = discord_helper::guild_id(&ctx)?;
    if !discord_helper::validate_command(&ctx, &mut conn, guild_id).await? {
        return Ok(());
    }

    let end_utc = Utc::now();
    let start_utc = duration.start_date(end_utc);
    let players = player_servers_db::query_server_players(&mut conn, Some(guild_id)).await?;
    if players.is_empty() {
        info!(guild_id = ?guild_id, "No players registered on server - cannot generate leaderboard");
        discord_helper::private_reply(
            &ctx,
            "No players are registered for this server, so a leaderboard cannot be generated."
                .to_string(),
        )
        .await?;

        return Ok(());
    }

    let duration_label = duration.to_label();
    let reply = discord_helper::private_reply(
        &ctx,
        format!(
            "Generating Leaderboard for {} [ {} -> {} ]",
            duration_label,
            dates::format_short(start_utc),
            dates::format_short(end_utc)
        ),
    )
    .await?;

    let leaderboard_messages = leaderboard_stats::get_leaderboard_messages(
        &mut conn,
        players,
        &start_utc,
        &end_utc,
        &duration_label,
    )
    .await?;
    if leaderboard_messages.is_empty() {
        let content = format!(
            "No matches found for any players in the duration: {} [ {} -> {} ]",
            duration_label,
            dates::format_short(start_utc),
            dates::format_short(end_utc)
        );
        reply
            .edit(ctx, CreateReply::default().content(content).ephemeral(true))
            .await?;
        return Ok(());
    }

    let section_count = leaderboard_messages.len();
    let batches = batch_contents(leaderboard_messages, ctx.data().config.max_message_length);

    info!(
        section_count = section_count,
        batch_count = batches.len(),
        batch_lengths = ?batches.iter().map(|b| b.len()).collect::<Vec<usize>>(),
        "Batching leaderboard sections into messages"
    );

    let mut replies = vec![];
    for batch in batches {
        let batch_reply = discord_helper::private_reply(&ctx, batch).await?;
        replies.push(batch_reply);
    }

    Ok(())
}

fn batch_contents(contents: Vec<String>, max_length: usize) -> Vec<String> {
    let mut batches = Vec::new();
    let mut current_batch = String::new();
    for content in contents {
        let separator_len = if current_batch.is_empty() { 0 } else { 1 }; // newline
        if current_batch.len() + content.len() + separator_len > max_length {
            batches.push(current_batch);
            current_batch = content;
        } else {
            current_batch.push_str(&content);
        }
    }
    if !current_batch.is_empty() {
        batches.push(current_batch);
    }
    batches
}
