//! Local-time helpers. Timestamps are milliseconds since the Unix epoch.

use chrono::{Duration, Local, NaiveDate, TimeZone, Utc};

pub fn now_ms() -> i64 {
    Utc::now().timestamp_millis()
}

pub fn date_of(ms: i64) -> NaiveDate {
    Local.timestamp_millis_opt(ms).single().map(|d| d.date_naive()).unwrap_or_default()
}

/// Local midnight at the start of `date`. In the rare zones where a DST change skips midnight,
/// the day starts an hour later.
pub fn local_midnight(date: NaiveDate) -> i64 {
    let midnight = date.and_hms_opt(0, 0, 0).unwrap();
    Local
        .from_local_datetime(&midnight)
        .earliest()
        .or_else(|| Local.from_local_datetime(&(midnight + Duration::hours(1))).earliest())
        .map(|d| d.timestamp_millis())
        .unwrap_or_default()
}

pub fn day_start(ms: i64) -> i64 {
    local_midnight(date_of(ms))
}

/// `[start, end)` of a local day.
pub fn day_range(date: NaiveDate) -> (i64, i64) {
    (local_midnight(date), local_midnight(date.succ_opt().unwrap()))
}

/// `4h 12m`, or `12m` under an hour.
pub fn format_duration(ms: i64) -> String {
    let minutes = ms / 60_000;
    if minutes >= 60 {
        format!("{}h {}m", minutes / 60, minutes % 60)
    } else {
        format!("{minutes}m")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_durations() {
        assert_eq!(format_duration(59_999), "0m");
        assert_eq!(format_duration(12 * 60_000), "12m");
        assert_eq!(format_duration((4 * 60 + 12) * 60_000 + 30_000), "4h 12m");
    }

    #[test]
    fn day_range_covers_its_timestamps() {
        let now = now_ms();
        let (start, end) = day_range(date_of(now));
        assert!(start <= now && now < end);
        assert_eq!(day_start(now), start);
    }
}
