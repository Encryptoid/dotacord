use poise::serenity_prelude::User;
use tracing::info;

use super::discord_helper;
use crate::api::open_dota_links;
use crate::database::player_servers_db::PlayerServerModel;
use crate::database::{database_access, player_servers_db, players_db, servers_db};
use crate::discord::discord_helper::get_command_ctx;
use crate::markdown::{Link, TableBuilder, Text};
use crate::{Context, Error};

// default_member_permissions = Permissions::ADMINISTRATOR;
#[poise::command(slash_command, guild_only)]
pub async fn list_players(ctx: Context<'_>) -> Result<(), Error> {
    let cmd_ctx = get_command_ctx(&ctx)?;
    let guild_id = discord_helper::guild_id(&ctx)?;
    if !discord_helper::validate_command(&ctx, guild_id).await? {
        return Ok(());
    }

    let player_servers =
        player_servers_db::query_server_players(&cmd_ctx.txn, Some(guild_id)).await?;

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
    #[description = "Name for the player to add to this server"] name: String,
    #[description = "Dota Player Id(taken from OpenDota/Dotabuff)"] player_id: i64,
) -> Result<(), Error> {
    let db = database_access::get_connection()?;
    let guild_id = discord_helper::guild_id(&ctx)?;
    let server_name = discord_helper::guild_name(&ctx)?;
    if !discord_helper::validate_command(&ctx, guild_id).await? {
        return Ok(());
    }

    let player_servers = player_servers_db::query_server_players(db, Some(guild_id)).await?;

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

    if player_servers.iter().any(|ps| ps.player_id == player_id) {
        discord_helper::private_reply(
            &ctx,
            format!("Dota player {name} ({player_id}) is already on this server"),
        )
        .await?;
        return Ok(());
    }

    players_db::try_add_player(db, player_id).await?;

    info!("Inserting: {name} to Player Server: {server_name} (ID: {guild_id})");
    player_servers_db::insert_player_server(db, guild_id, player_id, &name).await?;
    discord_helper::private_reply(
        &ctx,
        format!("Player {name} ({player_id}) has been added to this server."),
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
    let db = database_access::get_connection()?;

    if !discord_helper::validate_command(&ctx, guild_id).await? {
        return Ok(());
    }

    let removed = player_servers_db::remove_server_player_by_user_id(
        db,
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
    let db = database_access::get_connection()?;
    if !discord_helper::validate_command(&ctx, guild_id).await? {
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
        db,
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

    let db = database_access::get_connection()?;
    match servers_db::query_server_by_id(db, guild_id).await? {
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
            servers_db::insert_server(db, guild_id, server_name, None).await?;
            discord_helper::private_reply(
                &ctx,
                format!("Server has been registered as a Dotacord server."),
            )
            .await?;
            Ok(())
        }
    }
}

pub fn format_list_players(players: &Vec<PlayerServerModel>) -> String {
    let title = format!("{} player(s) registered to this server:", players.len());

    if players.is_empty() {
        return format!("{}\nNo data available.", title);
    }

    let mut sorted_players: Vec<&PlayerServerModel> = players.iter().collect();
    sorted_players.sort_by(|a, b| a.player_name.cmp(&b.player_name));

    let nicknames: Vec<String> = sorted_players
        .iter()
        .map(|s| s.player_name.clone())
        .collect();
    let player_ids: Vec<String> = sorted_players
        .iter()
        .map(|s| s.player_id.to_string())
        .collect();
    let links: Vec<String> = sorted_players
        .iter()
        .map(|s| open_dota_links::profile_url(s.player_id))
        .collect();

    let section = TableBuilder::new(title.clone())
        .add_column(Text::new("Player Name", nicknames))
        .add_column(Text::new("Player ID", player_ids))
        .add_column(Link::new(links))
        .build();

    let mut lines: Vec<String> = Vec::new();
    lines.push(section.title);
    lines.extend(section.lines);
    lines.join("\n")
}
