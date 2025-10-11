use chrono::{DateTime, Utc};
use sqlx::SqliteConnection;
use tracing::info;

use crate::data::{player_matches_db, player_servers_db};
use crate::leaderboard::stats_calculator::{self, PlayerStats};
use crate::leaderboard::{leaderboard_stats, sections};
use crate::markdown::stats_formatter::{self, LeaderboardSection};
use crate::Error;

pub async fn get_leaderboard_messages(
    conn: &mut SqliteConnection,
    players: Vec<player_servers_db::PlayerServer>,
    start_utc: &DateTime<Utc>,
    end_utc: &DateTime<Utc>,
    duration_label: &str,
) -> Result<Vec<String>, Error> {
    let all_stats =
        leaderboard_stats::get_player_stats(conn, players, &start_utc, &end_utc).await?;
    let sections = get_leaderboard_sections(&duration_label, &all_stats);
    Ok(sections
        .iter()
        .filter_map(|s| s.as_ref())
        .map(|s| leaderboard_stats::section_to_msg_content(&s))
        .collect())
}

async fn get_player_stats(
    conn: &mut SqliteConnection,
    players: Vec<player_servers_db::PlayerServer>,
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

fn get_leaderboard_sections(
    duration_label: &str,
    all_stats: &[PlayerStats],
) -> Vec<Option<LeaderboardSection>> {
    vec![
        sections::format_overall_win_rate_section(duration_label, &all_stats),
        sections::format_ranked_win_rate_section(duration_label, &all_stats),
        sections::format_hero_spam_section(duration_label, &all_stats),
        sections::format_highest_kills_section(duration_label, &all_stats),
        sections::format_highest_assists_section(duration_label, &all_stats),
        sections::format_highest_deaths_section(duration_label, &all_stats),
        sections::format_longest_match_section(duration_label, &all_stats),
    ]
}

fn section_to_msg_content(section: &stats_formatter::LeaderboardSection) -> String {
    let mut content = format!("### {}\n", section.title);
    for line in &section.lines {
        content.push_str(&format!("{}\n", line));
    }
    content
}
