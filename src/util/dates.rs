use chrono::{DateTime, Local, Utc};

pub fn local_date_yyyy_mm_dd() -> String {
    let now: DateTime<Local> = Local::now();
    now.format("%Y-%m-%d").to_string()
}

pub fn discord_date(dt: DateTime<Utc>) -> String {
    format!("<t:{}:D>", dt.timestamp())
}

pub fn discord_datetime(dt: DateTime<Utc>) -> String {
    format!("<t:{}:f>", dt.timestamp())
}

pub fn discord_relative(dt: DateTime<Utc>) -> String {
    format!("<t:{}:R>", dt.timestamp())
}

pub fn discord_datetime_from_timestamp(timestamp: i64) -> String {
    format!("<t:{}:f>", timestamp)
}

pub fn discord_relative_from_timestamp(timestamp: i64) -> String {
    format!("<t:{}:R>", timestamp)
}

pub fn discord_date_from_timestamp(timestamp: i64) -> String {
    format!("<t:{}:d>", timestamp)
}
