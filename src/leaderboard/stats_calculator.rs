use crate::database::player_matches_db::PlayerMatchModel;
use crate::database::types::LobbyType;
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

    pub fn track(&mut self, player_match: &PlayerMatchModel) {
        self.total_matches += 1;
        if player_match.is_victory {
            self.wins += 1;
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct HeroPickStats {
    pub hero_id: i32,
    pub stats: OverallStats,
}
impl HeroPickStats {
    fn new(hero_id: i32) -> Self {
        Self {
            hero_id,
            stats: OverallStats::new(),
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

/// Lifetime tied to PlayerMatchModel
struct SingleMatchTracker<'a> {
    player_match: Option<&'a PlayerMatchModel>,
    value: i32,
    total: i32,
}

impl<'a> SingleMatchTracker<'a> {
    fn new() -> Self {
        Self {
            player_match: None,
            value: 0,
            total: 0,
        }
    }

    // Add total and checks current max
    fn track(&mut self, player_match: &'a PlayerMatchModel, stat_value: i32) {
        self.total += stat_value;

        if stat_value >= self.value {
            self.player_match = Some(player_match);
            self.value = stat_value;
        };
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

    let mut highest_kills_tracker = SingleMatchTracker::new();
    let mut highest_assists_tracker = SingleMatchTracker::new();
    let mut highest_deaths_tracker = SingleMatchTracker::new();
    let mut longest_match_tracker = SingleMatchTracker::new();

    for player_match in matches {
        // Overall Stats
        overall_stats.track(player_match);

        if player_match.lobby_type == LobbyType::Ranked.as_i32()
            || player_match.lobby_type == LobbyType::RankedSolo.as_i32()
        {
            ranked_stats.track(player_match);
        }

        // Most Played Hero
        let hero_tracker = hero_trackers
            .entry(player_match.hero_id)
            .or_insert_with(|| HeroPickStats::new(player_match.hero_id));

        hero_tracker.stats.track(player_match);

        // Single Match Stats
        highest_kills_tracker.track(player_match, player_match.kills);
        highest_assists_tracker.track(player_match, player_match.assists);
        highest_deaths_tracker.track(player_match, player_match.deaths);
        longest_match_tracker.track(player_match, player_match.duration);
    }

    let hero_pick_stat = hero_trackers
        .into_iter()
        .max_by_key(|(_, t)| t.stats.total_matches)
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
    tracker: &SingleMatchTracker,
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
