use super::stats_calculator::PlayerStats;
use crate::data::hero_cache;
use crate::data::player_servers_db::PlayerServer;
use crate::markdown::stats_formatter::HasPlayerId;
use crate::markdown::stats_formatter::{fmt_match_url, Column, LeaderboardSection, TableBuilder};
use crate::util::dates::format_section_timestamp;

impl HasPlayerId for PlayerStats {
    fn player_id(&self) -> i64 {
        self.player_id
    }
}

impl HasPlayerId for PlayerServer {
    fn player_id(&self) -> i64 {
        self.player_id
    }
}

pub fn build_winrate_section(
    duration_label: &str,
    all_stats: &[PlayerStats],
    selector: fn(&PlayerStats) -> (i32, i32),
    left_emoji: &str,
    right_emoji: &str,
    title_text: &str,
    win_rate_label: &str,
) -> Option<LeaderboardSection> {
    let mut sorted_stats: Vec<_> = all_stats.iter().filter(|s| selector(s).1 > 0).collect();

    sorted_stats.sort_by(|a, b| {
        let (a_wins, a_total) = selector(a);
        let (b_wins, b_total) = selector(b);
        let a_rate = a_wins as f64 / a_total as f64;
        let b_rate = b_wins as f64 / b_total as f64;
        b_rate
            .partial_cmp(&a_rate)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.most_recent_match_time.cmp(&a.most_recent_match_time))
    });

    let winner = sorted_stats.first()?;
    let (winner_wins, winner_total) = selector(winner);
    let win_rate = (winner_wins as f64 / winner_total as f64) * 100.0;
    let title = format!(
        "[{duration_label}] - {left_emoji} {title_text} {right_emoji} - __*{}*__ - {:.0}% {}",
        winner.player_name, win_rate, win_rate_label
    );

    let section = TableBuilder::new(title)
        .add_column(Column::new("Player", |s: &PlayerStats| {
            s.player_name.clone()
        }))
        .add_column(Column::new("Win%", move |s: &PlayerStats| {
            let (wins, total) = selector(s);
            if total > 0 {
                let win_rate = (wins as f64 / total as f64) * 100.0;
                format!("{:>3.0}%", win_rate)
            } else {
                "-".to_string()
            }
        }))
        .add_column(Column::new("Wins", move |s: &PlayerStats| {
            let (wins, total) = selector(s);
            if total > 0 {
                wins.to_string()
            } else {
                "-".to_string()
            }
        }))
        .add_column(Column::new("Total", move |s: &PlayerStats| {
            let (_, total) = selector(s);
            if total > 0 {
                total.to_string()
            } else {
                "-".to_string()
            }
        }))
        .build(sorted_stats);
    Some(section)
}

pub fn build_hero_spam_section(
    duration_label: &str,
    all_stats: &[PlayerStats],
    left_emoji: &str,
    right_emoji: &str,
    label: &str,
) -> Option<LeaderboardSection> {
    // Filter and sort once
    let mut sorted_stats: Vec<_> = all_stats
        .iter()
        .filter(|s| s.hero_pick_stat.matches > 0)
        .collect();

    sorted_stats.sort_by(|a, b| {
        let a_rate = a.hero_pick_stat.matches as f64 / a.overall_stats.total_matches as f64;
        let b_rate = b.hero_pick_stat.matches as f64 / b.overall_stats.total_matches as f64;
        b_rate
            .partial_cmp(&a_rate)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.most_recent_match_time.cmp(&a.most_recent_match_time))
    });

    let winner = sorted_stats.first()?;
    let pick_rate =
        (winner.hero_pick_stat.matches as f64 / winner.overall_stats.total_matches as f64) * 100.0;
    let hero_name =
        hero_cache::get_hero_by_id(winner.hero_pick_stat.hero_id).unwrap_or("Unknown Hero");
    let title = format!(
        "[{duration_label}] - {left_emoji} {label} {right_emoji} - __*{}*__ - {:.0}% {} ({})",
        winner.player_name, pick_rate, "Hero Pick Rate", hero_name
    );

    let section = TableBuilder::new(title)
        .add_column(Column::new("Player", |s: &PlayerStats| {
            s.player_name.clone()
        }))
        .add_column(Column::new("Hero", |s: &PlayerStats| {
            hero_cache::get_hero_by_id(s.hero_pick_stat.hero_id)
                .unwrap_or("Unknown Hero")
                .to_string()
        }))
        .add_column(Column::new("Count", |s: &PlayerStats| {
            s.hero_pick_stat.matches.to_string()
        }))
        .add_column(Column::new("Matches", |s: &PlayerStats| {
            s.overall_stats.total_matches.to_string()
        }))
        .add_column(Column::new("Pick%", |s: &PlayerStats| {
            let percentage =
                (s.hero_pick_stat.matches as f32 / s.overall_stats.total_matches as f32) * 100.0;
            format!("{:>3.0}%", percentage)
        }))
        .build(sorted_stats);
    Some(section)
}

