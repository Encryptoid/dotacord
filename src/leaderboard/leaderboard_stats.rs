use chrono::{DateTime, Utc};
use sqlx::SqliteConnection;
use tracing::info;

use crate::database::{player_matches_db, player_servers_db};
use crate::leaderboard::section::LeaderboardSection;
use crate::leaderboard::stats_calculator::{self, PlayerStats};
use crate::leaderboard::{leaderboard_stats, sections};
use crate::Error;

pub async fn get_leaderboard_messages(
    conn: &mut SqliteConnection,
    players: Vec<player_servers_db::PlayerServerModel>,
    start_utc: &DateTime<Utc>,
    end_utc: &DateTime<Utc>,
    duration_label: &str,
) -> Result<Vec<String>, Error> {
    let all_stats =
        leaderboard_stats::get_player_stats(conn, players, &start_utc, &end_utc).await?;
    let sections = sections::get_leaderboard_sections(&duration_label, &all_stats);
    Ok(sections
        .iter()
        .filter_map(|s| s.as_ref())
        .map(|s| leaderboard_stats::section_to_msg_content(&s))
        .collect())
}

async fn get_player_stats(
    conn: &mut SqliteConnection,
    players: Vec<player_servers_db::PlayerServerModel>,
    start_utc: &DateTime<Utc>,
    end_utc: &DateTime<Utc>,
) -> Result<Vec<PlayerStats>, Error> {
    let mut all_stats = Vec::new();
    for player in players {
        let matches = player_matches_db::query_matches_by_duration(
            conn,
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
            player.display_name().to_string(),
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
