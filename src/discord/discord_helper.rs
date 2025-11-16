use poise::serenity_prelude::{Channel, CreateMessage, Http, MessageFlags};
use poise::{CreateReply, ReplyHandle};
use tokio::time::Duration;
use tracing::{debug, info, warn};

use crate::database::servers_db;
use crate::leaderboard::emoji::Emoji;
use crate::{Context, Error};

const GUILD_LOOKUP_ERROR: &str = "Could not get guild";
pub struct CommandCtx<'a> {
    pub app_cfg: crate::config::AppConfig,
    pub guild_id: i64,
    pub discord_ctx: Context<'a>,
}

pub(crate) async fn get_command_ctx<'a>(ctx: Context<'a>) -> Result<CommandCtx<'a>, Error> {
    let data = ctx.data();
    let guild_id = guild_id(&ctx)?;
    if !validate_command(&ctx, guild_id).await? {
        return Err(Error::from("Command validation failed"));
    }
    Ok(CommandCtx {
        app_cfg: data.config.clone(),
        guild_id,
        discord_ctx: ctx,
    })
}

pub(crate) fn guild_id(ctx: &Context<'_>) -> Result<i64, Error> {
    Ok(ctx.guild().ok_or_else(|| GUILD_LOOKUP_ERROR)?.id.get() as i64)
}

pub(crate) fn guild_name(ctx: &Context<'_>) -> Result<String, Error> {
    ctx.guild()
        .map(|guild| guild.name.to_string())
        .ok_or_else(|| GUILD_LOOKUP_ERROR.into())
}

pub(crate) fn channel_id(ctx: &Context<'_>) -> Result<i64, Error> {
    Ok(ctx.channel_id().get() as i64)
}

pub(crate) async fn validate_command(ctx: &Context<'_>, guild_id: i64) -> Result<bool, Error> {
    let author = ctx
        .author_member()
        .await
        .ok_or(Error::from("Could not get author"))?;
    let user_id = author.user.id.get().to_string();
    let name = author.user.display_name();

    info!(
        command_name = ctx.invoked_command_name(),
        command_text = ctx.invocation_string(),
        user_id,
        name,
        guild_id,
        "Command Invoked"
    );

    if !validate_server(guild_id).await? {
        warn!(
            guild_id = guild_id,
            "Command invoked in unregistered server"
        );
        return Ok(false);
    }

    Ok(true)
}

async fn validate_server(guild_id: i64) -> Result<bool, Error> {
    match servers_db::query_server_by_id(guild_id).await? {
        Some(_) => Ok(true),
        _ => {
            warn!(
                guild_id = guild_id,
                "Attempted command in unregistered server"
            );
            Ok(false)
        }
    }
}

pub(crate) async fn public_reply<'a>(
    ctx: &Context<'a>,
    content: String,
) -> Result<ReplyHandle<'a>, Error> {
    info!(content = content, "Sending public reply");
    Ok(ctx
        .send(
            CreateReply::new()
                .content(content)
                .ephemeral(false)
                .flags(MessageFlags::SUPPRESS_EMBEDS),
        )
        .await?)
}

impl<'a> CommandCtx<'a> {
    pub(crate) async fn private_reply<S>(&self, content: S) -> Result<ReplyHandle<'_>, Error>
    where
        S: Into<String>,
    {
        let content = content.into();
        debug!(content, "Sending reply to user");
        Ok(self
            .discord_ctx
            .send(
                CreateReply::new()
                    .content(content)
                    .ephemeral(true)
                    .flags(MessageFlags::SUPPRESS_EMBEDS | MessageFlags::EPHEMERAL),
            )
            .await?)
    }
}

pub async fn send_message(channel: &Channel, http: &Http, content: &str) -> Result<(), Error> {
    info!(content_length = content.len(), "Sending chat message");
    channel
        .id()
        .send_message(
            http,
            CreateMessage::default()
                .content(content)
                .flags(MessageFlags::SUPPRESS_EMBEDS | MessageFlags::EPHEMERAL),
        )
        .await?;

    Ok(())
}

pub(crate) async fn reply_countdown(
    ctx: &Context<'_>,
    initial_content: &str,
    countdown_text: &str,
    final_content: String,
) -> Result<(), Error> {
    let config = &ctx.data().config;
    let duration_ms = config.countdown_duration_ms;
    let offset_ms = config.countdown_offset_ms;
    let count = (duration_ms / 1000) as i32;
    let sleep_ms = (duration_ms / count as u64) + offset_ms;

    let reply = public_reply(&ctx, initial_content.to_string()).await?;
    for i in (1..=count).rev() {
        tokio::time::sleep(Duration::from_millis(sleep_ms)).await;
        let countdown_content = format!(
            "{}{} {} **{}...** {}",
            initial_content,
            countdown_text,
            Emoji::BIG_SLAP,
            i,
            Emoji::BIG_SLAP
        );
        reply
            .edit(*ctx, CreateReply::default().content(countdown_content))
            .await
            .ok();
    }

    reply
        .edit(
            *ctx,
            CreateReply::default().content(format!("{}{}", initial_content, final_content)),
        )
        .await
        .ok();
    Ok(())
}
