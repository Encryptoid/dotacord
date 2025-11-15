use std::collections::HashMap;

use super::player_matches_db::PlayerMatchModel;

fn get_most_played_hero(matches: &[PlayerMatchModel]) -> Option<i32> {
    matches
        .iter()
        .fold(HashMap::new(), |mut acc, m| {
            *acc.entry(m.hero_id).or_insert(0) += 1;
            acc
        })
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(hero, _)| hero)
}

#[derive(Debug, Clone, Default)]
pub struct HighlightedTotal {
    pub total: i32,
    pub record_value: Option<i32>,
    pub record_match_id: Option<i64>,
}

impl HighlightedTotal {
    pub fn update(&mut self, value: i32, match_id: i64) {
        self.total += value;

        let should_update = match self.record_value {
            Some(current) => value >= current,
            None => true,
        };

        if should_update {
            self.record_value = Some(value);
            self.record_match_id = Some(match_id);
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DurationAggregate {
    pub total_seconds: i32,
    pub longest_seconds: i32,
    pub longest_match_id: Option<i64>,
}

impl DurationAggregate {
    pub fn update(&mut self, seconds: i32, match_id: i64) {
        if seconds < 0 {
            return;
        }

        self.total_seconds += seconds;

        if self.longest_match_id.is_none() || seconds >= self.longest_seconds {
            self.longest_seconds = seconds;
            self.longest_match_id = Some(match_id);
        }
    }
}

#[derive(Debug)]
pub(crate) enum MapperError {
    MissingField { field: &'static str, match_id: i64 },
    InvalidDuration { value: i32, match_id: i64 },
    InvalidGameMode { value: i32, match_id: i64 },
    InvalidLobbyType { value: i32, match_id: i64 },
    UnknownHero { hero_id: i32, match_id: i64 },
}

impl std::fmt::Display for MapperError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingField { field, match_id } => {
                write!(f, "match {match_id}: missing {field}")
            }
            Self::InvalidDuration { value, match_id } => {
                write!(f, "match {match_id}: invalid duration {value}")
            }
            Self::InvalidGameMode { value, match_id } => {
                write!(f, "match {match_id}: invalid game mode {value}")
            }
            Self::InvalidLobbyType { value, match_id } => {
                write!(f, "match {match_id}: invalid lobby type {value}")
            }
            Self::UnknownHero { hero_id, match_id } => {
                write!(f, "match {match_id}: unknown hero id {hero_id}")
            }
        }
    }
}

impl std::error::Error for MapperError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub(crate) enum GameMode {
    Unknown = 0,
    AllPick = 1,
    CaptainsMode = 2,
    RandomDraft = 3,
    SingleDraft = 4,
    AllRandom = 5,
    LeastPlayed = 12,
    CaptainsDraft = 16,
    Ranked = 22,
    Turbo = 23,
}

impl GameMode {
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

impl TryFrom<i32> for GameMode {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Unknown,
            1 => Self::AllPick,
            2 => Self::CaptainsMode,
            3 => Self::RandomDraft,
            4 => Self::SingleDraft,
            5 => Self::AllRandom,
            12 => Self::LeastPlayed,
            16 => Self::CaptainsDraft,
            22 => Self::Ranked,
            23 => Self::Turbo,
            _ => return Err(()),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub(crate) enum LobbyType {
    Unranked = 0,
    Practice = 1,
    Tournament = 2,
    Tutorial = 3,
    CoopBots = 4,
    RankedTeam = 5,
    RankedSolo = 6,
    Ranked = 7,
    SoloMid1v1 = 8,
    BattleCup = 9,
}

impl LobbyType {
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

impl TryFrom<i32> for LobbyType {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Unranked,
            1 => Self::Practice,
            2 => Self::Tournament,
            3 => Self::Tutorial,
            4 => Self::CoopBots,
            5 => Self::RankedTeam,
            6 => Self::RankedSolo,
            7 => Self::Ranked,
            8 => Self::SoloMid1v1,
            9 => Self::BattleCup,
            _ => return Err(()),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub(crate) enum Faction {
    Radiant = 0,
    Dire = 1,
}

impl Faction {
    pub fn from_player_slot(slot: i32) -> Self {
        if slot < 128 {
            Self::Radiant
        } else {
            Self::Dire
        }
    }

    pub fn as_i32(self) -> i32 {
        self as i32
    }
}