pub fn build_single_match_stat_section(
    duration_label: &str,
    all_stats: &[PlayerStats],
    selector: fn(&PlayerStats) -> &super::stats_calculator::SingleMatchStat,
    left_emoji: &str,
    right_emoji: &str,
    label: &str,
    stat_name: &str,
) -> Option<LeaderboardSection> {
    // Filter and sort once
    let mut sorted_stats: Vec<_> = all_stats.iter().filter(|s| selector(s).value > 0).collect();

    sorted_stats.sort_by(|a, b| {
        selector(b)
            .value
            .cmp(&selector(a).value)
            .then_with(|| selector(b).date.cmp(&selector(a).date))
    });

    let winner = sorted_stats.first()?;
    let winner_stat = selector(winner);
    let hero_name = hero_cache::get_hero_by_id(winner_stat.hero_id).unwrap_or("Unknown Hero");
    let title = format!(
        "[{duration_label}] - {left_emoji} {label} {right_emoji} - __*{}*__ - {} {} ({})",
        winner.player_name, winner_stat.value, stat_name, hero_name
    );

    let section = TableBuilder::new(title)
        .add_column(
            Column::new("Player", |s: &PlayerStats| s.player_name.clone())
                .with_match_id_fn(move |s: &PlayerStats| Some(selector(s).match_id)),
        )
        .add_column(Column::new(stat_name, move |s: &PlayerStats| {
            selector(s).value.to_string()
        }))
        .add_column(Column::new("Hero", move |s: &PlayerStats| {
            hero_cache::get_hero_by_id(selector(s).hero_id)
                .unwrap_or("Unknown Hero")
                .to_string()
        }))
        .add_column(Column::new("Outcome", move |s: &PlayerStats| {
            if selector(s).is_victory {
                "Win"
            } else {
                "Loss"
            }
            .to_string()
        }))
        .add_column(Column::new("Average", move |s: &PlayerStats| {
            format!("{:.2}", selector(s).average)
        }))
        .add_column(Column::new("Total", move |s: &PlayerStats| {
            selector(s).total.to_string()
        }))
        .add_column(Column::new("Date", move |s: &PlayerStats| {
            format_section_timestamp(selector(s).date)
        }))
        .with_link_fn(move |s| fmt_match_url("@", selector(s).match_id))
        .build(sorted_stats);
    Some(section)
}

fn format_duration(seconds: i32) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    if hours > 0 {
        format!("{}h {:02}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

pub fn build_longest_match_section(
    duration_label: &str,
    all_stats: &[PlayerStats],
    left_emoji: &str,
    right_emoji: &str,
    label: &str,
    stat_name: &str,
) -> Option<LeaderboardSection> {
    // Filter and sort once
    let mut sorted_stats: Vec<_> = all_stats
        .iter()
        .filter(|s| s.longest_match_stat.value > 0)
        .collect();

    sorted_stats.sort_by(|a, b| {
        b.longest_match_stat
            .value
            .cmp(&a.longest_match_stat.value)
            .then_with(|| b.longest_match_stat.date.cmp(&a.longest_match_stat.date))
    });

    let winner = sorted_stats.first()?;
    let duration = format_duration(winner.longest_match_stat.value);
    let player_name = winner.player_name.as_str();

    let title = format!(
        "[{}] - {} {} {} - __*{}*__ - {} - {}",
        duration_label, left_emoji, label, right_emoji, player_name, stat_name, duration,
    );

    let section = TableBuilder::new(title)
        .add_column(
            Column::new("Player", |s: &PlayerStats| s.player_name.clone())
                .with_match_id_fn(|s: &PlayerStats| Some(s.longest_match_stat.match_id)),
        )
        .add_column(Column::new("Duration", |s: &PlayerStats| {
            format_duration(s.longest_match_stat.value)
        }))
        .add_column(Column::new("Hero", |s: &PlayerStats| {
            hero_cache::get_hero_by_id(s.longest_match_stat.hero_id)
                .unwrap_or("Unknown Hero")
                .to_string()
        }))
        .add_column(Column::new("Outcome", |s: &PlayerStats| {
            if s.longest_match_stat.is_victory {
                "Win"
            } else {
                "Loss"
            }
            .to_string()
        }))
        .add_column(Column::new("Average", |s: &PlayerStats| {
            format_duration(s.longest_match_stat.average as i32)
        }))
        .add_column(Column::new("Total", |s: &PlayerStats| {
            format_duration(s.longest_match_stat.total)
        }))
        .add_column(Column::new("Date", |s: &PlayerStats| {
            format_section_timestamp(s.longest_match_stat.date)
        }))
        .with_link_fn(|s| fmt_match_url("@", s.longest_match_stat.match_id))
        .build(sorted_stats);
    Some(section)
}
