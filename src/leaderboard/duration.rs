use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Utc};
use poise::ChoiceParameter;

#[derive(Debug, Clone, Copy, ChoiceParameter)]
pub(crate) enum Duration {
    Day,
    Week,
    Month,
    Year,
    #[name = "All Time"]
    AllTime,
}

impl Duration {
    pub(crate) fn start_date(self, end: DateTime<Utc>) -> DateTime<Utc> {
        match self {
            Duration::Day => end - chrono::Duration::days(1),
            Duration::Week => end - chrono::Duration::weeks(1),
            Duration::Month => subtract_months(end, 1),
            Duration::Year => subtract_months(end, 12),
            Duration::AllTime => Utc
                .with_ymd_and_hms(2010, 1, 1, 0, 0, 0)
                .single()
                .expect("valid all-time anchor"),
        }
    }

    /// Human-friendly short label for the duration (used in leaderboard headings).
    pub(crate) fn to_label(self) -> &'static str {
        match self {
            Duration::Day => "Day",
            Duration::Week => "Week",
            Duration::Month => "Month",
            Duration::Year => "Year",
            Duration::AllTime => "All Time",
        }
    }
}

fn subtract_months(end: DateTime<Utc>, months: u32) -> DateTime<Utc> {
    let naive = end.naive_utc();
    let date = naive.date();

    let mut year = date.year();
    let mut month = date.month() as i32 - months as i32;

    while month <= 0 {
        month += 12;
        year -= 1;
    }

    let month_u32 = month as u32;
    let day = date.day().min(days_in_month(year, month_u32));
    let replacement_date =
        NaiveDate::from_ymd_opt(year, month_u32, day).expect("valid date after month subtraction");

    DateTime::<Utc>::from_naive_utc_and_offset(replacement_date.and_time(naive.time()), Utc)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };

    let first_of_next =
        NaiveDate::from_ymd_opt(next_year, next_month, 1).expect("first of next month must exist");
    (first_of_next - chrono::Duration::days(1)).day()
}
