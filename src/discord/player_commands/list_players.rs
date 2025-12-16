use tracing::info;

use crate::api::open_dota_links;
use crate::database::player_servers_db::PlayerServerModel;
use crate::database::player_servers_db;
use crate::discord::discord_helper::{get_command_ctx, CmdCtx, Ephemeral};
use crate::markdown::{Link, TableBuilder, Text};
use crate::{Context, Error};

#[poise::command(slash_command, guild_only, rename = "list")]
pub async fn list_players_command(ctx: Context<'_>) -> Result<(), Error> {
    let cmd_ctx = get_command_ctx(ctx).await?;
    list_players(&cmd_ctx).await?;
    Ok(())
}

async fn list_players(ctx: &CmdCtx<'_>) -> Result<(), Error> {
    let player_servers = player_servers_db::query_server_players(ctx.guild_id).await?;

    let member = ctx
        .discord_ctx
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
        "No players are registered for this server. Add one with `/players add` or `/register` yourself."
            .to_string()
    };
    ctx.reply(Ephemeral::Private, content).await?;
    Ok(())
}

fn format_list_players(players: &Vec<PlayerServerModel>) -> String {
    let title = format!("{} player(s) registered to this server:", players.len());

    if players.is_empty() {
        return format!("{}\nNo data available.", title);
    }

    let mut sorted_players: Vec<&PlayerServerModel> = players.iter().collect();
    sorted_players.sort_by(|a, b| {
        let name_a = a
            .player_name
            .as_ref()
            .unwrap_or(&a.discord_name);
        let name_b = b
            .player_name
            .as_ref()
            .unwrap_or(&b.discord_name);
        name_a.cmp(name_b)
    });

    let nicknames: Vec<String> = sorted_players
        .iter()
        .map(|s| {
            s.player_name
                .clone()
                .unwrap_or_else(|| s.discord_name.clone())
        })
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
