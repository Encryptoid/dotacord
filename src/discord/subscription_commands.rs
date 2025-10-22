use tracing::info;

use super::discord_helper;
use crate::database::{database_access, servers_db};
use crate::{Context, Error};

#[poise::command(slash_command, guild_only)]
pub async fn subscribe_channel(
    ctx: Context<'_>,
    #[description = "The channel ID to subscribe"] channel_id: String,
) -> Result<(), Error> {
    let mut conn = database_access::get_new_connection().await?;
    let guild_id = discord_helper::guild_id(&ctx)?;
    if !discord_helper::validate_command(&ctx, &mut conn, guild_id).await? {
        return Ok(());
    }

    let channel_id_parsed = channel_id.parse::<i64>().map_err(|_| {
        Error::from("Invalid channel ID format. Please provide a valid numeric channel ID.")
    })?;

    servers_db::update_server_channel(&mut conn, guild_id, channel_id_parsed).await?;

    info!(
        guild_id = guild_id,
        channel_id = channel_id_parsed,
        "Subscription channel updated"
    );

    discord_helper::public_reply(
        &ctx,
        format!("Subscription channel set to <#{}>", channel_id_parsed),
    )
    .await?;

    Ok(())
}

#[poise::command(slash_command, guild_only)]
pub async fn subscribe_week(ctx: Context<'_>) -> Result<(), Error> {
    let mut conn = database_access::get_new_connection().await?;
    let guild_id = discord_helper::guild_id(&ctx)?;
    if !discord_helper::validate_command(&ctx, &mut conn, guild_id).await? {
        return Ok(());
    }

    let server = servers_db::query_server_by_id(&mut conn, guild_id)
        .await?
        .ok_or(Error::from("Server not found in database"))?;

    let new_state = server.is_sub_week == 0;
    servers_db::update_server_sub_week(&mut conn, guild_id, new_state).await?;

    info!(
        guild_id = guild_id,
        is_sub_week = new_state,
        "Weekly subscription toggled"
    );

    let message = if new_state {
        "Weekly leaderboard subscription **enabled**"
    } else {
        "Weekly leaderboard subscription **disabled**"
    };

    discord_helper::public_reply(&ctx, message.to_string()).await?;

    Ok(())
}

#[poise::command(slash_command, guild_only)]
pub async fn subscribe_month(ctx: Context<'_>) -> Result<(), Error> {
    let mut conn = database_access::get_new_connection().await?;
    let guild_id = discord_helper::guild_id(&ctx)?;
    if !discord_helper::validate_command(&ctx, &mut conn, guild_id).await? {
        return Ok(());
    }

    let server = servers_db::query_server_by_id(&mut conn, guild_id)
        .await?
        .ok_or(Error::from("Server not found in database"))?;

    let new_state = server.is_sub_month == 0;
    servers_db::update_server_sub_month(&mut conn, guild_id, new_state).await?;

    info!(
        guild_id = guild_id,
        is_sub_month = new_state,
        "Monthly subscription toggled"
    );

    let message = if new_state {
        "Monthly leaderboard subscription **enabled**"
    } else {
        "Monthly leaderboard subscription **disabled**"
    };

    discord_helper::public_reply(&ctx, message.to_string()).await?;

    Ok(())
}
