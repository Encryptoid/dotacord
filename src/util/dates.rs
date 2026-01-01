use chrono::{DateTime, Local, Utc};

pub fn local_date_yyyy_mm_dd() -> String {
    let now: DateTime<Local> = Local::now();
    now.format("%Y-%m-%d").to_string()
}

pub fn discord_date(dt: DateTime<Utc>) -> String {
    format!("<t:{}:D>", dt.timestamp())
}

pub fn discord_relative_from_timestamp(timestamp: i64) -> String {
    format!("<t:{}:R>", timestamp)
}

pub fn format_short_date_from_timestamp(timestamp: i64) -> String {
    chrono::DateTime::<Utc>::from_timestamp(timestamp, 0)
        .map(|dt| dt.format("%d-%b-%y").to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}
