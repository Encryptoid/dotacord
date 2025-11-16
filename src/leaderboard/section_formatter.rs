use super::stats_calculator::PlayerStats;
use crate::api::open_dota_links;
use crate::database::hero_cache;
use crate::leaderboard::section::LeaderboardSection;
use crate::markdown::{Link, TableBuilder, Text};
use crate::util::dates::format_section_timestamp;
use crate::str;

pub fn build_winrate_section(
    duration_label: &str,
    all_stats: &[PlayerStats],
    selector: fn(&PlayerStats) -> (i32, i32),
    left_emoji: &str,
    right_emoji: &str,
    title_text: &str,
    win_rate_label: &str,
    include_links: bool,
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

    let mut builder = TableBuilder::new(title);
    if include_links {
        let link_urls: Vec<String> = sorted_stats
            .iter()
            .map(|s| open_dota_links::profile_url(s.player_id))
            .collect();
        builder = builder.add_column(Link::new(link_urls));
    }
    Some(
        builder
            .add_column(Text::new(
                "Player",
                sorted_stats.iter().map(|s| str!(s.player_name)).collect(),
            ))
            .add_column(Text::new(
                "Win%",
                sorted_stats
                    .iter()
                    .map(|s| {
                        let (wins, total) = selector(s);
                        if total > 0 {
                            let win_rate = (wins as f64 / total as f64) * 100.0;
                            format!("{:>3.0}%", win_rate)
                        } else {
                            str!("-")
                        }
                    })
                    .collect(),
            ))
            .add_column(Text::new(
                "Wins",
                sorted_stats
                    .iter()
                    .map(|s| {
                        let (wins, _) = selector(s);
                        str!(wins)
                    })
                    .collect(),
            ))
            .add_column(Text::new(
                "Total",
                sorted_stats
                    .iter()
                    .map(|s| {
                        let (_, total) = selector(s);
                        str!(total)
                    })
                    .collect(),
            ))
            .build(),
    )
}

pub fn build_hero_spam_section(
    duration_label: &str,
    all_stats: &[PlayerStats],
    left_emoji: &str,
    right_emoji: &str,
    label: &str,
    include_links: bool,
) -> Option<LeaderboardSection> {
    let mut sorted_stats: Vec<_> = all_stats
        .iter()
        .filter(|s| s.hero_pick_stat.stats.total_matches > 0)
        .collect();

    sorted_stats.sort_by(|a, b| {
        let a_rate =
            a.hero_pick_stat.stats.total_matches as f64 / a.overall_stats.total_matches as f64;
        let b_rate =
            b.hero_pick_stat.stats.total_matches as f64 / b.overall_stats.total_matches as f64;
        b_rate
            .partial_cmp(&a_rate)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.most_recent_match_time.cmp(&a.most_recent_match_time))
    });

    let winner = sorted_stats.first()?;
    let pick_rate = (winner.hero_pick_stat.stats.total_matches as f64
        / winner.overall_stats.total_matches as f64)
        * 100.0;
    let hero_name =
        hero_cache::get_hero_by_id(winner.hero_pick_stat.hero_id).unwrap_or("Unknown Hero");
    let title = format!(
        "[{duration_label}] - {left_emoji} {label} {right_emoji} - __*{}*__ - {:.0}% {} ({})",
        winner.player_name, pick_rate, "Hero Pick Rate", hero_name
    );

    let mut builder = TableBuilder::new(title);
    if include_links {
        let link_urls: Vec<String> = sorted_stats
            .iter()
            .map(|s| open_dota_links::profile_url(s.player_id))
            .collect();
        builder = builder.add_column(Link::new(link_urls));
    }
    Some(
        builder
            .add_column(Text::new(
                "Player",
                sorted_stats.iter().map(|s| str!(s.player_name)).collect(),
            ))
            .add_column(Text::new(
                "Hero",
                sorted_stats
                    .iter()
                    .map(|s| {
                        str!(hero_cache::get_hero_by_id(s.hero_pick_stat.hero_id)
                            .unwrap_or("Unknown Hero"))
                    })
                    .collect(),
            ))
            .add_column(Text::new(
                "Count",
                sorted_stats
                    .iter()
                    .map(|s| str!(s.hero_pick_stat.stats.total_matches))
                    .collect(),
            ))
            .add_column(Text::new(
                "Matches",
                sorted_stats
                    .iter()
                    .map(|s| str!(s.overall_stats.total_matches))
                    .collect(),
            ))
            .add_column(Text::new(
                "Pick%",
                sorted_stats
                    .iter()
                    .map(|s| {
                        let percentage = (s.hero_pick_stat.stats.total_matches as f32
                            / s.overall_stats.total_matches as f32)
                            * 100.0;
                        format!("{:>3.0}%", percentage)
                    })
                    .collect(),
            ))
            .build(),
    )
}

