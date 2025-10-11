use chrono::{DateTime, Local, Utc};

/// Centralised date/time formats used across the project.
pub const SHORT_DATE_FORMAT: &str = "%-d %B %Y"; // e.g. 8 September 2025
pub const SECTION_DATE_FORMAT: &str = "%a, %d-%b-%y"; // e.g. Tue, 08-Sep-25

/// Format a DateTime<Utc> for short human display (used in immediate replies).
pub fn format_short(dt: DateTime<Utc>) -> String {
    dt.format(SHORT_DATE_FORMAT).to_string()
}

/// Format a timestamp (seconds since epoch) for leaderboard section rows.
pub fn format_section_timestamp(timestamp: i64) -> String {
    chrono::DateTime::<Utc>::from_timestamp(timestamp, 0)
        .unwrap_or_else(|| chrono::DateTime::<Utc>::from_timestamp(0, 0).unwrap())
        .format(SECTION_DATE_FORMAT)
        .to_string()
}

/// Format current local date as YYYY-MM-DD (used for log path replacements)
pub fn local_date_yyyy_mm_dd() -> String {
    let now: DateTime<Local> = Local::now();
    now.format("%Y-%m-%d").to_string()
}
