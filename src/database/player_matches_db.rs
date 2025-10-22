use sqlx::{FromRow, SqliteConnection};
use tracing::debug;

use crate::api::open_dota_api::ApiPlayerMatch;
use crate::database::hero_cache;
use crate::database::types::{Faction, GameMode, LobbyType, MapperError};
use crate::Error;

#[derive(Debug, Clone, FromRow)]
pub(crate) struct PlayerMatch {
    pub match_id: i64,
    pub player_id: i64,
    pub hero_id: i32,
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub rank: i32,
    pub party_size: i32,
    pub faction: i32,
    pub is_victory: bool,
    pub start_time: i64,
    pub duration: i32,
    pub game_mode: i32,
    pub lobby_type: i32,
}

pub(crate) fn map_to_player_match(
    api_match: &ApiPlayerMatch,
    player_id: i64,
) -> Result<Option<PlayerMatch>, MapperError> {
    let match_id = api_match.match_id;
    if match_id == 1439386853 {
        return Ok(None);
    }
    let hero_id = match api_match.hero_id {
        Some(id) if id != 0 => id,
        _ => return Ok(None),
    };

    if hero_cache::get_hero_by_id(hero_id).is_none() {
        return Err(MapperError::UnknownHero { hero_id, match_id });
    }

    if matches!(api_match.leaver_status, Some(1 | 2)) {
        return Ok(None);
    }

    let game_mode_value = api_match.game_mode.ok_or(MapperError::MissingField {
        field: "game_mode",
        match_id,
    })?;
    let lobby_type_value = api_match.lobby_type.ok_or(MapperError::MissingField {
        field: "lobby_type",
        match_id,
    })?;

    let game_mode =
        GameMode::try_from(game_mode_value).map_err(|_| MapperError::InvalidGameMode {
            value: game_mode_value,
            match_id,
        })?;
    let lobby_type =
        LobbyType::try_from(lobby_type_value).map_err(|_| MapperError::InvalidLobbyType {
            value: lobby_type_value,
            match_id,
        })?;

    const RELEVANT_GAME_MODES: [GameMode; 2] = [GameMode::Ranked, GameMode::AllPick];
    const RELEVANT_LOBBY_TYPES: [LobbyType; 3] = [
        LobbyType::Unranked,
        LobbyType::Ranked,
        LobbyType::RankedSolo,
    ];

    if !RELEVANT_GAME_MODES.contains(&game_mode) || !RELEVANT_LOBBY_TYPES.contains(&lobby_type) {
        return Ok(None);
    }

    let start_time = api_match
        .start_time_seconds
        .ok_or(MapperError::MissingField {
            field: "start_time_seconds",
            match_id,
        })?;
    let duration = api_match.duration.ok_or(MapperError::MissingField {
        field: "duration",
        match_id,
    })?;
    if duration < 0 {
        return Err(MapperError::InvalidDuration {
            value: duration,
            match_id,
        });
    }

    let kills = api_match.kills.ok_or(MapperError::MissingField {
        field: "kills",
        match_id,
    })?;
    let deaths = api_match.deaths.ok_or(MapperError::MissingField {
        field: "deaths",
        match_id,
    })?;
    let assists = api_match.assists.ok_or(MapperError::MissingField {
        field: "assists",
        match_id,
    })?;

    let player_slot = api_match.player_slot.ok_or(MapperError::MissingField {
        field: "player_slot",
        match_id,
    })?;
    let radiant_win = api_match.radiant_win.ok_or(MapperError::MissingField {
        field: "radiant_win",
        match_id,
    })?;

    let faction = Faction::from_player_slot(player_slot);
    let is_victory = matches!(faction, Faction::Radiant) == radiant_win;

    Ok(Some(PlayerMatch {
        match_id,
        player_id,
        hero_id,
        kills,
        deaths,
        assists,
        rank: api_match.average_rank.unwrap_or(0),
        party_size: api_match.party_size.unwrap_or(0),
        faction: faction.as_i32(),
        is_victory,
        start_time,
        duration,
        game_mode: game_mode.as_i32(),
        lobby_type: lobby_type.as_i32(),
    }))
}

pub async fn insert_player_match(
    conn: &mut SqliteConnection,
    player_match: &PlayerMatch,
) -> Result<(), Error> {
    debug!(
        match_id = player_match.match_id,
        player_id = player_match.player_id,
        "Inserting player match"
    );
    sqlx::query(
        r#"
            INSERT INTO player_matches (
                match_id,
                player_id,
                hero_id,
                kills,
                deaths,
                assists,
                rank,
                party_size,
                faction,
                is_victory,
                start_time,
                duration,
                game_mode,
                lobby_type
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(player_match.match_id)
    .bind(player_match.player_id)
    .bind(player_match.hero_id)
    .bind(player_match.kills)
    .bind(player_match.deaths)
    .bind(player_match.assists)
    .bind(player_match.rank)
    .bind(player_match.party_size)
    .bind(player_match.faction)
    .bind(if player_match.is_victory { 1 } else { 0 })
    .bind(player_match.start_time)
    .bind(player_match.duration)
    .bind(player_match.game_mode)
    .bind(player_match.lobby_type)
    .execute(conn)
    .await?;

    Ok(())
}

#[allow(dead_code)]
pub async fn query_matches_by_player_id(
    conn: &mut SqliteConnection,
    player_id: i64,
) -> Result<Vec<PlayerMatch>, Error> {
    let rows: Vec<PlayerMatch> = sqlx::query_as(
        r#"
            SELECT
                match_id,
                player_id,
                hero_id,
                kills,
                deaths,
                assists,
                rank,
                party_size,
                faction,
                is_victory,
                start_time,
                duration,
                game_mode,
                lobby_type
            FROM player_matches
            WHERE player_id = ?
        "#,
    )
    .bind(player_id as i64)
    .fetch_all(conn)
    .await?;

    Ok(rows)
}

pub async fn query_matches_by_duration(
    conn: &mut SqliteConnection,
    player_id: i64,
    start_date: i32,
    end_date: i32,
) -> Result<Vec<PlayerMatch>, Error> {
    let rows: Vec<PlayerMatch> = sqlx::query_as(
        r#"
            SELECT
                pm.match_id,
                pm.player_id,
                pm.hero_id,
                pm.kills,
                pm.deaths,
                pm.assists,
                pm.rank,
                pm.party_size,
                pm.faction,
                pm.is_victory,
                pm.start_time,
                pm.duration,
                pm.game_mode,
                pm.lobby_type
            FROM player_matches pm
            WHERE pm.player_id = ?
              AND pm.start_time BETWEEN ? AND ?
        "#,
    )
    .bind(player_id as i64)
    .bind(start_date)
    .bind(end_date)
    .fetch_all(conn)
    .await?;

    Ok(rows)
}
