//! Time / duration helpers built on `chrono` and `humantime`.

use std::time::Duration;

use anyhow::Result;
use chrono::{DateTime, Utc};

/// Parse a human-friendly duration: `"1h30m"`, `"2d 4h"`, `"500ms"`.
pub fn parse_duration(s: &str) -> Result<Duration> {
    Ok(humantime::parse_duration(s)?)
}

/// Format a duration readably, e.g. `"1h 30m 5s"`.
pub fn humanize_duration(d: Duration) -> String {
    humantime::format_duration(d).to_string()
}

/// Current Unix timestamp (seconds).
pub fn now_ts() -> i64 {
    Utc::now().timestamp()
}

/// Current Unix timestamp (milliseconds).
pub fn now_ts_millis() -> i64 {
    Utc::now().timestamp_millis()
}

/// Convert a Unix timestamp (seconds) to a UTC datetime.
pub fn ts_to_datetime(ts: i64) -> DateTime<Utc> {
    DateTime::<Utc>::from_timestamp(ts, 0).unwrap_or(DateTime::UNIX_EPOCH)
}

/// Convert a UTC datetime to a Unix timestamp (seconds).
pub fn datetime_to_ts(dt: &DateTime<Utc>) -> i64 {
    dt.timestamp()
}

/// Human-readable "time ago" relative to `now` (English).
pub fn time_ago(dt: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let secs = now.signed_duration_since(dt).num_seconds().max(0);
    match secs {
        s if s < 60 => format!("{s}s ago"),
        s if s < 3600 => format!("{}m ago", s / 60),
        s if s < 86400 => format!("{}h ago", s / 3600),
        s if s < 2592000 => format!("{}d ago", s / 86400),
        s if s < 31536000 => format!("{}mo ago", s / 2592000),
        s => format!("{}y ago", s / 31536000),
    }
}

/// `time_ago` relative to now.
pub fn time_ago_now(dt: DateTime<Utc>) -> String {
    time_ago(dt, Utc::now())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_duration_examples() {
        assert_eq!(
            parse_duration("1h30m").unwrap(),
            Duration::from_secs(90 * 60)
        );
        assert_eq!(parse_duration("500ms").unwrap(), Duration::from_millis(500));
        assert_eq!(
            parse_duration("2d").unwrap(),
            Duration::from_secs(2 * 86400)
        );
    }

    #[test]
    fn parse_duration_invalid() {
        assert!(parse_duration("not a duration").is_err());
    }

    #[test]
    fn humanize_examples() {
        assert_eq!(humanize_duration(Duration::from_secs(5405)), "1h 30m 5s");
        assert_eq!(humanize_duration(Duration::from_secs(0)), "0s");
    }

    #[test]
    fn ts_roundtrip() {
        let now = Utc::now();
        let ts = datetime_to_ts(&now);
        let back = ts_to_datetime(ts);
        assert_eq!(back.timestamp(), ts);
    }

    #[test]
    fn ts_to_datetime_epoch() {
        assert_eq!(ts_to_datetime(0), DateTime::UNIX_EPOCH);
    }

    #[test]
    fn now_ts_is_recent() {
        let t = now_ts();
        assert!(t > 1_700_000_000);
        assert!(now_ts_millis() >= t * 1000);
    }

    #[test]
    fn time_ago_buckets() {
        let now = ts_to_datetime(1_000_000_000);
        let secs_30 = now - chrono::Duration::seconds(30);
        assert_eq!(time_ago(secs_30, now), "30s ago");
        let mins_5 = now - chrono::Duration::minutes(5);
        assert_eq!(time_ago(mins_5, now), "5m ago");
        let hours_2 = now - chrono::Duration::hours(2);
        assert_eq!(time_ago(hours_2, now), "2h ago");
        let days_3 = now - chrono::Duration::days(3);
        assert_eq!(time_ago(days_3, now), "3d ago");
        let future = now + chrono::Duration::seconds(10);
        assert_eq!(time_ago(future, now), "0s ago");
    }
}
