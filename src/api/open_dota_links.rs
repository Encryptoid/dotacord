pub fn profile_url(player_id: i64) -> String {
    format!("https://www.opendota.com/players/{}", player_id)
}

pub fn match_url(match_id: i64) -> String {
    format!("https://www.opendota.com/matches/{}", match_id)
}
