use std::collections::HashMap;

use chrono::{DateTime, Utc};
use llm::builder::{FunctionBuilder, ParamBuilder};
use llm::ToolCall;
use serde::Serialize;
use tracing::info;

use crate::api::open_dota_links;
use crate::database::{heroes_db, player_matches_db, player_servers_db};
use crate::Error;

const MAX_TOOL_ROUNDS: usize = 5;

pub struct ToolContext {
    pub server_id: i64,
    pub max_recent_match_days: u64,
    pub max_recent_matches: usize,
}

pub fn max_tool_rounds() -> usize {
    MAX_TOOL_ROUNDS
}

pub fn get_recent_matches_tool() -> FunctionBuilder {
    FunctionBuilder::new("get_recent_matches")
        .description(
            "Get recent Dota 2 matches for a player registered in this Discord server. \
             Returns match summaries including hero, K/D/A, result, and friends who played together.",
        )
        .param(
            ParamBuilder::new("username")
                .type_of("string")
                .description("Discord display name or Dota username of the player"),
        )
        .required(vec!["username".to_string()])
}

pub fn get_hero_by_nickname_tool() -> FunctionBuilder {
    FunctionBuilder::new("get_hero_by_nickname")
        .description(
            "Use this if the user is asking about a hero that you do not know, \
             or has returned bad results from other tasks.",
        )
        .param(
            ParamBuilder::new("nickname")
                .type_of("string")
                .description("Hero name or nickname (e.g. 'Tree' for Treant Protector, or 'Storm Spirit')"),
        )
        .required(vec!["nickname".to_string()])
}

pub fn get_match_details_tool() -> FunctionBuilder {
    FunctionBuilder::new("get_match_details")
        .description(
            "Get detailed information about a specific Dota 2 match. \
             Returns stats for all server players who participated, plus an OpenDota link for full details.",
        )
        .param(
            ParamBuilder::new("match_id")
                .type_of("integer")
                .description("The Dota 2 match ID"),
        )
        .required(vec!["match_id".to_string()])
}

pub async fn execute_tool(tool_call: &ToolCall, ctx: &ToolContext) -> Result<String, Error> {
    info!(tool_name = %tool_call.function.name, arguments = %tool_call.function.arguments, "Executing tool");

    match tool_call.function.name.as_str() {
        "get_recent_matches" => execute_get_recent_matches(&tool_call.function.arguments, ctx).await,
        "get_match_details" => execute_get_match_details(&tool_call.function.arguments, ctx).await,
        "get_hero_by_nickname" => execute_get_hero_by_nickname(&tool_call.function.arguments).await,
        unknown => Ok(format!("{{\"error\": \"Unknown tool: {unknown}\"}}")),
    }
}

#[derive(Serialize)]
struct MatchSummary {
    match_id: i64,
    hero: String,
    won: bool,
    kills: i32,
    deaths: i32,
    assists: i32,
    date: String,
    duration_minutes: i32,
    friends: Vec<String>,
}

#[derive(Serialize)]
struct RecentMatchesResponse {
    player_name: String,
    matches: Vec<MatchSummary>,
    total_matches: usize,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

async fn execute_get_recent_matches(arguments: &str, ctx: &ToolContext) -> Result<String, Error> {
    let args: serde_json::Value = serde_json::from_str(arguments)?;
    let username = args["username"]
        .as_str()
        .ok_or_else(|| Error::from("Missing 'username' parameter"))?;

    let server_players = player_servers_db::query_server_players(ctx.server_id).await?;

    let target = find_player_by_name(username, &server_players);
    let Some(target) = target else {
        let available: Vec<&str> = server_players.iter().map(|p| p.discord_name.as_str()).collect();
        return Ok(serde_json::to_string(&ErrorResponse {
            error: format!(
                "Player '{}' not found in this server. Available players: {}",
                username,
                available.join(", ")
            ),
        })?);
    };

    let display_name = target
        .player_name
        .clone()
        .unwrap_or_else(|| target.discord_name.clone());

    let now = Utc::now();
    let start = now - chrono::Duration::days(ctx.max_recent_match_days as i64);
    let start_ts = start.timestamp() as i32;
    let end_ts = now.timestamp() as i32;

    let matches = player_matches_db::query_matches_by_duration(target.player_id, start_ts, end_ts).await?;

    let mut sorted_matches = matches;
    sorted_matches.sort_by(|a, b| b.start_time.cmp(&a.start_time));
    sorted_matches.truncate(ctx.max_recent_matches);

    if sorted_matches.is_empty() {
        return Ok(serde_json::to_string(&RecentMatchesResponse {
            player_name: display_name,
            matches: vec![],
            total_matches: 0,
        })?);
    }

    let match_ids: std::collections::HashSet<i64> =
        sorted_matches.iter().map(|m| m.match_id).collect();

    let hero_lookup = heroes_db::HeroLookup::load().await?;
    let friends_map = build_friends_map(
        &match_ids,
        target.player_id,
        &server_players,
        start_ts,
        end_ts,
    )
    .await?;

    let summaries: Vec<MatchSummary> = sorted_matches
        .iter()
        .map(|m| {
            let hero = hero_lookup
                .get_name(m.hero_id)
                .unwrap_or("Unknown Hero")
                .to_string();

            let date = DateTime::from_timestamp(m.start_time, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M UTC").to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            let friends = friends_map
                .get(&m.match_id)
                .cloned()
                .unwrap_or_default();

            MatchSummary {
                match_id: m.match_id,
                hero,
                won: m.is_victory,
                kills: m.kills,
                deaths: m.deaths,
                assists: m.assists,
                date,
                duration_minutes: m.duration / 60,
                friends,
            }
        })
        .collect();

    let total = summaries.len();
    Ok(serde_json::to_string(&RecentMatchesResponse {
        player_name: display_name,
        matches: summaries,
        total_matches: total,
    })?)
}

fn find_player_by_name<'a>(
    username: &str,
    players: &'a [player_servers_db::PlayerServerModel],
) -> Option<&'a player_servers_db::PlayerServerModel> {
    let lower = username.to_lowercase();

