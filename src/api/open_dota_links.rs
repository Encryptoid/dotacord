use crate::fmt;

pub fn profile_url(player_id: i64) -> String {
    fmt!("https://www.opendota.com/players/{}", player_id)
}

pub fn match_url(match_id: i64) -> String {
    fmt!("https://www.opendota.com/matches/{}", match_id)
}
