use poise::serenity_prelude::User;
use tracing::{info, warn};

use super::discord_helper;
use crate::data::player_servers_db::PlayerServer;
use crate::data::{database_access, player_servers_db, players_db, servers_db};
use crate::markdown::stats_formatter::{Column, TableBuilder};
use crate::{Context, Error};

// default_member_permissions = Permissions::ADMINISTRATOR;
#[poise::command(slash_command, guild_only)]
pub async fn list_players(ctx: Context<'_>) -> Result<(), Error> {
    let mut conn = database_access::get_new_connection().await?;
    let guild_id = discord_helper::guild_id(&ctx)?;
    if !discord_helper::validate_command(&ctx, &mut conn, guild_id).await? {
        return Ok(());
    }

    let player_servers = player_servers_db::query_server_players(&mut conn, Some(guild_id)).await?;
    let member = ctx
        .author_member()
        .await
        .ok_or(Error::from("Failed to get author member"))?;
    info!(
        permissions = member.permissions.unwrap().to_string(),
        "Player Command"
    );

    let content = if player_servers.len() > 0 {
        format_list_players(&player_servers)
    } else {
        "No players are registered for this server, so a leaderboard cannot be generated."
            .to_string()
    };
    discord_helper::private_reply(&ctx, content).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn add_player(
    ctx: Context<'_>,
    #[description = "The Discord user"] discord_user: User,
    #[description = "Dota Player Id(taken from OpenDota/Dotabuff)"] player_id: i64,
    #[description = "A custom name for the player on this server (optional)"] name: Option<String>,
) -> Result<(), Error> {
    let mut conn = database_access::get_new_connection().await?;
    let guild_id = discord_helper::guild_id(&ctx)?;
    let server_name = discord_helper::guild_name(&ctx)?;
    if !discord_helper::validate_command(&ctx, &mut conn, guild_id).await? {
        return Ok(());
    }

    let player_servers = player_servers_db::query_server_players(&mut conn, Some(guild_id)).await?;

    if player_servers.len() >= ctx.data().config.max_players_per_server {
        discord_helper::private_reply(
            &ctx,
            format!(
                "Maximum number of players ({}) reached for this server.",
                ctx.data().config.max_players_per_server
            ),
        )
        .await?;
        return Ok(());
    }

    let display_name = discord_user.display_name().to_string();
    if player_servers.iter().any(|ps| ps.player_id == player_id) {
        discord_helper::private_reply(
            &ctx,
            format!("Dota player {display_name} ({player_id}) is already on this server"),
        )
        .await?;
        return Ok(());
    }

    players_db::try_add_player(&mut conn, player_id).await?;

    let discord_user_id = discord_user.id.get() as i64;
    let discord_name = discord_user.global_name.unwrap_or(discord_user.name);
    if let Some(existing) = player_servers
        .iter()
        .find(|ps| ps.user_id == discord_user_id)
    {
        discord_helper::private_reply(
            &ctx,
            format!(
                "Discord user {display_name} is already registered on this server as player {}",
                existing.player_id
            ),
        )
        .await?;
        return Ok(());
    }

    info!("Inserting: {display_name} to Player Server: {server_name} (ID: {guild_id})");
    player_servers_db::insert_player_server(
        &mut conn,
        guild_id,
        player_id,
        discord_user.id.get() as i64,
        &discord_name,
        name.as_deref(),
    )
    .await?;
    discord_helper::private_reply(
        &ctx,
        format!("Player {discord_name} ({player_id}) has been added to this server."),
    )
    .await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn remove_player(
    ctx: Context<'_>,
    #[description = "The Discord user"] discord_user: User,
) -> Result<(), Error> {
    let guild_id = discord_helper::guild_id(&ctx)?;
    let server_name = discord_helper::guild_name(&ctx)?;
    let mut conn = database_access::get_new_connection().await?;

    if !discord_helper::validate_command(&ctx, &mut conn, guild_id).await? {
        return Ok(());
    }

    let removed = player_servers_db::remove_server_player_by_user_id(
        &mut conn,
        guild_id,
        discord_user.id.get() as i64,
    )
    .await?;

    let display_name = discord_user.display_name();
    let message = if removed {
        format!("Removed player: {display_name} from this server.")
    } else {
        format!("Player: {display_name} does not exist on this server.")
    };
    discord_helper::private_reply(&ctx, message).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn rename_player(
    ctx: Context<'_>,
    #[description = "The Discord user"] discord_user: User,
    #[description = "The new custom name for the player on this server"] new_name: String,
) -> Result<(), Error> {
    let guild_id = discord_helper::guild_id(&ctx)?;
    let server_name = discord_helper::guild_name(&ctx)?;
    let mut conn = database_access::get_new_connection().await?;
    if !discord_helper::validate_command(&ctx, &mut conn, guild_id).await? {
        return Ok(());
    }

    let new_name = new_name.trim().to_string();
    if new_name.is_empty() {
        discord_helper::private_reply(&ctx, "Player name cannot be empty.".to_owned()).await?;
        return Ok(());
    }

    let display_name = discord_user.display_name();
    info!(
        "Request to Rename Player: {display_name} to {new_name} on Server: {server_name} (ID: {guild_id})"
    );

    let renamed = player_servers_db::rename_server_player_by_user_id(
        &mut conn,
        guild_id,
        discord_user.id.get() as i64,
        &new_name,
    )
    .await?;

    let message = if renamed {
        format!("Renamed player: {display_name} to {new_name} on this server.")
    } else {
        format!("Player: {display_name} does not exist on this server.")
    };
    discord_helper::private_reply(&ctx, message).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn register_server(
    ctx: Context<'_>,
    #[description = "The server id to register"] server_id: Option<i64>,
) -> Result<(), Error> {
    let guild_id = match server_id {
        Some(id) => id,
        None => discord_helper::guild_id(&ctx)?,
    };
    let server_name = discord_helper::guild_name(&ctx)?;

    let mut conn = database_access::get_new_connection().await?;
    match servers_db::query_server_by_id(&mut conn, guild_id).await? {
        Some(_) => {
            discord_helper::private_reply(
                &ctx,
                format!("This server is already registered as a Dotacord server."),
            )
            .await?;
            Ok(())
        }
        None => {
            info!("Registering Server: {server_name} (ID: {guild_id})");
            servers_db::insert_server(&mut conn, guild_id, server_name, None).await?;
            discord_helper::private_reply(
                &ctx,
                format!("Server has been registered as a Dotacord server."),
            )
            .await?;
            Ok(())
        }
    }
}

pub fn format_list_players(players: &Vec<PlayerServer>) -> String {
    let title = format!("{} player(s) registered to this server:", players.len());

    if players.is_empty() {
        return format!("{}\nNo data available.", title);
    }

    let mut sorted_players: Vec<&PlayerServer> = players.iter().collect();
    sorted_players.sort_by(|a, b| a.display_name().cmp(&b.display_name()));

    let section = TableBuilder::new(title.clone())
        .add_column(Column::new("Discord User", move |s: &PlayerServer| {
            format!("@{}", s.discord_name.clone())
        }))
        .add_column(Column::new("Nickname", |s: &PlayerServer| {
            s.player_name.clone().unwrap_or("-".to_string())
        }))
        .add_column(Column::new("Player ID", |s: &PlayerServer| {
            s.player_id.to_string()
        }))
        .build(sorted_players);

    let mut lines: Vec<String> = Vec::new();
    lines.push(section.title);
    lines.extend(section.lines);
    lines.join("\n")
}
