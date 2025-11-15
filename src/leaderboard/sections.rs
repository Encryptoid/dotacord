use crate::leaderboard::section::LeaderboardSection;

use super::emoji::Emoji;
use super::section_formatter;
use super::stats_calculator::PlayerStats;

pub(crate) fn get_leaderboard_sections(
    duration_label: &str,
    all_stats: &[PlayerStats],
) -> Vec<Option<LeaderboardSection>> {
    vec![
        format_overall_win_rate_section(duration_label, &all_stats, false),
        format_ranked_win_rate_section(duration_label, &all_stats, false),
        format_hero_spam_section(duration_label, &all_stats, false),
        format_highest_kills_section(duration_label, &all_stats, true),
        format_highest_assists_section(duration_label, &all_stats, true),
        format_highest_deaths_section(duration_label, &all_stats, true),
        format_longest_match_section(duration_label, &all_stats, true),
    ]
}

fn format_overall_win_rate_section(
    duration_label: &str,
    all_stats: &[PlayerStats],
    include_links: bool,
) -> Option<LeaderboardSection> {
    section_formatter::build_winrate_section(
        duration_label,
        all_stats,
        |s: &PlayerStats| (s.overall_stats.wins, s.overall_stats.total_matches),
        Emoji::AEGIS2015,
        Emoji::WIZ_WOW,
        &format!("Gamer of the {duration_label}"),
        "Overall Win Rate",
        include_links,
    )
}

fn format_ranked_win_rate_section(
    duration_label: &str,
    all_stats: &[PlayerStats],
    include_links: bool,
) -> Option<LeaderboardSection> {
    section_formatter::build_winrate_section(
        duration_label,
        all_stats,
        |s: &PlayerStats| (s.ranked_stats.wins, s.ranked_stats.total_matches),
        Emoji::ONLOOKER,
        Emoji::IMMORTAL,
        "Ranked Overlord",
        "Ranked Win Rate",
        include_links,
    )
}

fn format_hero_spam_section(
    duration_label: &str,
    all_stats: &[PlayerStats],
    include_links: bool,
) -> Option<LeaderboardSection> {
    section_formatter::build_hero_spam_section(
        duration_label,
        all_stats,
        Emoji::FROG,
        Emoji::SICK,
        "Filthiest Hero Spammer",
        include_links,
    )
}

fn format_highest_kills_section(
    duration_label: &str,
    all_stats: &[PlayerStats],
    include_links: bool,
) -> Option<LeaderboardSection> {
    section_formatter::build_single_match_stat_section(
        duration_label,
        all_stats,
        |s: &PlayerStats| &s.most_kills_stat,
        Emoji::DEVIL,
        Emoji::DOUBLEDAMAGE,
        "1v9 Miracle Child",
        "Kills",
        include_links,
    )
}

fn format_highest_assists_section(
    duration_label: &str,
    all_stats: &[PlayerStats],
    include_links: bool,
) -> Option<LeaderboardSection> {
    section_formatter::build_single_match_stat_section(
        duration_label,
        all_stats,
        |s: &PlayerStats| &s.most_assists_stat,
        Emoji::GIFF,
        Emoji::WIZ_GLHF,
        "Support Award",
        "Assists",
        include_links,
    )
}

fn format_highest_deaths_section(
    duration_label: &str,
    all_stats: &[PlayerStats],
    include_links: bool,
) -> Option<LeaderboardSection> {
    section_formatter::build_single_match_stat_section(
        duration_label,
        all_stats,
        |s: &PlayerStats| &s.most_deaths_stat,
        Emoji::POOP,
        Emoji::WIZ_HELP,
        "Head Chef",
        "Deaths",
        include_links,
    )
}

fn format_longest_match_section(
    duration_label: &str,
    all_stats: &[PlayerStats],
    include_links: bool,
) -> Option<LeaderboardSection> {
    section_formatter::build_longest_match_section(
        duration_label,
        all_stats,
        Emoji::SLEEPING,
        Emoji::IOSTRESS,
        "Most Traumatised",
        "Longest Match Duration",
        include_links,
    )
}
