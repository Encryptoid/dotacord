use serenity::all::Channel;
use tracing::info;

use crate::database::servers_db;
use crate::discord::discord_helper::{self, CmdCtx, Ephemeral};
use crate::str;
use crate::{Context, Error};

enum SubscriptionType {
    Week,
    Month,
    Reload,
}

#[poise::command(
    slash_command,
    subcommands(
        "subscribe_channel",
        "subscribe_week",
        "subscribe_month",
        "subscribe_reload"
    )
)]
pub async fn subscribe(_: Context<'_>) -> Result<(), Error> {
    unreachable!();
}

#[poise::command(slash_command, rename = "channel")]
pub async fn subscribe_channel(
    ctx: Context<'_>,
    #[description = "The Channel to publish leaderboard subscriptions to"] channel: Channel,
) -> Result<(), Error> {
    let cmd_ctx = discord_helper::get_command_ctx(ctx).await?;
    subscribe_channel_command(&cmd_ctx, channel).await?;
    Ok(())
}

async fn subscribe_channel_command(ctx: &CmdCtx<'_>, channel: Channel) -> Result<(), Error> {
    let channel_id = channel.id().get() as i64;
    let category = channel
        .guild()
        .ok_or(Error::from("Could not get channel category information"))?;

    if !category.is_text_based() {
        ctx.reply(
            Ephemeral::Private,
            str!("The provided channel is not a text-based channel."),
        )
        .await?;
        return Ok(());
    }

    servers_db::update_server_channel(ctx.guild_id, channel_id).await?;

    info!(
        guild_id = ctx.guild_id,
        channel_id, "Subscription channel updated"
    );

    ctx.reply(
        Ephemeral::Private,
        format!("Subscription channel set to <#{}>", channel_id),
    )
    .await?;

    Ok(())
}

#[poise::command(slash_command, rename = "week")]
pub async fn subscribe_week(ctx: Context<'_>) -> Result<(), Error> {
    let cmd_ctx = discord_helper::get_command_ctx(ctx).await?;
    subscribe_command(&cmd_ctx, SubscriptionType::Week).await?;
    Ok(())
}

#[poise::command(slash_command, rename = "month")]
pub async fn subscribe_month(ctx: Context<'_>) -> Result<(), Error> {
    let cmd_ctx = discord_helper::get_command_ctx(ctx).await?;
    subscribe_command(&cmd_ctx, SubscriptionType::Month).await?;
    Ok(())
}

#[poise::command(slash_command, rename = "reload")]
pub async fn subscribe_reload(ctx: Context<'_>) -> Result<(), Error> {
    let cmd_ctx = discord_helper::get_command_ctx(ctx).await?;
    subscribe_command(&cmd_ctx, SubscriptionType::Reload).await?;
    Ok(())
}

async fn subscribe_command(
    ctx: &CmdCtx<'_>,
    subscription_type: SubscriptionType,
) -> Result<(), Error> {
    let server = servers_db::query_server_by_id(ctx.guild_id)
        .await?
        .ok_or(Error::from("Server not found in database"))?;

    if server.channel_id.is_none() {
        ctx.reply(
            Ephemeral::Private,
            str!("No subscription channel configured. Set one with `/subscribe_channel <channel_id>`."),
        )
        .await?;

        return Ok(());
    }

    let message = match subscription_type {
        SubscriptionType::Week => {
            let new_state = server.is_sub_week == 0;
            servers_db::update_server_sub_week(ctx.guild_id, new_state).await?;
            format!("{} to Weekly Leaderboard Updates", get_state(new_state))
        }
        SubscriptionType::Month => {
            let new_state = server.is_sub_month == 0;
            servers_db::update_server_sub_month(ctx.guild_id, new_state).await?;
            format!("{} to Monthly Leaderboard Updates", get_state(new_state))
        }
        SubscriptionType::Reload => {
            let new_state = server.is_sub_reload == 0;
            servers_db::update_server_sub_reload(ctx.guild_id, new_state).await?;
            format!("{} to Automatic Match Reloads", get_state(new_state))
        }
    };

    ctx.reply(Ephemeral::Private, str!(message)).await?;

    Ok(())
}

fn get_state(is_subscribed: bool) -> &'static str {
    if is_subscribed {
        "Subscribed"
    } else {
        "Unsubscribed"
    }
}