pub fn build_single_match_stat_section(
    duration_label: &str,
    all_stats: &[PlayerStats],
    selector: fn(&PlayerStats) -> &super::stats_calculator::SingleMatchStat,
    left_emoji: &str,
    right_emoji: &str,
    label: &str,
    stat_name: &str,
    include_links: bool,
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

    let player_column: Vec<String> = sorted_stats.iter().map(|s| str!(s.player_name)).collect();
    let stat_column: Vec<String> = sorted_stats
        .iter()
        .map(|s| str!(selector(s).value))
        .collect();
    let hero_column: Vec<String> = sorted_stats
        .iter()
        .map(|s| str!(hero_cache::get_hero_by_id(selector(s).hero_id).unwrap_or("Unknown Hero")))
        .collect();
    let outcome_column: Vec<String> = sorted_stats
        .iter()
        .map(|s| {
            str!(if selector(s).is_victory {
                "Win"
            } else {
                "Loss"
            })
        })
        .collect();
    let average_column: Vec<String> = sorted_stats
        .iter()
        .map(|s| format!("{:.2}", selector(s).average))
        .collect();
    let total_column: Vec<String> = sorted_stats
        .iter()
        .map(|s| str!(selector(s).total))
        .collect();
    let date_column: Vec<String> = sorted_stats
        .iter()
        .map(|s| format_section_timestamp(selector(s).date))
        .collect();
    let link_urls: Vec<String> = if include_links {
        sorted_stats
            .iter()
            .map(|s| open_dota_links::match_url(selector(s).match_id))
            .collect()
    } else {
        sorted_stats
            .iter()
            .map(|s| open_dota_links::profile_url(s.player_id))
            .collect()
    };

    let mut builder = TableBuilder::new(title);
    if include_links {
        builder = builder.add_column(Link::new(link_urls));
    }
    builder = builder
        .add_column(Text::new("Player", player_column))
        .add_column(Text::new(stat_name, stat_column))
        .add_column(Text::new("Hero", hero_column))
        .add_column(Text::new("Outcome", outcome_column))
        .add_column(Text::new("Average", average_column))
        .add_column(Text::new("Total", total_column))
        .add_column(Text::new("Date", date_column));
    let section = builder.build();
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
    include_links: bool,
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

    let player_column: Vec<String> = sorted_stats.iter().map(|s| str!(s.player_name)).collect();
    let duration_column: Vec<String> = sorted_stats
        .iter()
        .map(|s| format_duration(s.longest_match_stat.value))
        .collect();
    let hero_column: Vec<String> = sorted_stats
        .iter()
        .map(|s| {
            str!(hero_cache::get_hero_by_id(s.longest_match_stat.hero_id).unwrap_or("Unknown Hero"))
        })
        .collect();
    let outcome_column: Vec<String> = sorted_stats
        .iter()
        .map(|s| {
            str!(if s.longest_match_stat.is_victory {
                "Win"
            } else {
                "Loss"
            })
        })
        .collect();
    let average_column: Vec<String> = sorted_stats
        .iter()
        .map(|s| format_duration(s.longest_match_stat.average as i32))
        .collect();
    let total_column: Vec<String> = sorted_stats
        .iter()
        .map(|s| format_duration(s.longest_match_stat.total))
        .collect();
    let date_column: Vec<String> = sorted_stats
        .iter()
        .map(|s| format_section_timestamp(s.longest_match_stat.date))
        .collect();
    let link_urls: Vec<String> = if include_links {
        sorted_stats
            .iter()
            .map(|s| open_dota_links::match_url(s.longest_match_stat.match_id))
            .collect()
    } else {
        sorted_stats
            .iter()
            .map(|s| open_dota_links::profile_url(s.player_id))
            .collect()
    };

    let mut builder = TableBuilder::new(title);
    if include_links {
        builder = builder.add_column(Link::new(link_urls));
    }
    builder = builder
        .add_column(Text::new("Player", player_column))
        .add_column(Text::new("Duration", duration_column))
        .add_column(Text::new("Hero", hero_column))
        .add_column(Text::new("Outcome", outcome_column))
        .add_column(Text::new("Average", average_column))
        .add_column(Text::new("Total", total_column))
        .add_column(Text::new("Date", date_column));
    let section = builder.build();
    Some(section)
}
