use chrono::{DateTime, Utc};
use tracing::info;

use crate::database::{command_events_db, player_matches_db, player_servers_db};
use crate::leaderboard::emoji::Emoji;
use crate::leaderboard::section::LeaderboardSection;
use crate::leaderboard::stats_calculator::{self, PlayerStats};
use crate::leaderboard::{leaderboard_stats, sections};
use crate::util::dates;
use crate::Error;

pub async fn get_leaderboard_messages(
    server_id: i64,
    players: Vec<player_servers_db::PlayerServerModel>,
    start_utc: &DateTime<Utc>,
    end_utc: &DateTime<Utc>,
    duration_label: &str,
) -> Result<Vec<String>, Error> {
    let all_stats = leaderboard_stats::get_player_stats(players, &start_utc, &end_utc).await?;
    let sections = sections::get_leaderboard_sections(&duration_label, &all_stats);

    let last_reload = command_events_db::query_last_event(server_id, command_events_db::EventType::AdminRefresh, None).await?;
    let last_refreshed = match last_reload {
        Some(event) => dates::discord_relative_from_timestamp(event.event_time),
        None => "Never".to_string(),
    };

    let section_messages: Vec<String> = sections
        .iter()
        .filter_map(|s| s.as_ref())
        .map(|s| leaderboard_stats::section_to_msg_content(&s))
        .collect();

    if section_messages.is_empty() {
        return Ok(vec![]);
    }

    let title = format!(
        "# {} {}ly Leaderboard {} - {} -> {}\n> Last refreshed: {}\n",
        Emoji::TOP1,
        duration_label,
        Emoji::AEGIS2015,
        dates::discord_date(*start_utc),
        dates::discord_date(*end_utc),
        last_refreshed
    );

    let mut messages = vec![title];
    messages.extend(section_messages);
    Ok(messages)
}

async fn get_player_stats(
    players: Vec<player_servers_db::PlayerServerModel>,
    start_utc: &DateTime<Utc>,
    end_utc: &DateTime<Utc>,
) -> Result<Vec<PlayerStats>, Error> {
    let mut all_stats = Vec::new();
    for player in players {
        let matches = player_matches_db::query_matches_by_duration(
            player.player_id,
            start_utc.timestamp() as i32,
            end_utc.timestamp() as i32,
        )
        .await?;

        if matches.is_empty() {
            info!(
                player_id = player.player_id,
                "No matches found for player in duration"
            );
            continue;
        }

        let stats = stats_calculator::player_matches_to_stats(
            &matches,
            player.player_id,
            player
                .player_name
                .clone()
                .unwrap_or_else(|| player.discord_name.clone()),
        )?;
        all_stats.push(stats);
    }

    Ok(all_stats)
}

fn section_to_msg_content(section: &LeaderboardSection) -> String {
    let mut content = format!("### {}\n", section.title);
    for line in &section.lines {
        content.push_str(&format!("{}\n", line));
    }
    content
}

