use chrono::Utc;
use poise::CreateReply;
use tracing::{error, info};

use crate::database::{database_access, player_servers_db};
use crate::discord::discord_helper::{self, CommandCtx};
use crate::leaderboard::duration::Duration;
use crate::leaderboard::leaderboard_stats;
use crate::util::dates;
use crate::{Context, Error};

#[poise::command(slash_command, prefix_command, rename = "dev_leaderboard")]
pub async fn leaderboard(
    ctx: Context<'_>,
    #[description = "The duration for the leaderboard"] duration: Duration,
) -> Result<(), Error> {
    let cmd_ctx = discord_helper::get_command_ctx(ctx).await?;
    leaderboard_command(&cmd_ctx, duration).await?;
    Ok(())
}

pub async fn leaderboard_command(ctx: &CommandCtx<'_>, duration: Duration) -> Result<(), Error> {
    let end_utc = Utc::now();
    let start_utc = duration.start_date(end_utc);
    let txn = database_access::get_transaction().await?;
    let players = player_servers_db::query_server_players(&txn, Some(ctx.guild_id)).await?;
    if players.is_empty() {
        error!(
            guild_id = ctx.guild_id,
            "No players registered on server - cannot generate leaderboard"
        );
        ctx.private_reply(
            "No players are registered for this server, so a leaderboard cannot be generated.",
        )
        .await?;

        return Ok(());
    }

    let duration_label = duration.to_label();
    let reply = &ctx
        .private_reply(format!(
            "Generating Leaderboard for {} [ {} -> {} ]",
            duration_label,
            dates::format_short(start_utc),
            dates::format_short(end_utc)
        ))
        .await?;

    let leaderboard_messages = leaderboard_stats::get_leaderboard_messages(
        &txn,
        players,
        &start_utc,
        &end_utc,
        &duration_label,
    )
    .await?;
    txn.commit().await?;
    if leaderboard_messages.is_empty() {
        let content = format!(
            "No matches found for any players in the duration: {} [ {} -> {} ]",
            duration_label,
            dates::format_short(start_utc),
            dates::format_short(end_utc)
        );
        reply
            .edit(
                ctx.discord_ctx,
                CreateReply::default().content(content).ephemeral(true),
            )
            .await?;
        return Ok(());
    }

    let section_count = leaderboard_messages.len();
    let batches = batch_contents(
        leaderboard_messages,
        ctx.discord_ctx.data().config.max_message_length,
    );

    info!(
        section_count,
        batch_count = batches.len(),
        batch_lengths = ?batches.iter().map(|b| b.len()).collect::<Vec<usize>>(),
        "Batching leaderboard sections into messages"
    );

    let mut replies = vec![];
    for batch in batches {
        let batch_reply = ctx.private_reply(batch).await?;
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
