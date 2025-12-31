use serenity::all::Channel;
use tracing::info;

use crate::database::servers_db;
use crate::discord::discord_helper::{self, CmdCtx, Ephemeral};
use crate::leaderboard::emoji::Emoji;
use crate::str;
use crate::{Context, Error};

enum SubscriptionType {
    Week,
    Month,
    Reload,
}

fn format_subscription(is_subscribed: bool, description: &str) -> String {
    if is_subscribed {
        format!("Subscribed to `{}` {}", description, Emoji::GOODJOB)
    } else {
        format!("Unsubscribed from `{}` {}", description, Emoji::SLEEPING)
    }
}

#[poise::command(
    slash_command,
    prefix_command,
    subcommands(
        "subscribe_info",
        "subscribe_channel",
        "subscribe_week",
        "subscribe_month",
        "subscribe_reload"
    )
)]
pub async fn subscribe(_: Context<'_>) -> Result<(), Error> {
    todo!();
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

#[poise::command(slash_command, rename = "info")]
pub async fn subscribe_info(ctx: Context<'_>) -> Result<(), Error> {
    let cmd_ctx = discord_helper::get_command_ctx(ctx).await?;
    subscribe_info_command(&cmd_ctx).await?;
    Ok(())
}

async fn subscribe_info_command(ctx: &CmdCtx<'_>) -> Result<(), Error> {
    let server = get_server(ctx).await?;

    let mut channel_display = match server.channel_id {
        Some(id) => format!("This server's updates will be posted in: <#{}>", id),
        None => format!("No subscription channel configured."),
    };

    channel_display.push_str("\nSet the channel with `/subscribe_channel <channel_id>`.");

    let response = format!(
        r#"
{} - (/subscribe channel)
{} - Weekly Updates (/subscribe week)
{} - Monthly Updates (/subscribe month)
{} - Auto Reload Matches (/subscribe reload)"#,
        channel_display,
        discord_helper::format_bool(server.is_sub_week),
        discord_helper::format_bool(server.is_sub_month),
        discord_helper::format_bool(server.is_sub_reload)
    );

    ctx.reply(Ephemeral::Private, str!(response)).await?;

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
    let server = get_server(ctx).await?;

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
            let new_state = 1 - server.is_sub_week;
            servers_db::update_server_sub_week(ctx.guild_id, new_state).await?;
            format_subscription(new_state == 1, "Weekly Leaderboard Updates")
        }
        SubscriptionType::Month => {
            let new_state = 1 - server.is_sub_month;
            servers_db::update_server_sub_month(ctx.guild_id, new_state).await?;
            format_subscription(new_state == 1, "Monthly Leaderboard Updates")
        }
        SubscriptionType::Reload => {
            let new_state = 1 - server.is_sub_reload;
            servers_db::update_server_sub_reload(ctx.guild_id, new_state).await?;
            format_subscription(new_state == 1, "Automatic Match Reloads")
        }
    };

    ctx.reply(Ephemeral::Private, str!(message)).await?;

    Ok(())
}

async fn get_server(ctx: &CmdCtx<'_>) -> Result<servers_db::DiscordServer, Error> {
    servers_db::query_server_by_id(ctx.guild_id)
        .await?
        .ok_or_else(|| Error::from("Server not found in database"))
}

