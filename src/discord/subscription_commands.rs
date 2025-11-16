use tracing::info;

use super::discord_helper::{self, CommandCtx};
use crate::database::{database_access, servers_db};
use crate::{Context, Error};

#[poise::command(slash_command, guild_only)]
pub async fn subscribe_channel(
    ctx: Context<'_>,
    #[description = "The channel ID to subscribe"] channel_id: String,
) -> Result<(), Error> {
    let cmd_ctx = discord_helper::get_command_ctx(ctx).await?;
    subscribe_channel_command(&cmd_ctx, channel_id).await?;
    Ok(())
}

async fn subscribe_channel_command(ctx: &CommandCtx<'_>, channel_id: String) -> Result<(), Error> {
    let channel_id_parsed = channel_id.parse::<i64>().map_err(|_| {
        Error::from("Invalid channel ID format. Please provide a valid numeric channel ID.")
    })?;

    let txn = database_access::get_transaction().await?;
    servers_db::update_server_channel(&txn, ctx.guild_id, channel_id_parsed).await?;
    txn.commit().await?;

    info!(
        guild_id = ctx.guild_id,
        channel_id = channel_id_parsed,
        "Subscription channel updated"
    );

    discord_helper::public_reply(
        &ctx.discord_ctx,
        format!("Subscription channel set to <#{}>", channel_id_parsed),
    )
    .await?;

    Ok(())
}

#[poise::command(slash_command, guild_only)]
pub async fn subscribe_week(ctx: Context<'_>) -> Result<(), Error> {
    let cmd_ctx = discord_helper::get_command_ctx(ctx).await?;
    subscribe_week_command(&cmd_ctx).await?;
    Ok(())
}

async fn subscribe_week_command(ctx: &CommandCtx<'_>) -> Result<(), Error> {
    let txn = database_access::get_transaction().await?;
    let server = servers_db::query_server_by_id(&txn, ctx.guild_id)
        .await?
        .ok_or(Error::from("Server not found in database"))?;

    let new_state = server.is_sub_week == 0;
    servers_db::update_server_sub_week(&txn, ctx.guild_id, new_state).await?;
    txn.commit().await?;

    info!(
        guild_id = ctx.guild_id,
        is_sub_week = new_state,
        "Weekly subscription toggled"
    );

    let message = if new_state {
        "Weekly leaderboard subscription `Enabled`"
    } else {
        "Weekly leaderboard subscription `Disabled`"
    };

    discord_helper::public_reply(&ctx.discord_ctx, message.to_string()).await?;

    Ok(())
}

#[poise::command(slash_command, guild_only)]
pub async fn subscribe_month(ctx: Context<'_>) -> Result<(), Error> {
    let cmd_ctx = discord_helper::get_command_ctx(ctx).await?;
    subscribe_month_command(&cmd_ctx).await?;
    Ok(())
}

async fn subscribe_month_command(ctx: &CommandCtx<'_>) -> Result<(), Error> {
    let txn = database_access::get_transaction().await?;
    let server = servers_db::query_server_by_id(&txn, ctx.guild_id)
        .await?
        .ok_or(Error::from("Server not found in database"))?;

    let new_state = server.is_sub_month == 0;
    servers_db::update_server_sub_month(&txn, ctx.guild_id, new_state).await?;
    txn.commit().await?;

    info!(
        guild_id = ctx.guild_id,
        is_sub_month = new_state,
        "Monthly subscription toggled"
    );

    let message = if new_state {
        "Monthly leaderboard subscription **enabled**"
    } else {
        "Monthly leaderboard subscription **disabled**"
    };

    discord_helper::public_reply(&ctx.discord_ctx, message.to_string()).await?;

    Ok(())
}
