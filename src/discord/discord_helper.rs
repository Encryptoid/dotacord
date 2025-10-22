use poise::serenity_prelude::{
    Channel, CreateMessage, Http,
    MessageFlags,
};
use poise::{CreateReply, ReplyHandle};
use tracing::{debug, info, warn};

use crate::database::servers_db;
use crate::{Context, Error};

const GUILD_LOOKUP_ERROR: &str = "Could not get guild";

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

pub(crate) async fn validate_command(
    ctx: &Context<'_>,
    conn: &mut sqlx::SqliteConnection,
    guild_id: i64,
) -> Result<bool, Error> {
    let author = ctx
        .author_member()
        .await
        .ok_or(Error::from("Could not get author"))?;
    let user_id = author.user.id.get().to_string();
    let name = author.user.display_name();

    info!(
        command_name = ctx.invoked_command_name(),
        command_text = ctx.invocation_string(),
        user_id = user_id,
        name = name,
        guild_id = guild_id,
        "Command Invoked"
    );

    if !validate_server(conn, guild_id).await? {
        warn!(
            guild_id = guild_id,
            "Command invoked in unregistered server"
        );
        return Ok(false);
    }

    Ok(true)
}

async fn validate_server(conn: &mut sqlx::SqliteConnection, guild_id: i64) -> Result<bool, Error> {
    match servers_db::query_server_by_id(conn, guild_id).await? {
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

pub(crate) async fn private_reply<'a>(
    ctx: &'a Context<'a>,
    content: String,
) -> Result<ReplyHandle<'a>, Error> {
    debug!(content = content, "Sending reply to user");
    Ok(ctx
        .send(
            CreateReply::new()
                .content(content)
                .ephemeral(true)
                .flags(MessageFlags::SUPPRESS_EMBEDS | MessageFlags::EPHEMERAL),
        )
        .await?)
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
