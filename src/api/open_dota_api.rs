use serde::Deserialize;
use tracing::info;

const BASE_URL: &str = "https://api.opendota.com/api";

#[derive(Debug, Clone, Deserialize)]
pub struct ApiPlayerMatch {
    pub match_id: i64,
    pub player_slot: Option<i32>,
    pub radiant_win: Option<bool>,
    pub duration: Option<i32>,
    pub game_mode: Option<i32>,
    pub lobby_type: Option<i32>,
    pub hero_id: Option<i32>,
    #[serde(rename = "start_time")]
    pub start_time_seconds: Option<i64>,
    pub version: Option<i32>,
    pub kills: Option<i32>,
    pub deaths: Option<i32>,
    pub assists: Option<i32>,
    pub average_rank: Option<i32>,
    pub skill: Option<i32>,
    pub leaver_status: Option<i32>,
    pub party_size: Option<i32>,
    pub hero_variant: Option<i32>,
}

#[tracing::instrument(level = "trace")]
pub(crate) async fn get_player_matches(
    player_id: i64,
) -> Result<Vec<ApiPlayerMatch>, reqwest::Error> {
    let url = format!("{BASE_URL}/players/{player_id}/matches");
    info!(player_id, url, "Fetching API player matches");
    let response = reqwest::Client::new()
        .get(url)
        .send()
        .await?
        .error_for_status()?;

    let matches = response.json::<Vec<ApiPlayerMatch>>().await?;
    info!(
        player_id,
        Count = matches.len(),
        "Fetched player matches from OpenDota"
    );

    Ok(matches)
}