    // Exact match on discord_name
    if let Some(p) = players.iter().find(|p| p.discord_name.to_lowercase() == lower) {
        return Some(p);
    }

    // Exact match on player_name
    if let Some(p) = players.iter().find(|p| {
        p.player_name
            .as_ref()
            .is_some_and(|n| n.to_lowercase() == lower)
    }) {
        return Some(p);
    }

    // Partial match on discord_name
    if let Some(p) = players.iter().find(|p| p.discord_name.to_lowercase().contains(&lower)) {
        return Some(p);
    }

    // Partial match on player_name
    players.iter().find(|p| {
        p.player_name
            .as_ref()
            .is_some_and(|n| n.to_lowercase().contains(&lower))
    })
}

async fn build_friends_map(
    target_match_ids: &std::collections::HashSet<i64>,
    target_player_id: i64,
    server_players: &[player_servers_db::PlayerServerModel],
    start_ts: i32,
    end_ts: i32,
) -> Result<HashMap<i64, Vec<String>>, Error> {
    let mut friends_map: HashMap<i64, Vec<String>> = HashMap::new();

    for player in server_players {
        if player.player_id == target_player_id {
            continue;
        }

        let their_matches =
            player_matches_db::query_matches_by_duration(player.player_id, start_ts, end_ts)
                .await?;

        let friend_name = player
            .player_name
            .clone()
            .unwrap_or_else(|| player.discord_name.clone());

        for m in &their_matches {
            if target_match_ids.contains(&m.match_id) {
                friends_map
                    .entry(m.match_id)
                    .or_default()
                    .push(friend_name.clone());
            }
        }
    }

    Ok(friends_map)
}

#[derive(Serialize)]
struct MatchPlayerDetail {
    player_name: String,
    hero: String,
    kills: i32,
    deaths: i32,
    assists: i32,
    won: bool,
    faction: String,
}

#[derive(Serialize)]
struct MatchDetailsResponse {
    match_id: i64,
    date: String,
    duration_minutes: i32,
    opendota_url: String,
    players: Vec<MatchPlayerDetail>,
}

async fn execute_get_match_details(arguments: &str, ctx: &ToolContext) -> Result<String, Error> {
    let args: serde_json::Value = serde_json::from_str(arguments)?;
    let match_id = args["match_id"]
        .as_i64()
        .ok_or_else(|| Error::from("Missing 'match_id' parameter"))?;

    let match_records = player_matches_db::query_match_by_id(match_id).await?;

    if match_records.is_empty() {
        return Ok(serde_json::to_string(&ErrorResponse {
            error: format!(
                "No data found for match {}. It may not involve any players registered in this server.",
                match_id
            ),
        })?);
    }

    let server_players = player_servers_db::query_server_players(ctx.server_id).await?;
    let hero_lookup = heroes_db::HeroLookup::load().await?;
    let player_name_map: HashMap<i64, String> = server_players
        .iter()
        .map(|p| {
            let name = p
                .player_name
                .clone()
                .unwrap_or_else(|| p.discord_name.clone());
            (p.player_id, name)
        })
        .collect();

    let first = &match_records[0];
    let date = DateTime::from_timestamp(first.start_time, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let players: Vec<MatchPlayerDetail> = match_records
        .iter()
        .map(|m| {
            let player_name = player_name_map
                .get(&m.player_id)
                .cloned()
                .unwrap_or_else(|| format!("Player {}", m.player_id));

            let hero = hero_lookup
                .get_name(m.hero_id)
                .unwrap_or("Unknown Hero")
                .to_string();

            let faction = if m.faction == 0 {
                "Radiant".to_string()
            } else {
                "Dire".to_string()
            };

            MatchPlayerDetail {
                player_name,
                hero,
                kills: m.kills,
                deaths: m.deaths,
                assists: m.assists,
                won: m.is_victory,
                faction,
            }
        })
        .collect();

    Ok(serde_json::to_string(&MatchDetailsResponse {
        match_id,
        date,
        duration_minutes: first.duration / 60,
        opendota_url: open_dota_links::match_url(match_id),
        players,
    })?)
}

#[derive(Serialize)]
struct HeroLookupResponse {
    hero_id: i32,
    name: String,
    positions: Vec<String>,
    nicknames: Vec<String>,
}

async fn execute_get_hero_by_nickname(arguments: &str) -> Result<String, Error> {
    let args: serde_json::Value = serde_json::from_str(arguments)?;
    let nickname = args["nickname"]
        .as_str()
        .ok_or_else(|| Error::from("Missing 'nickname' parameter"))?;

    let hero_lookup = heroes_db::HeroLookup::load().await?;

    let Some(hero) = hero_lookup.find_by_name(nickname) else {
        return Ok(serde_json::to_string(&ErrorResponse {
            error: format!("No hero found matching '{}'.", nickname),
        })?);
    };

    let mut positions = Vec::new();
    if hero.is_carry { positions.push("Carry".to_string()); }
    if hero.is_mid { positions.push("Mid".to_string()); }
    if hero.is_offlane { positions.push("Offlane".to_string()); }
    if hero.is_support { positions.push("Support".to_string()); }

    let nicknames = hero_lookup.get_nicknames(hero.hero_id).to_vec();

    Ok(serde_json::to_string(&HeroLookupResponse {
        hero_id: hero.hero_id,
        name: hero.name.clone(),
        positions,
        nicknames,
    })?)
}
