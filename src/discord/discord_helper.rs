use std::fmt::Display;

use clap::command;
use poise::serenity_prelude::MessageFlags;
use poise::{CreateReply, ReplyHandle};
use serenity::all::Permissions;
use tokio::time::Duration;
use tracing::{debug, info, warn};

use crate::database::servers_db;
use crate::leaderboard::emoji::Emoji;
use crate::{seq_span, Context, Error};

const GUILD_LOOKUP_ERROR: &str = "Could not get guild";
pub struct CmdCtx<'a> {
    pub app_cfg: crate::config::AppConfig,
    pub guild_id: i64,
    pub discord_ctx: Context<'a>,
}

pub(crate) async fn get_command_ctx<'a>(ctx: Context<'a>) -> Result<CmdCtx<'a>, Error> {
    let data = ctx.data();
    let guild_id = guild_id(&ctx)?;
    if let Some(err) = validate_command(&ctx, guild_id).await? {
        return Err(Error::from(err));
    }
    Ok(CmdCtx {
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

pub(crate) fn format_bool(is_subscribed: i32) -> &'static str {
    if is_subscribed != 0 {
        Emoji::GOODJOB
    } else {
        Emoji::SLEEPING
    }
}

pub(crate) async fn validate_command(
    ctx: &Context<'_>,
    guild_id: i64,
) -> Result<Option<String>, Error> {
    let author = ctx
        .author_member()
        .await
        .ok_or(Error::from("Could not get author"))?;
    let user_id = author.user.id.get().to_string();
    let name = author.user.display_name();

    seq_span!(
        "command",
        command_name = ctx.invoked_command_name(),
        command_text = ctx.invocation_string(),
        user_id,
        name,
        guild_id
    );
    info!("Command Invoked");

    if !validate_server(guild_id).await? {
        warn!(
            guild_id = guild_id,
            "Command invoked in unregistered server"
        );
        return Ok(Some("Server is not registered as a dotacord server. If you are an admin, you can register with /register_server".to_string()));
    }

    Ok(None)
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

#[derive(Debug)]
pub enum Ephemeral {
    Public,
    Private,
}

impl Display for Ephemeral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ephemeral::Public => write!(f, "Public"),
            Ephemeral::Private => write!(f, "Private"),
        }
    }
}

impl<'a> CmdCtx<'a> {
    pub(crate) async fn reply<S>(
        &self,
        ephemeral: Ephemeral,
        content: S,
    ) -> Result<ReplyHandle<'_>, Error>
    where
        S: Into<String>,
    {
        let content = content.into();
        debug!(content, ephemeral = %ephemeral, "Sending Reply");
        Ok(self
            .discord_ctx
            .send(
                CreateReply::new()
                    .content(content)
                    .ephemeral(matches!(ephemeral, Ephemeral::Private))
                    .flags(MessageFlags::SUPPRESS_EMBEDS | MessageFlags::EPHEMERAL),
            )
            .await?)
    }

    pub(crate) async fn edit(&self, reply: &ReplyHandle<'_>, content: String) -> Result<(), Error> {
        debug!(content, "Editing Reply");
        reply
            .edit(self.discord_ctx, CreateReply::default().content(content))
            .await?;
        Ok(())
    }
}

// pub async fn send_message(channel: &Channel, http: &Http, content: &str) -> Result<(), Error> {
//     info!(content_length = content.len(), "Sending chat message");
//     channel
//         .id()
//         .send_message(
//             http,
//             CreateMessage::default()
//                 .content(content)
//                 .flags(MessageFlags::SUPPRESS_EMBEDS | MessageFlags::EPHEMERAL),
//         )
//         .await?;

//     Ok(())
// }

pub(crate) async fn reply_countdown(
    ctx: &CmdCtx<'_>,
    initial_content: &str,
    countdown_text: &str,
    final_content: String,
) -> Result<(), Error> {
    let config = &ctx.app_cfg;
    let duration_ms = config.countdown_duration_ms;
    let offset_ms = config.countdown_offset_ms;
    let count = (duration_ms / 1000) as i32;
    let sleep_ms = (duration_ms / count as u64) + offset_ms;

    let reply = ctx
        .reply(Ephemeral::Public, initial_content.to_string())
        .await?;
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
            .edit(
                ctx.discord_ctx,
                CreateReply::default().content(countdown_content),
            )
            .await
            .ok();
    }

    reply
        .edit(
            ctx.discord_ctx,
            CreateReply::default().content(format!("{}{}", initial_content, final_content)),
        )
        .await
        .ok();
    Ok(())
}

pub(super) async fn ensure_admin(ctx: &CmdCtx<'_>) -> Result<bool, Error> {
    let member = ctx
        .discord_ctx
        .author_member()
        .await
        .ok_or_else(|| Error::from("Failed to load command author"))?;

    let permissions = member.permissions.unwrap_or_else(Permissions::empty);
    if permissions.administrator() {
        return Ok(true);
    }

    ctx.reply(
        Ephemeral::Private,
        "Only server administrators can manage tracked players.",
    )
    .await?;

    Ok(false)
}
