use serenity::all::Channel;
use tracing::info;

use crate::database::servers_db;
use crate::discord::discord_helper::{self, CommandCtx};
use crate::str;
use crate::{Context, Error};

enum SubscriptionType {
    Week,
    Month,
}

#[poise::command(
    slash_command,
    subcommands("subscribe_channel", "subscribe_week", "subscribe_month")
)]
pub async fn subscribe(_: Context<'_>) -> Result<(), Error> {
    unreachable!();
}

#[poise::command(slash_command, rename = "channel")]
pub async fn subscribe_channel(
    ctx: Context<'_>,
    #[description = "The Channel Id to subscribe"] channel_id: Channel,
) -> Result<(), Error> {
    let cmd_ctx = discord_helper::get_command_ctx(ctx).await?;
    let channel_id = channel_id.id().get() as i64;
    subscribe_channel_command(&cmd_ctx, channel_id).await?;
    Ok(())
}

async fn subscribe_channel_command(ctx: &CommandCtx<'_>, channel_id: i64) -> Result<(), Error> {
    // let channel_id_parsed = channel_id.parse::<i64>().map_err(|_| {
    //     Error::from("Invalid Channel Id format. Please provide a valid numeric channel ID.")
    // })?;

    servers_db::update_server_channel(ctx.guild_id, channel_id).await?;

    info!(
        guild_id = ctx.guild_id,
        channel_id, "Subscription channel updated"
    );

    discord_helper::public_reply(
        &ctx.discord_ctx,
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

async fn subscribe_command(
    ctx: &CommandCtx<'_>,
    subscription_type: SubscriptionType,
) -> Result<(), Error> {
    let server = servers_db::query_server_by_id(ctx.guild_id)
        .await?
        .ok_or(Error::from("Server not found in database"))?;

    if server.channel_id.is_none() {
        ctx.private_reply(str!(
            "No subscription channel configured. Set one with `/subscribe_channel <channel_id>`."
        ))
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
    };

    ctx.private_reply(str!(message)).await?;

    Ok(())
}

fn get_state(is_subscribed: bool) -> &'static str {
    if is_subscribed {
        "Subscribed"
    } else {
        "Unsubscribed"
    }
}
