use crate::database::player_matches_db::PlayerMatchModel;
use crate::Error;

#[derive(Debug, Clone, Default)]
pub struct PlayerStats {
    pub player_id: i64,
    pub player_name: String,

    pub overall_stats: OverallStats,
    pub ranked_stats: OverallStats,
    pub hero_pick_stat: HeroPickStats,

    pub most_kills_stat: SingleMatchStat,
    pub most_assists_stat: SingleMatchStat,
    pub most_deaths_stat: SingleMatchStat,
    pub longest_match_stat: SingleMatchStat,

    pub most_recent_match_time: i64,
}

#[derive(Debug, Clone, Default)]
pub struct OverallStats {
    pub total_matches: i32,
    pub wins: i32,
}

impl OverallStats {
    pub fn new() -> Self {
        Self {
            total_matches: 0,
            wins: 0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct HeroPickStats {
    pub hero_id: i32,
    pub wins: i32,
    pub matches: i32,
}
impl HeroPickStats {
    fn new(hero_id: i32) -> Self {
        Self {
            hero_id,
            wins: 0,
            matches: 0,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SingleMatchStat {
    pub total: i32,
    pub value: i32,
    pub average: f32,

    pub match_id: i64,
    pub date: i64,
    pub hero_id: i32,
    pub is_victory: bool,
}

struct StatTracker<'a> {
    player_match: Option<&'a PlayerMatchModel>,
    value: i32,
    total: i32,
}

impl<'a> StatTracker<'a> {
    fn new() -> Self {
        Self {
            player_match: None,
            value: 0,
            total: 0,
        }
    }
}

#[tracing::instrument(level = "trace", skip(matches))]
pub fn player_matches_to_stats(
    matches: &[PlayerMatchModel],
    player_id: i64,
    player_name: String,
) -> Result<PlayerStats, Error> {
    let mut overall_stats = OverallStats::new();
    let mut ranked_stats = OverallStats::new();

    let mut hero_trackers: std::collections::HashMap<i32, HeroPickStats> =
        std::collections::HashMap::new();

    let mut highest_kills_tracker = StatTracker::new();
    let mut highest_assists_tracker = StatTracker::new();
    let mut highest_deaths_tracker = StatTracker::new();
    let mut longest_match_tracker = StatTracker::new();

    for player_match in matches {
        // Overall Stats
        overall_stats.total_matches += 1;
        if player_match.is_victory {
            overall_stats.wins += 1;
        }

        if player_match.lobby_type == 7 || player_match.lobby_type == 6 {
            ranked_stats.total_matches += 1;
            if player_match.is_victory {
                ranked_stats.wins += 1;
            }
        }

        // Most Played Hero
        let tracker = hero_trackers
            .entry(player_match.hero_id)
            .or_insert_with(|| HeroPickStats::new(player_match.hero_id));

        tracker.matches += 1;
        if player_match.is_victory {
            tracker.wins += 1;
        }

        // Single Match Stats
        highest_kills_tracker.total += player_match.kills;
        highest_assists_tracker.total += player_match.assists;
        highest_deaths_tracker.total += player_match.deaths;
        longest_match_tracker.total += player_match.duration;

        if player_match.kills >= highest_kills_tracker.value {
            highest_kills_tracker.player_match = Some(player_match);
            highest_kills_tracker.value = player_match.kills;
        }
        if player_match.assists >= highest_assists_tracker.value {
            highest_assists_tracker.player_match = Some(player_match);
            highest_assists_tracker.value = player_match.assists;
        }
        if player_match.deaths >= highest_deaths_tracker.value {
            highest_deaths_tracker.player_match = Some(player_match);
            highest_deaths_tracker.value = player_match.deaths;
        }
        if player_match.duration >= longest_match_tracker.value {
            longest_match_tracker.player_match = Some(player_match);
            longest_match_tracker.value = player_match.duration;
        }
    }

    let hero_pick_stat = hero_trackers
        .into_iter()
        .max_by_key(|(_, t)| t.matches)
        .ok_or_else(|| Error::from("No matches found for player when checking most played hero"))?
        .1;

    let total_matches = matches.len() as f32;
    let most_recent_match_time = matches.iter().map(|m| m.start_time).max().unwrap_or(0);

    Ok(PlayerStats {
        player_id,
        player_name,

        overall_stats,
        ranked_stats,
        hero_pick_stat,

        most_kills_stat: create_single_match_stat(&highest_kills_tracker, &total_matches)?,
        most_assists_stat: create_single_match_stat(&highest_assists_tracker, &total_matches)?,
        most_deaths_stat: create_single_match_stat(&highest_deaths_tracker, &total_matches)?,
        longest_match_stat: create_single_match_stat(&longest_match_tracker, &total_matches)?,

        most_recent_match_time,
    })
}

fn create_single_match_stat(
    tracker: &StatTracker,
    matches_len: &f32,
) -> Result<SingleMatchStat, Error> {
    let player_match = tracker
        .player_match
        .ok_or_else(|| Error::from("No matches found for player"))?;

    Ok(SingleMatchStat {
        total: tracker.total,
        value: tracker.value,
        average: tracker.total as f32 / matches_len,

        match_id: player_match.match_id,
        date: player_match.start_time,
        hero_id: player_match.hero_id,
        is_victory: player_match.is_victory,
    })
}
